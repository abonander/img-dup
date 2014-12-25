#![feature(macro_rules, globs, unsafe_destructor, phase)]

extern crate conrod;
extern crate current;
extern crate event;
extern crate file_dialog;
extern crate graphics;
extern crate getopts;
extern crate gl;
extern crate image;
extern crate img_hash;
extern crate libc;
#[phase(plugin, link)] extern crate log;
extern crate opengl_graphics;
extern crate sdl2;
extern crate sdl2_window;
extern crate serialize;
extern crate time;

use config::{parse_args, ProgramSettings};
use output::{output_results, test_outfile};
use processing::process;

use std::io::util::NullWriter;

use std::intrinsics;
use std::os;

macro_rules! json_insert(
    ($map:expr, $key:expr, $val:expr) => (
        $map.insert($key.into_string(), $val.to_json())
    );
);

mod config;
mod img;
mod output;
mod processing;
mod ui;
mod par_queue;

fn main() {
    run();

    // Exit immediately, don't leave any threads alive
    unsafe { libc::exit(0); }   
}

fn run() {
    let args = os::args();

    let settings = parse_args(args.as_slice());

	if settings.gui {
		ui::show_gui(settings);
		return;
	}

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

    let mut image_paths = processing::find_images(&settings);

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

fn get_output(settings: &ProgramSettings) -> Box<Writer> {
    if settings.silent_stdout() {
        box NullWriter as Box<Writer> 
    } else {
        box std::io::stdio::stdout() as Box<Writer>
    }    
}

