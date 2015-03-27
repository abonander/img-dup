use image::{self, DynamicImage, GenericImage, ImageError};
use img_hash::{ImageHash, HashType};

use std::path::{Path, PathBuf};
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

#[derive(Copy)]
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
	Panicked,	
}

pub enum ImgStatus {
    Unhashed(PathBuf),
    Hashed(Image),
    Error(PathBuf, ImageError),
}

impl ImgStatus {
    pub fn hash(&mut self, settings: HashSettings) -> &ImgStatus {
        let result = if let ImgStatus::Unhashed(ref path) = *self {
			Image::load_and_hash(path, settings)	
        } else { return self; };

		match result {
			Ok(img) => *self = ImgStatus::Hashed(img),
			Err(img_err) => self.unhashed_to_error(img_err),
		}

		self
    }

	fn unhashed_to_error(&mut self, err: ImageError) {
		let new_self = match *self {
			ImgStatus::Unhashed(ref path) => ImgStatus::Error(path.clone(), err),
			_ => unimplemented!(),
		};

		*self = new_self;
	}

	pub fn is_err(&self) -> bool {
		match *self {
			ImgStatus::Error(_, _) => true,
			_ => false,
		}
	}
}

pub struct ImgResults {
    pub uniques: Vec<Image>,
    pub errors: Vec<(PathBuf, ImgDupError)>,
}

impl ImgResults {
	pub fn from_statuses(statuses: Vec<ImgStatus>) -> ImgResults {
		let mut errors = Vec::new();
		let uniques = statuses.into_iter()
			.filter_map(|status| 
				match status {
					ImgStatus::Hashed(img) => Some(img),
					ImgStatus::Unhashed(path) => {
						errors.push((path, ImgDupError::Panicked));
						None
					},
					ImgStatus::Error(path, err) => {
						errors.push((path, ImgDupError::Loading(err)));
						None
					},
			})
			.collect();

		ImgResults {
			uniques: uniques,
			errors: errors,
		}
	}
}
