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
use std::collections::BTreeMap;
use std::io::Result as IoResult;
use std::io::Write;
use std::thread;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

pub struct Results {
    pub total: Total,
    pub start_time: Tm,
    pub end_time: Tm,
    pub uniques: Vec<UniqueImage>,
    pub errors: Vec<ProcessingError>,
}

unsafe impl Send for Results {}

impl Results {

    fn start_time(&self) -> String {
        self.start_time.ctime().to_string()
    }

    fn end_time(&self) -> String {
        self.end_time.ctime().to_string()
    }

    pub fn info_json(&self) -> Json {
        let mut info = BTreeMap::new();
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

    pub fn write_info(&self, out: &mut Write) -> IoResult<()> {
        try!(writeln!(out, "Start time: {}", self.start_time()));
        try!(writeln!(out, "End time: {}", self.end_time()));
        try!(writeln!(out, "Images found: {}", self.total));
        try!(writeln!(out, "Processed: {}", self.uniques.len()));
        writeln!(out, "Errors: {}", self.errors.len())
    }

    pub fn write_uniques(&self, out: &mut Write, relative_to: &Path, dup_only: bool) -> IoResult<()> {
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

    pub fn write_errors(&self, out: &mut Write, relative_to: &Path) -> IoResult<()> {
        for error in self.errors.iter() {
            try!(
                newline_before_after(out,
                    |outa| error.write_self(outa, relative_to))
            );
        }

        Ok(())
    }
}

pub enum ProcessingError {
    Decoding(PathBuf, ImageError),
    Misc(PathBuf, String),
}

unsafe impl Send for ProcessingError {}

impl ProcessingError {

    pub fn relative_path<'a>(&'a self, relative_to: &'a Path) -> &'a Path {
        let path = match *self {
            ProcessingError::Decoding(ref path, _) => path,
            ProcessingError::Misc(ref path, _) => path,
        };

        path.strip_prefix(relative_to).unwrap_or(&path)
    }

    pub fn err_msg(&self) -> String {
        match *self {
            ProcessingError::Decoding(_, ref img_err) => format!("Loading error: {}", img_err),
            ProcessingError::Misc(_, ref misc_err) => format!("Hashing error: {}", misc_err),
        }
    }

    pub fn to_json(&self, relative_to: &Path) -> Json {
        let mut json = BTreeMap::new();

        json_insert!(json, "path", self.relative_path(relative_to).display().to_string());
        json_insert!(json, "error", self.err_msg());

        Json::Object(json)
    }

    pub fn write_self(&self, out: &mut Write, relative_to: &Path) -> IoResult<()> {
        writeln!(out, "Image: {}\n {}\n", self.relative_path(relative_to).display().to_string(), self.err_msg())
    }
}

/// Nanoseconds
pub type LoadTime = u64;
pub type HashTime = u64;

pub type ImageResult = Result<Image, ProcessingError>;

pub type TimedImageResult = Result<(Image, LoadTime, HashTime), ProcessingError>;

pub type Total = usize;

pub fn process(settings: &ProgramSettings, paths: Vec<PathBuf>) -> Results {
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

fn process_multithread(settings: &ProgramSettings, paths: Vec<PathBuf>)
    -> (Total, Vec<UniqueImage>, Vec<ProcessingError>) {
    let rx = spawn_threads(settings, paths);

    receive_images(rx, settings)
}

pub fn spawn_threads(settings: &ProgramSettings, paths: Vec<PathBuf>)
    -> Receiver<TimedImageResult> {

    let work = ParQueue::from_vec(paths).into_iter();

    let (tx, rx) = channel();

    let hash_settings = settings.hash_settings();

    for _ in 0..settings.threads {
        let task_tx = tx.clone();
        let mut task_work = work.clone();

        thread::spawn(move || {
            for path in task_work {
                let img_result = load_and_hash_image(&hash_settings, path);

                if task_tx.send(img_result).is_err() { break; }
            }
        });
    }

    rx
}

type ImageLoadResult = Result<DynamicImage, ImageError>;


fn try_fn<'a, T, F: FnOnce() -> T>(f: F) -> Result<T, Box<&'a str>> {
    let mut maybe: Option<T> = None;

    // FIXME
    Ok(f())

    // let err = unsafe { try(|| maybe = Some(f())) };

    // match maybe {
    //     Some(val) => Ok(val),
    //     None => Err(err.unwrap_err().downcast::<&str>().unwrap()),
    // }
}

fn load_and_hash_image(settings: &HashSettings, path: PathBuf) -> TimedImageResult {
    let start_load = precise_time_ns();
    let image = try_fn(|| image::open(&path));
    let load_time =  precise_time_ns() - start_load;

    match image {
        Ok(Ok(image)) => {
            let start_hash = precise_time_ns();
            let hash_type = if settings.fast { HashType::Mean } else { HashType::DCT };
            let hash = try!(try_hash_image(path, &image, settings.hash_size, hash_type));
            let hash_time = precise_time_ns() - start_hash;

            Ok((hash, load_time, hash_time))
        },
        Ok(Err(img_err)) => Err(ProcessingError::Decoding(path, img_err)),
        Err(cause) => Err(ProcessingError::Misc(path, cause.to_string())),
    }
}

fn try_hash_image(path: PathBuf, img: &DynamicImage, hash_size: u32, hash_type: HashType) -> ImageResult {
    let (width, height) = img.dimensions();

    match try_fn(|| ImageHash::hash(img, hash_size, hash_type)) {
        Ok(hash) => Ok(Image::new(path, hash, width, height)),
        Err(cause) => Err(ProcessingError::Misc(path, cause.to_string())),
    }
}

fn receive_images(rx: Receiver<TimedImageResult>, settings: &ProgramSettings)
    -> (Total, Vec<UniqueImage>, Vec<ProcessingError>){
    let mut unique_images = Vec::new();
    let mut errors = Vec::new();
    let mut total = 0usize;

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

pub fn find_images(settings: &ProgramSettings) -> Vec<PathBuf> {
    use std::fs;
    use walkdir;

    let exts: Vec<&str> = settings.exts.iter().map(|string| &string[..]).collect();

    if settings.recurse {
        walkdir::WalkDir::new(&settings.dir)
                .into_iter()
                .map(|f| f.unwrap().path().to_owned())
                .filter(|file| check_ext(file, &*exts))
                .collect()
    } else {
        fs::read_dir(&settings.dir)
            .unwrap()
            .map(|f| f.unwrap().path())
            .filter(|file| {
                !file.is_dir() && check_ext(file, &*exts)
            })
            .collect()
    }
}

fn check_ext(file: &Path, exts: &[&str]) -> bool {
    match file.extension().and_then(|s| s.to_str()) {
        Some(ext) => exts.iter().any(|&a| a.eq_ignore_ascii_case(ext)),
        None => false
    }
}

