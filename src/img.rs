use image::{self, DynamicImage, GenericImage, ImageError};
use img_hash::{ImageHash, HashType};

use vp_tree::VpTree;
use vp_tree::dist::DistFn;

use std::any::Any;
use std::borrow::ToOwned;
use std::fmt;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Eq, PartialEq, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub hash: ImageHash,
    pub dimensions: (u32, u32),
    pub size: u64,
}

impl Image {
	pub fn hash(path: PathBuf, image: DynamicImage, settings: HashSettings, size: u64) -> Image {

		let hash = ImageHash::hash(&image,settings.hash_size, settings.hash_type);

		Image {
			path: path,
			hash: hash,
			dimensions: image.dimensions(),
		    size: size,
        }
	}
}

#[derive(Copy, Clone)]
pub struct HashSettings {
    pub hash_size: u32,
    pub hash_type: HashType,
}

fn duration_with_val<T, F: FnOnce() -> T>(f: F) -> (T, Duration) {
    let start = Instant::now();
    let val = f();
    (val, start.elapsed())
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

struct ImageDistFn;

impl DistFn<Image> for ImageDistFn {
    fn dist(&self, left: &Image, right: &Image) -> u64 {
        left.hash.dist(&right.hash) as u64
    }
}

pub struct ImageManager {
    tree: VpTree<Image, ImageDistFn>,
}
