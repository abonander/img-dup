use image::{self, DynamicImage, GenericImage, ImageError};
use img_hash::ImageHash;

use vp_tree::VpTree;
use vp_tree::dist::DistFn;

use std::any::Any;
use std::borrow::ToOwned;
use std::fmt;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use std::path::PathBuf;
use img_hash::ImageHash;

pub struct Images {
    pub images: Vec<HashedImage>,
    pub settings: HashSettings,
}

impl Images {
    pub fn collate<F>(self, on_iter: F) where F: FnMut(usize) {

    }
}

pub struct CollagedImages {
    pub tree: VpTree<HashedImage, ImageDistFn>,
    pub settings: HashSettings,
}

#[derive(Eq, PartialEq, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub hash: ImageHash,
    pub dimensions: (u32, u32),
    pub size: u64,
    pub load_time: u64,
    pub hash_time: u64,
}

pub struct HashedImage {
    pub image: Image,
    pub hash: ImageHash,
    pub hash_time: u64,
}

#[derive(Copy, Clone)]
pub struct HashSettings {
    pub hash_size: u32,
    pub hash_type: HashType,
}

fn time_span_ms<T, F: FnOnce() -> T>(f: F) -> (T, u64) {
    let start = Instant::now();
    let val = f();
    (val, duration_millis(start.elapsed()))
}

fn duration_millis(duration: Duration) -> u64 {
    let ms_secs = duration.secs() * 1000;
    // 1 ms == 1M ns
    let ms_nanos = duration.subsec_nanos() as u64 / 1_000_000;

    ms_secs + ms_nanos
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
