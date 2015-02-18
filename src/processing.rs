use config::{ProgramSettings, HashSettings};
use img::{Image, UniqueImage};
use output::newline_before_after;
use par_queue::ParQueue;

use image;
use image::{DynamicImage, GenericImage, ImageError};

use img_hash::{ImageHash, HashType};

use serialize::json::{ToJson, Json};

use time::{Tm, now, precise_time_ns};

use std::ascii::AsciiExt;
use std::boxed::BoxAny;
use std::collections::BTreeMap;
use std::io::IoResult;
use std::io::fs::PathExtensions;
use std::path::{Path, PathBuf};
use std::rt::unwind::try;
use std::sync::atomic::{AtomicUsize, Relaxed};
use std::thread::Thread;

#[derive(Clone)]
pub struct ParQueue {
    vec: Arc<Vec<PathBuf>>,
    curr: Arc<AtomicUsize>,
}

impl ParQueue {
    pub fn from_vec(vec: Vec<PathBuf>) -> ParQueue {
        ParQueue {
            vec: Arc::new(vec),
            curr: AtomicUsize::new(0),
        }
    }
}

impl Iterator for ParQueue {
    type Item = &Path;
    fn next(&mut self) -> Option<&Path> {
        let idx = self.curr.fetch_add(1, Relaxed);
        self.vec.get(idx)
    }
}

pub struct ImgResults {
    pub total: Total,
    pub start_time: Tm,
    pub end_time: Tm,
    pub uniques: Vec<UniqueImage>,
    pub errors: Vec<ProcessingError>,
}
   

pub enum ProcessingError {
    Decoding(PathBuf, ImageError),
    Misc(PathBuf, String),
}

pub struct HashSettings {
    pub hash_size: u32,
    pub hash_type: HashType,
    pub threshold: f32,
}

/// Nanoseconds
pub type LoadTime = u64;
/// Hash time of an image, in nanoseconds.
pub type HashTime = u64;

pub type ImageResult = Result<Image, ProcessingError>;

pub type TimedImageResult = Result<(Image, LoadTime, HashTime), ProcessingError>;

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

pub fn spawn_threads(settings: &ProgramSettings, paths: Vec<Path>)
    -> Receiver<TimedImageResult> {

    let work = ParQueue::from_vec(paths);

    let (tx, rx) = channel();

    let hash_settings = settings.hash_settings();

    for _ in range(0, settings.threads) {
        let task_tx = tx.clone();
        let mut task_work = work.clone();

        Thread::spawn(move || {
            for path in task_work {
                let img_result = load_and_hash_image(&hash_settings, path);

                if task_tx.send_opt(img_result).is_err() { break; }
            }
        });
    }

    rx
}

type ImageLoadResult = Result<DynamicImage, ImageError>;


fn try_fn<T, F: FnOnce() -> T>(f: F) -> Result<T, String> {
    let mut maybe: Option<T> = None;

    let err = unsafe { try(|| maybe = Some(f())) };

    match maybe {
        Some(val) => Ok(val),
        None => Err(err.unwrap_err().downcast().unwrap()),
    }
}

fn load_and_hash_image(settings: &HashSettings, path: Path) -> TimedImageResult {
    let start_load = precise_time_ns();
    let image = try_fn(|| image::open(&path));
    let load_time =  precise_time_ns() - start_load;

    match image {
        Ok(Ok(image)) => {
            let start_hash = precise_time_ns();
            let hash = try!(try_hash_image(path, &image, settings.hash_size, settings.fast));
            let hash_time = precise_time_ns() - start_hash;

            Ok((hash, load_time, hash_time))
        },
        Ok(Err(img_err)) => Err(ProcessingError::Decoding(path, img_err)),
        Err(cause) => Err(ProcessingError::Misc(path, cause.to_string())),
    }
}

fn try_hash_image(path: Path, img: &DynamicImage, hash_size: u32, fast: bool) -> ImageResult {
    let (width, height) = img.dimensions();

    match try_fn(|| ImageHash::hash(img, hash_size, fast)) {
        Ok(hash) => Ok(Image::new(path, hash, width, height)),
        Err(cause) => Err(ProcessingError::Misc(path, cause.to_string())),
    }
}

fn receive_images(rx: Receiver<TimedImageResult>, settings: &ProgramSettings)
    -> (Total, Vec<UniqueImage>, Vec<ProcessingError>){
    let mut unique_images = Vec::new();
    let mut errors = Vec::new();
    let mut total = 0u;

    for img_result in rx.iter() {
        match img_result {
            Ok((image, _, _)) => {
                manage_images(&mut unique_images, image, settings);
                total += 1;
            },
            Err(img_err) => errors.push(img_err),
        }
    }

    (total, unique_images, errors)
}

pub fn manage_images(images: &mut Vec<UniqueImage>,
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

