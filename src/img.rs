use img_hash::ImageHash;

use std::path::PathBuf;
use std::mem;

/// Nanoseconds
pub type LoadTime = u64;
/// Hash time of an image, in nanoseconds.
pub type HashTime = u64;

#[deriving(Eq, PartialEq, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub hash: ImageHash,
    pub dimensions: (u32, u32),
    pub load_time: LoadTime,
    pub hash_time: HashTime,
}

pub struct UniqueImage {
    pub img: Image,
    pub similars: Vec<SimilarImage>,
}

impl UniqueImage {
    pub fn from_image(img: Image) -> UniqueImage {
        UniqueImage {
           img: img,
           similars: Vec::new(),
        }
    }
    
    pub fn is_similar(&self, img: &Image, thresh: f32) -> bool {
        self.img.hash.dist_ratio(&img.hash) < thresh
    }
 
    pub fn add_similar(&mut self, img: Image) {
        let dist_ratio = self.img.hash.dist_ratio(&img.hash);

        self.similars.push(SimilarImage::from_image(img, dist_ratio));
    }

    pub fn promote(&mut self, idx: uint) {
        mem::swap(&mut self.similars[idx].img, &mut self.img);
        for similar in self.similars.iter_mut() {
            let dist_ratio = self.img.hash.dist_ratio(&similar.img.hash);
            similar.dist_ratio = dist_ratio;
        }
        
        self.similars.sort()
    } 
}

#[deriving(PartialEq, Eq, Clone)]
pub struct SimilarImage {
   pub img: Image, 
   // Distance from the containing UniqueImage
   pub dist_ratio: f32,
}

impl SimilarImage {

    fn from_image(img: Image, dist_ratio: f32) -> SimilarImage {
        SimilarImage {
            img: img,
            dist_ratio: dist_ratio,
        }
    } 
}

impl Ord for SimilarImage {
    fn cmp(&self, other: &SimilarImage) -> Ordering {
        self.partial_cmp(other).unwrap_or(Equal)   
    }
}

impl PartialOrd for SimilarImage {
    fn partial_cmp(&self, other: &SimilarImage) -> Option<Ordering> {
        self.dist_ratio.partial_cmp(&other.dist_ratio)                    
    }    
}

pub struct ImageManager {
    images: Vec<UniqueImage>,
    threshold: f32,
}

impl ImageManager {
    pub fn new(threshold: f32) -> Self {
        ImageManager {
            images: Vec::new(),
            threshold: threshold,
        }
    }

    pub fn add_image(&mut self, image: Image) {
        let parent = images.iter_mut()
            .find(|parent| parent.is_similar(&image, self.threshold));

        if let Some(parent) = parent {
            parent.add_similar(image);
        } else {
            self.images.push(UniqueImage::from_image(image);
        }
    }

    pub fn into_vec(self) -> Vec<UniqueImage> {
        self.images
    }
}

