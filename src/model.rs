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

use hash_types::HashType;

pub struct Images {
    pub images: Vec<HashedImage>,
    pub settings: HashSettings,
}

impl Images {
    pub fn collate<F>(self) -> CollatedImages {
        CollatedImages {
            tree: VpTree::from_vec_with_dist(self.images, ImageDistFn),
            settings: self.settings,
        }
    }
}

pub struct CollatedImages {
    pub tree: VpTree<HashedImage, ImageDistFn>,
    pub settings: HashSettings,
}

#[derive(Eq, PartialEq, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub dimensions: (u32, u32),
    pub size: u64,
    pub load_time: u64,
}

#[derive(Eq, PartialEq, Clone)]
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

struct ImageDistFn;

impl DistFn<HashedImage> for ImageDistFn {
    fn dist(&self, left: &Image, right: &Image) -> u64 {
        left.hash.dist(&right.hash) as u64
    }
}
