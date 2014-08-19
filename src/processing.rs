extern crate serialize;

use output::json_insert;

use serialize::json::{ToJson, Json, Object};

use std::rt::unwind::try;
use std::sync::Future;
use std::sync::deque::{BufferPool, Data, Empty};
use std::task::deschedule;
use std::to_string::ToString;

#[deriving(Send)]
pub struct Results {
    total: Total,
    start_time: Tm,
    end_time: Tm,
    uniques: Vec<UniqueImage>,
    errors: Vec<ProcessingError>,    
}

#[deriving(Send)]
pub enum ProcessingError {
    DecodingError(Path, ImageError),
    MiscError(Path, String),
}

impl Show for ProcessingError {
    
    fn fmt(&self, out: &mut Formatter) -> Result<(), FormatError> {
        match self {
            DecodingError(&path, &img_err) =>
                writeln!(out, "Error decoding image: {}\nReason: {}", path.display(), img_err),
            MiscError(&path, &err) =>
                writeln!(out, "Error processing image: {}\nReason: {}", path.display(), err.as_slice()),
        }
    }
}



impl ToJson for Results {

    fn to_json(&self) -> Json {
        let mut info = Treemap::new();
        json_insert!(info, "start", self.start_time.ctime());
        json_insert!(info, "end", self.end_time.ctime());
        json_insert!(info, "found", self.total);
        json_insert!(info, "processed", self.uniques.len());
        json_insert!(info, "errors", self.errors.len());

        let mut my_json = Treemap::new();
        json_insert!(my_json, info);
        json_insert!(my_json, "images", self.images.as_slice());
        json_insert!(my_json, "errors", self.errors.as_slice());

        my_json.to_json();
    }
} 

pub type ImageResult = Result<Image, ProcessingError>;

pub type Total = uint;

pub fn process_future(settings: &ProgramSettings, paths: Vec<Path>) -> Future<Results> {
    Future::spawn(proc(){ process(settings, paths) })    
}

pub fn process(settings: &ProgramSettings, paths: Vec<Path>) -> Results {
    let start_time = now();
   
    let (total, uniques, errors) = process_multithread(settings, paths);

    Results {
        total: total,
        start_time: start_time,
        end_time: end_time,
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
    let buffer: BufferPool<Path> = BufferPool::new();   
    let (worker, stealer) = buffer.deque();

    for path in paths.move_iter() {
        worker.push(path);
    }

    let (tx, rx) = channel();

    let hash_settings = settings.hash_settings();

    for thread in range(0, settings.threads) {
        let task_stealer = stealer.clone();
        let task_tx = tx.clone();

        spawn(proc() {            
            loop {
                let img_result = match task_stealer.steal() {
                    Data(path) => load_and_hash_image(&hash_settings, path),
                    Empty => break,
                    _ => None,
                };
               
                if task_tx.send_opt(img_result).is_err() { 
                    break;    
                }

                deschedule();
            }
        });
    }

    rx
}

fn load_and_hash_image(settings: &HashSettings, path: Path) -> ImageResult {
    match image::open(&path) {
        Ok(image) => try_hash_image(path, &image,
                                    settings.hash_size, settings.fast),
        Err(img_err) => Err(DecodingError(path, img_err)),
    }
}

fn try_hash_image(path: Path, img: &Image, hash_size: u32, fast: bool) -> Result<Image, String> {
    let (width, height) = img.dimensions(); 
    
    let img_hash = unsafe {
        let mut hash: Option<Image> = None;

        try(|| hash = ImageHash::hash(img, hash_size, fast) );

        hash
    };

    match img_hash {
        Ok(hash) => Ok(Image::new(path, hash, width, height)),
        Err(cause) => Err(MiscError(path, cause.to_string())),
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
       
        deschedule();
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
        Some(index) => {
            let parent = images.get_mut(index);

            parent.add_similar(image);
        },
        None => images.push(UniqueImage::from_image(image)),
    }
}
