extern crate image;
extern crate time;

use image::GenericImage;

use self::time::{Tm, now};

use std::os;
use std::io::fs;

use std::ascii::AsciiExt;

use std::sync::deque::{BufferPool, Data, Empty};

use std::task::deschedule;

use comp::{Image, UniqueImage};
use parse_args::{ProgramSettings, HashSettings, parse_args};
use hash::ImageHash;

mod comp;
mod hash;
mod parse_args;
mod dct;

fn main() {
    let args = os::args();

    let start_time = now();

    let settings = parse_args(args.as_slice());

    println!("Searching for images...");

    let mut image_paths = find_images(&settings.dir, 
                                      settings.exts.as_slice(), 
                                      settings.recurse);

    let image_count = image_paths.len();

    println!("Images found: {}", image_count);

    if settings.limit > 0 {
        println!("Limiting to: {}", settings.limit);
        image_paths.truncate(settings.limit);
    }

    println!("Processing images in {} threads. Please wait...\n", 
             settings.threads);

    let(processed, uniques) = process_multithread(&settings, image_paths);

    println!("");

    match settings.outfile {
        Some(ref out) => {
            // out should be unchanged if it was absolute to begin with
            let ref outfile = settings.dir.join(out);

            output_results(&settings,
                processed,
                start_time,
                uniques.as_slice(),
                &mut fs::File::create(outfile).unwrap())
        },
        None => output_results(&settings,
                               processed,
                               start_time,
                               uniques.as_slice(),
                               &mut std::io::stdio::stdout()),
    }
}

fn process_multithread(settings: &ProgramSettings, paths: Vec<Path>)
    -> (Total, Vec<UniqueImage>) {                
    let rx = spawn_threads(settings, paths);

    receive_images(rx, settings)       
}

fn spawn_threads(settings: &ProgramSettings, paths: Vec<Path>) 
    -> Receiver<Image> {
    let buffer: BufferPool<Path> = BufferPool::new();   
    let (worker, stealer) = buffer.deque();

    for path in paths.move_iter() {
        worker.push(path);
    }

    let (tx, rx) = channel::<Image>();

    let hash_settings = settings.hash_settings();

    for thread in range(0, settings.threads) {
        let task_stealer = stealer.clone();
        let task_tx = tx.clone();

        spawn(proc() {
            println!("Thread {} starting...", thread);
            
            loop {
                let image = match task_stealer.steal() {
                    Data(path) => load_and_hash_image(&hash_settings, path),
                    Empty => break,
                    _ => None,
                };

                match image {
                    Some(img) => { 
                        if task_tx.send_opt(img).is_err() { 
                            break;    
                        } 
                    },
                    _ => (),
                };

                deschedule();
            }

            println!("Thread {} dying...", thread);  

        });
    }

    rx
} 

fn load_and_hash_image(settings: &HashSettings, path: Path) -> Option<Image> {
    match image::open(&path) {
        Ok(image) => {
            let (width, height) = image.dimensions(); 
            let hash = ImageHash::hash(&image, settings.hash_size, settings.fast);
            Some(Image::new(path, hash, width, height))
        },
        _ => None,
    }
}

type Total = uint;

fn receive_images(rx: Receiver<Image>, settings: &ProgramSettings) 
    -> (Total, Vec<UniqueImage>){
    let mut unique_images: Vec<UniqueImage> = Vec::new();
    let mut total = 0u;
   
    for image in rx.iter() {
        manage_images(&mut unique_images, image, settings);
        total += 1;

        if total % 10u == 0 {
            println!("{} images processed.", total);
        }
       
        deschedule();
    }

    (total, unique_images)
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

fn find_images(dir: &Path, exts: &[String], recurse: bool) -> Vec<Path> {
    let exts: Vec<&str> = exts.iter().map(|string| string.as_slice()).collect();

    if recurse {
        fs::walk_dir(dir)
            .unwrap()
            .filter( |file| check_ext(file, exts.as_slice()) )
            .collect()   
    } else {
        fs::readdir(dir)
            .unwrap()
            .move_iter()
            .filter( |file| !file.is_dir() && check_ext(file, exts.as_slice()) )
            .collect()
    } 
}

fn check_ext<'a>(file: &'a Path, exts: &'a [&'a str]) -> bool {   
    match file.extension_str() {
        Some(ext) => exts.iter().any(|&a| a.eq_ignore_ascii_case(ext)),
        None => false
    }
} 

fn output_results(settings: &ProgramSettings, 
                  total: Total, start_time: Tm,
                  uniques: &[UniqueImage], out: &mut Writer) {
    let end_time = now();

    out.write_line("Image Duplicate Finder results\n");
    writeln!(out, "Start time: {}", start_time.ctime());
    writeln!(out, "End time: {}\n", end_time.ctime());

    writeln!(out, "Settings:\n{}", settings);

    writeln!(out, "Images processed: {} Original images: {}\n", 
             total, uniques.len());

    if settings.dup_only {
        out.write_line("Skipping images without duplicates.\n");
    }

    for unique in uniques.iter() {
        if settings.dup_only && unique.similars_len() == 0 {
            continue;
        }

        unique.write_self(out, &settings.dir);   
    }
}
