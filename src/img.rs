use img_hash::ImageHash;

use serialize::json::{Json, ToJson};

use std::collections::BTreeMap;
use std::io::IoResult;
use std::mem;
use std::path::Path;

#[deriving(Eq, PartialEq, Clone)]
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

    pub fn to_treemap(&self, relative_to: &Path) -> BTreeMap<String, Json> {
        let mut json = BTreeMap::new();

        json_insert!(json, "path", self.relative_path(relative_to).display().to_string());
        json_insert!(json, "hash", self.hash.to_base64());
        json_insert!(json, "width", &self.width);
        json_insert!(json, "height", &self.height);

        json
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
        temp.sort();
        temp    
    }

    pub fn write_self(&self, out: &mut Writer, relative_to: &Path) -> IoResult<()> {
        try!(writeln!(out, "Original: ({}x{}) {} ", 
                    self.img.width, self.img.height,
                    self.img.relative_path(relative_to).display()
                ));
        
        try!(out.write_line("Similars [% different]:"));
    
        for similar in self.similars().iter() {
            try!(similar.write_self(out, relative_to));
        }

        out.write_char('\n')
    }

    pub fn to_json(&self, relative_to: &Path) -> Json {
        let mut json = self.img.to_treemap(relative_to);

        let similars_json: Vec<Json> = self.similars.iter()
            .map( |similar| similar.to_json(relative_to) )
            .collect();

        json_insert!(json, "similars", similars_json);

        Json::Object(json)
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

#[deriving(PartialEq, Clone)]
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

    fn write_self(&self, out: &mut Writer, relative_to: &Path) -> IoResult<()> {
        writeln!(out, "[{0:.2}%] ({1}x{2}) {3}",
            self.dist_ratio * 100f32,
            self.img.width, self.img.height,
            self.img.relative_path(relative_to).display()
        )
    }

    fn to_json(&self, relative_to: &Path) -> Json {
        let mut json = self.img.to_treemap(relative_to);

        json_insert!(json, "diff", self.dist_ratio);

        Json::Object(json)
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

impl Eq for SimilarImage {}

