use img_hash::ImageHash;

use vp_tree::VpTree;
use vp_tree::dist::{DistFn, KnownDist};

use std::path::PathBuf;

use hash::HashSettings;

pub struct CollatedResults {
    pub tree: ImageTree,
    pub collate_time: u64,
    pub settings: HashSettings,
    pub errors: Vec<::Error>
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
        let left = &left.hash.bitv;
        let right = &right.hash.bitv;

        assert_eq!(left.len(), right.len());

        left.storage().iter().zip(right.storage())
            .fold(0u64, |count, (&left, &right)| count + (left ^ right).count_ones() as u64)
    }
}

impl KnownDist for HashedImage {
    type DistFn = ImageDistFn;

    fn dist_fn() -> Self::DistFn {
        ImageDistFn
    }
}

pub type ImageTree = VpTree<HashedImage, ImageDistFn>;
