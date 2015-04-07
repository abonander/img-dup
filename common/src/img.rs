use image::{self, DynamicImage, GenericImage, ImageError};
use img_hash::{ImageHash, HashType};

use std::any::Any;
use std::borrow::ToOwned;
use std::fmt;
use std::path::PathBuf;
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
	fn load_and_hash_inner(path: PathBuf, settings: HashSettings) -> Result<Image, ImageError> {
		let (image, load_time) = duration_with_val(|| image::open(&path));
		let image: DynamicImage = try!(image);

		let (hash, hash_time) = duration_with_val(|| ImageHash::hash(&image,settings.hash_size, settings.hash_type));

		Ok(Image {
			path: path,
			hash: hash,
			dimensions: image.dimensions(),
			load_time: load_time,
			hash_time: hash_time,
		})
	}

    pub fn load_and_hash(path: PathBuf, settings: HashSettings) -> ImgDupResult {
        let move_path = path.clone();
	    let result = thread::catch_panic(move || Image::load_and_hash_inner(move_path, settings));

        match result {
            Ok(Ok(img)) => Ok(img),
            Ok(Err(load_err)) => Err((path, ImgDupError::Loading(load_err))),
            Err(box_any) => Err((path, ImgDupError::from_box_any(box_any))),
        }
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

pub enum ImgDupError {
	Loading(ImageError),
	Panicked(String),
}

impl ImgDupError {
    fn from_box_any(box_any: Box<Any + 'static>) -> ImgDupError {
        fn get_err_msg(box_any: Box<Any + 'static>) -> String {
            let box_any = match box_any.downcast::<String>() {
                Ok(panic_msg) => return (*panic_msg).clone(),
                Err(box_any) => box_any
            };

            match box_any.downcast::<&'static str>() {
                Ok(panic_msg) => (*panic_msg).to_owned(),
                Err(box_any) => format!("{:?}", box_any), 
            }
        }

        ImgDupError::Panicked(get_err_msg(box_any))
    }
}

impl fmt::Display for ImgDupError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ImgDupError::Loading(ref err) => fmt.write_fmt(format_args!("(loading error) {}", err)),
            ImgDupError::Panicked(ref msg) => fmt.write_fmt(format_args!("(panic) {}", msg)),
        }
    }
}

pub type ImgDupResult = Result<Image, (PathBuf, ImgDupError)>;

pub struct ImgResults {
    pub images: Vec<Image>,
    pub errors: Vec<(PathBuf, ImgDupError)>,
}

impl ImgResults {
    pub fn empty() -> ImgResults {
        ImgResults {
            images: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn push_result(&mut self, result: ImgDupResult) {
        match result {
			Ok(img) => self.images.push(img),
            Err(err) => self.errors.push(err),
		}        
    }

    pub fn total(&self) -> usize {
        self.images.len() + self.errors.len()
    }
}
