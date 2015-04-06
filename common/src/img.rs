use image::{self, DynamicImage, GenericImage, ImageError};
use img_hash::{ImageHash, HashType};

use std::any::Any;
use std::borrow::ToOwned;
use std::fmt;
use std::mem;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[derive(Eq, PartialEq, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub hash: ImageHash,
    pub dimensions: (u32, u32),
    pub load_time: Duration,
    pub hash_time: Duration,
}

impl Image {
	fn load_and_hash(path: &Path, settings: HashSettings) 
	-> Result<Image, ImageError> {
		let (image, load_time) = duration_with_val(|| image::open(path));
		let image: DynamicImage = try!(image);

		let (hash, hash_time) = duration_with_val(|| ImageHash::hash(&image,settings.hash_size, settings.hash_type));

		Ok(Image {
			path: path.to_path_buf(),
			hash: hash,
			dimensions: image.dimensions(),
			load_time: load_time,
			hash_time: hash_time,
		})
	}
}

#[derive(Copy, Clone)]
pub struct HashSettings {
    pub hash_size: u32,
    pub hash_type: HashType,
}

fn duration_with_val<T, F: FnOnce() -> T>(f: F) -> (T, Duration) {
    let mut opt_val: Option<T> = None;
    let duration = Duration::span(|| opt_val = Some(f()));
    (opt_val.unwrap(), duration)
}

pub type ImgDupResult = Result<Image, ImgDupError>;

pub enum ImgDupError {
	Loading(ImageError),
	Panicked(String),
}

impl ImgDupError {
    fn from_box_any(box_any: Box<Any + 'static>) -> ImgDupError {
        fn get_err_msg(box_any: Box<Any + 'static>) -> String {
            let box_any = match box_any.downcast::<String>() {
                Some(panic_msg) => return (*panic_msg).clone(),
                Err(box_any) => box_any
            };

            match box_any.downcast::<&'static str>() {
                Some(panic_msg) => (*panic_msg).to_owned(),
                Err(box_any) => format!("{:?}", box_any), 
            }
        }

        ImgDupError::Panicked(get_err_msg(box_any))
    }
}

impl fmt::Display for ImgDupError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ImgDupError::Loading(ref img_err) => 
                fmt.write_fmt(format_args!("(loading error) {}", img_err)),
            ImgDupError::Panicked => fmt.write_str(
                "An unexpected error occurred while processing this image.
                Please see the stderr feed for more info."
            ),
        }
    }
}

#[derive(Clone)]
pub enum ImgStatus {
    Taken,
    Unhashed(PathBuf),
    Hashed(Image),
    Error(PathBuf, ImgDupError),
}

impl ImgStatus {
    pub fn hash(&mut self, settings: HashSettings) -> &ImgStatus {
        if let ImgStatus::Unhashed(path) = mem::replace(self, ImgStatus::Taken) {
			let result = thread::catch_panic(Image::load_and_hash(&path, settings));

            *self = match result {
			    Ok(Ok(img)) => ImgStatus::Hashed(img),
			    Ok(Err(img_err)) => ImgStatus::Error(path, ImgDupError::LoadingError(img_err)), 
                Err(box_any) => ImgStatus::Error(path, ImgDupError::from_box_any(box_any)),
		    };
        }

		self
    }

	pub fn is_err(&self) -> bool {
		match *self {
			ImgStatus::Error(_, _) => true,
			_ => false,
		}
	}
}

pub struct ImgResults {
    pub images: Vec<Image>,
    pub errors: Vec<(PathBuf, ImgDupError)>,
}

impl ImgResults {
	pub fn from_statuses(statuses: Vec<ImgStatus>) -> ImgResults {
		let mut errors = Vec::new();
		let images = statuses.into_iter()
			.filter_map(|status| 
				match status {
					ImgStatus::Hashed(img) => Some(img),
					ImgStatus::Error(path, err) => {
						errors.push((path, err));
						None
					},
                    _ => None,
			})
			.collect();

		ImgResults {
			images: images,
			errors: errors,
		}
	}
}
