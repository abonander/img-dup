extern crate image;

use std::path::Path;

use hash::ImageHash;

#[deriving(Clone)]
pub struct Image {
    pub path: Path,
    pub hash: ImageHash,
    pub width: u32,
    pub height: u32,
}

impl Image {

    pub fn new(path: Path, hash: ImageHash, width: u32, height: u32) -> Image {
        Image {
            path: path,
            hash: hash,
            width: width,
            height: height,
        } 
    }

    fn relative_path(&self, relative_to: &Path) -> Path {
        self.path.path_relative_from(relative_to).unwrap_or(self.path.clone())
    }
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

    pub fn similars(&self) -> Vec<SimilarImage> {
        let mut temp = self.similars.clone();
        temp.sort_by(|a, b| a.dist_ratio.partial_cmp(&b.dist_ratio).unwrap());
        temp    
    }

    pub fn write_self(&self, out: &mut Writer, relative_to: &Path) {
        writeln!(out, "Original: ({}x{}) {} ", 
                    self.img.width, self.img.height,
                    self.img.relative_path(relative_to).display()
                );
        out.write_line("Similars [% different]:");
    
        for similar in self.similars().iter() {
            similar.write_self(out, relative_to);
        }

        out.write_char('\n');
    }

    pub fn similars_len(&self) -> uint {
        self.similars.len()
    }
}

#[deriving(Clone)]
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

    fn write_self(&self, out: &mut Writer, relative_to: &Path) {
        writeln!(out, "[{0:.2f}%] ({1}x{2}) {3}",
            self.dist_ratio * 100f32,
            self.img.width, self.img.height,
            self.img.relative_path(relative_to).display()
        );
    }
}

