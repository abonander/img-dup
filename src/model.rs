use img_hash::ImageHash;

use vp_tree::VpTree;
use vp_tree::dist::DistFn;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use hash::{HashType, HashSettings};

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
    // The number of bytes in-memory
    pub loaded_size: u64,
    pub load_time: u64,
}

#[derive(Eq, PartialEq, Clone)]
pub struct HashedImage {
    pub image: Image,
    pub hash: ImageHash,
    pub hash_time: u64,
}

pub struct ImageDistFn;

impl DistFn<HashedImage> for ImageDistFn {
    fn dist(&self, left: &HashedImage, right: &HashedImage) -> u64 {
        left.hash.dist(&right.hash) as u64
    }
}
