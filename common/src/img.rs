use img_hash::ImageHash;

use std::cmp::Ordering;
use std::mem;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Eq, PartialEq, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub hash: ImageHash,
    pub dimensions: (u32, u32),
    pub load_time: Duration,
    pub hash_time: Duration,
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
    
    pub fn is_similar(&self, img: &Image, thresh: usize) -> bool {
        self.img.hash.dist(&img.hash) < thresh
    }
 
    pub fn add_similar(&mut self, img: Image) {
        let dist = self.img.hash.dist(&img.hash);

        self.similars.push(SimilarImage::from_image(img, dist));
        self.similars.sort()
    }

    pub fn promote(&mut self, idx: usize) {
        mem::swap(&mut self.similars[idx].img, &mut self.img);
        for similar in self.similars.iter_mut() {
            let dist = self.img.hash.dist(&similar.img.hash);
            similar.dist = dist;
        }
        
        self.similars.sort()
    } 
}

#[derive(PartialEq, Eq, Clone)]
pub struct SimilarImage {
   pub img: Image, 
   // Distance from the containing UniqueImage
   pub dist: usize,
}

impl SimilarImage {

    fn from_image(img: Image, dist: usize) -> SimilarImage {
        SimilarImage {
            img: img,
            dist: dist,
        }
    } 
}

impl Ord for SimilarImage {
    fn cmp(&self, other: &SimilarImage) -> Ordering {
        self.dist.cmp(other.dist)   
    }
}

impl PartialOrd for SimilarImage {
    fn partial_cmp(&self, other: &SimilarImage) -> Option<Ordering> {
        Some(self.cmp(other))
    }    
}

impl Eq for SimilarImage {}

pub struct ImageManager {
    images: Vec<UniqueImage>,
    threshold: usize,
}

impl ImageManager {
    pub fn new(threshold: usize) -> Self {
        ImageManager {
            images: Vec::new(),
            threshold: threshold,
        }
    }

    pub fn add_image(&mut self, image: Image) {
        let threshold = self.threshold;

        match self.images.iter_mut().find(|parent| parent.is_similar(&image, threshold)) {
            Some(parent) => {                
                parent.add_similar(image);
                None
            },
            None => Some(image),
        }
        .map(|image| self.images.push(UniqueImage::from_image(image)));
    }

    pub fn into_vec(self) -> Vec<UniqueImage> {
        self.images
    }
}

