#![feature(macro_rules)]

extern crate getopts;
extern crate image;
extern crate serialize;
extern crate time;

use config::{parse_args, ProgramSettings};
use output::{output_results, test_outfile};
use processing::process;

use std::ascii::AsciiExt;
use std::io::fs;
use std::io::util::NullWriter;
use std::os;

macro_rules! json_insert(
    ($map:expr, $key:expr, $val:expr) => (
        $map.insert($key.into_string(), $val.to_json())
    );
)

mod config;
mod dct;
mod img;
mod hash;
mod output;
mod processing;

fn main() {
    let args = os::args();

    let settings = parse_args(args.as_slice());

    // Silence standard messages if we're outputting JSON
    let mut out = get_output(&settings);    

    match settings.outfile {
        Some(ref outfile) => {
            (writeln!(out, "Testing output file ({})...",
                outfile.display())).unwrap();
            test_outfile(outfile).unwrap();
        },
        None => (),        
    };
    
    out.write_line("Searching for images...").unwrap();

    let mut image_paths = find_images(&settings.dir, 
                                      settings.exts.as_slice(), 
                                      settings.recurse);

    let image_count = image_paths.len();

    (writeln!(out, "Images found: {}", image_count)).unwrap();

    if settings.limit > 0 {
        (writeln!(out, "Limiting to: {}", settings.limit)).unwrap();
        image_paths.truncate(settings.limit);
    }

    (writeln!(out, "Processing images in {} threads. Please wait...\n", 
             settings.threads)).unwrap();

    let results = processing::process(&settings, image_paths);

    out.write_line("").unwrap();

    output::output_results(&settings, &results).unwrap()   
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

fn get_output(settings: &ProgramSettings) -> Box<Writer> {
    if settings.silent_stdout() {
        box NullWriter as Box<Writer> 
    } else {
        box std::io::stdio::stdout() as Box<Writer>
    }    
}

