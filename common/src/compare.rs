use img::Image;

use std::cmp::Ordering;
use std::mem;

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
    
    pub fn is_similar(&self, img: &Image, thresh: u32) -> bool {
        self.img.hash.dist(&img.hash) < thresh as usize
    }
 
    pub fn add_similar(&mut self, img: Image) {
        let dist = self.img.hash.dist(&img.hash) as u32;

        self.similars.push(SimilarImage::from_image(img, dist));
        self.similars.sort()
    }

    pub fn promote(&mut self, idx: usize) {
        mem::swap(&mut self.similars[idx].img, &mut self.img);
        for similar in self.similars.iter_mut() {
            let dist = self.img.hash.dist(&similar.img.hash) as u32;
            similar.dist = dist;
        }
        
        self.similars.sort()
    } 
}

#[derive(PartialEq, Eq, Clone)]
pub struct SimilarImage {
   pub img: Image, 
   /// Distance, in bits, from the containing UniqueImage
   pub dist: u32,
}

impl SimilarImage {

    fn from_image(img: Image, dist: u32) -> SimilarImage {
        SimilarImage {
            img: img,
            dist: dist,
        }
    } 
}

impl Ord for SimilarImage {
    fn cmp(&self, other: &SimilarImage) -> Ordering {
        self.dist.cmp(&other.dist)   
    }
}

impl PartialOrd for SimilarImage {
    fn partial_cmp(&self, other: &SimilarImage) -> Option<Ordering> {
        Some(self.cmp(other))
    }    
}

pub struct ImageManager {
    images: Vec<UniqueImage>,
    threshold: u32,
}

impl ImageManager {
    pub fn new(threshold: u32) -> Self {
        ImageManager {
            images: Vec::new(),
            threshold: threshold,
        }
    }

    pub fn add(&mut self, image: Image) {
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
	
	pub fn add_all(&mut self, images: Vec<Image>) {
		for image in images {
			self.add(image);
		}
	}

    pub fn into_vec(self) -> Vec<UniqueImage> {
        self.images
    }
}
