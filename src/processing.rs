use config::{ProgramSettings, HashSettings};
use img::{Image, UniqueImage};
use output::newline_before_after;
use par_queue::ParQueue;

use image;
use image::{DynamicImage, GenericImage, ImageError};

use img_hash::ImageHash;

use rustrt::unwind::try;
 
use serialize::json::{ToJson, Json};

use time::{Tm, now};

use std::boxed::BoxAny;
use std::collections::TreeMap;
use std::io::IoResult;

#[deriving(Send)]
pub struct Results {
    pub total: Total,
    pub start_time: Tm,
    pub end_time: Tm,
    pub uniques: Vec<UniqueImage>,
    pub errors: Vec<ProcessingError>,    
}

impl Results {

    fn start_time(&self) -> String {
        self.start_time.ctime().to_string()
    }

    fn end_time(&self) -> String {
        self.end_time.ctime().to_string()
    }    

    pub fn info_json(&self) -> Json {
        let mut info = TreeMap::new();
        json_insert!(info, "start", self.start_time());
        json_insert!(info, "end", self.end_time());
        json_insert!(info, "found", self.total);
        json_insert!(info, "processed", self.uniques.len());
        json_insert!(info, "errors", self.errors.len());

        Json::Object(info)
    }

    pub fn uniques_json(&self, relative_to: &Path, dup_only: bool) -> Json {
        let uniques_json: Vec<Json> = self.uniques.iter()
        .filter_map( |unique| 
                if dup_only && unique.similars.is_empty() {
                    None
                } else {
                    Some(unique.to_json(relative_to))
                }  
        ).collect();

        Json::Array(uniques_json)
    }

    pub fn errors_json(&self, relative_to: &Path) -> Json {
        let errors_json: Vec<Json> = self.errors.iter()
            .map( |error| error.to_json(relative_to) )
            .collect();

        Json::Array(errors_json)        
    }

    pub fn write_info(&self, out: &mut Writer) -> IoResult<()> {
        try!(writeln!(out, "Start time: {}", self.start_time()));
        try!(writeln!(out, "End time: {}", self.end_time()));
        try!(writeln!(out, "Images found: {}", self.total));
        try!(writeln!(out, "Processed: {}", self.uniques.len()));
        writeln!(out, "Errors: {}", self.errors.len())
    }

    pub fn write_uniques(&self, out: &mut Writer, relative_to: &Path, dup_only: bool) -> IoResult<()> {
        for unique in self.uniques.iter() {
            if dup_only && unique.similars.is_empty() {
                continue;
            } else {
                try!(
                    newline_before_after(out, 
                        |outa| unique.write_self(outa, relative_to))
                );
            }
        }

        Ok(())
    }

    pub fn write_errors(&self, out: &mut Writer, relative_to: &Path) -> IoResult<()> {
        for error in self.errors.iter() {
            try!(
                newline_before_after(out, 
                    |outa| error.write_self(outa, relative_to))
            );
        }

        Ok(())
    }
} 

#[deriving(Send)]
pub enum ProcessingError {
    Decoding(Path, ImageError),
    Misc(Path, String),
}

impl ProcessingError {
    
    pub fn relative_path(&self, relative_to: &Path) -> Path {
        let path = match *self {
            ProcessingError::Decoding(ref path, _) => path,
            ProcessingError::Misc(ref path, _) => path,
        };

        path.path_relative_from(relative_to).unwrap_or(path.clone())
    }

    pub fn err_msg(&self) -> String {
        match *self {
            ProcessingError::Decoding(_, ref img_err) => format!("Decoding error: {}", img_err),
            ProcessingError::Misc(_, ref misc_err) => format!("Processing error: {}", misc_err),
        }
    }

    pub fn to_json(&self, relative_to: &Path) -> Json {
        let mut json = TreeMap::new();

        json_insert!(json, "path", self.relative_path(relative_to).display().to_string());
        json_insert!(json, "error", self.err_msg());

        Json::Object(json)        
    }

    pub fn write_self(&self, out: &mut Writer, relative_to: &Path) -> IoResult<()> {
        writeln!(out, "Image: {}\n {}\n", self.relative_path(relative_to).display().to_string(), self.err_msg())
    }
}

pub type ImageResult = Result<Image, ProcessingError>;

pub type Total = uint;


pub fn process(settings: &ProgramSettings, paths: Vec<Path>) -> Results {
    let start_time = now();
   
    let (total, uniques, errors) = process_multithread(settings, paths);

    Results {
        total: total,
        start_time: start_time,
        end_time: now(),
        uniques: uniques,
        errors: errors,
    }    
}

fn process_multithread(settings: &ProgramSettings, paths: Vec<Path>)
    -> (Total, Vec<UniqueImage>, Vec<ProcessingError>) {                
    let rx = spawn_threads(settings, paths);

    receive_images(rx, settings)       
}

fn spawn_threads(settings: &ProgramSettings, paths: Vec<Path>) 
    -> Receiver<ImageResult> {
    
    let work = ParQueue::from_vec(paths).into_iter();

    let (tx, rx) = channel();

    let hash_settings = settings.hash_settings();

    for _ in range(0, settings.threads) {
        let task_tx = tx.clone();
        let mut task_work = work.clone(); 

        spawn(proc() {            
            for path in task_work {
                let img_result = load_and_hash_image(&hash_settings, path);
                                                
                if task_tx.send_opt(img_result).is_err() { break; }
            }
        });
    }

    rx
}

pub fn load_and_hash_image(settings: &HashSettings, path: Path) -> ImageResult {
    match image::open(&path) {
        Ok(image) => try_hash_image(path, &image,
                                    settings.hash_size, settings.fast),
        Err(img_err) => Err(ProcessingError::Decoding(path, img_err)),
    }
}

fn try_hash_image(path: Path, img: &DynamicImage, hash_size: u32, fast: bool) -> ImageResult {
    let (width, height) = img.dimensions(); 
    
    let img_hash = unsafe {
        let mut maybe_hash: Option<ImageHash> = None;

        let err = try(|| maybe_hash = Some(ImageHash::hash(img, hash_size, fast)) );

        match maybe_hash {
            Some(actual_hash) => Ok(actual_hash),
            None => {
                let err = err.unwrap_err().downcast::<&'static str>().unwrap();
                Err(err)                
            }
        }
    };

    match img_hash {
        Ok(hash) => Ok(Image::new(path, hash, width, height)),
        Err(cause) => Err(ProcessingError::Misc(path, cause.into_string())),
    }        
}

fn receive_images(rx: Receiver<ImageResult>, settings: &ProgramSettings) 
    -> (Total, Vec<UniqueImage>, Vec<ProcessingError>){
    let mut unique_images = Vec::new();
    let mut errors = Vec::new();
    let mut total = 0u;
   
    for img_result in rx.iter() {
        match img_result {
            Ok(image) => {
                manage_images(&mut unique_images, image, settings);
                total += 1;
            },
            Err(img_err) => errors.push(img_err),
        }                
    }

    (total, unique_images, errors)
}

fn manage_images(images: &mut Vec<UniqueImage>, 
                 image: Image, settings: &ProgramSettings) { 
    let parent_idx = images
        .iter()
        .enumerate()
        .find(|&(_, parent)| parent.is_similar(&image, settings.threshold))
        .map(|(idx, _)| idx);

    match parent_idx {
        Some(index) => images[index].add_similar(image),
        None => images.push(UniqueImage::from_image(image)),
    }
}

