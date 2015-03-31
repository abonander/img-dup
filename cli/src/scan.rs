use common::ImageSearch;

use getopts::{Options, Matches};

use std::env;
use std::path::PathBuf; 

fn build_options() -> Options {
    let mut options = Options::new();

    options
        .optopt(
            "", "dir",
            "The directory to search. If not given, searches current dir.",
            "DIRECTORY",
        )
        .optopt(
            "t", "threads",
            "The number of threads to use for hashing. Defaults to the number of CPUs (cores).",
            "THREADS",
        )
        .optmulti(
            "", "ext",
            "Add a file extension to search parameters.\n
            'gif', 'png', and 'jpg' are included by default.",
            "EXTENSION",
        )
        .optflag(
            "r", "recursive",
            "If supplied, recursively search subdirectories."
        )
        .optopt(
            "o", "output",
            ;

    options
}

pub fn execute<I: Iterator<Item=String>>(args: I) {  
    let args = build_options().parse(args);

    match args {
        Ok(args) => execute_with_args(args),
        Err(msg) => {
            println!("{}", msg);
            print_usage();
        },
    }
}

fn execute_with_args(ref args: Matches) {
    let search_dir = get_search_dir(args);   
    let exts = args.opt_strs("ext");

    let mut image_search = ImageSearch::with_dir(&search_dir);
    image_search.recursive(args.opt_present("recursive"));

    for ext in exts.iter() {
        image_search.ext(ext);
    }

    println!("Searching for images...");

    let images = match image_search.search() {
        Ok(images) => images,
        Err(msg) => {
            println!("An error occurred while searching: {}", msg);
            return;
        },
    };

    println!("{} images found.", images.len());              
}

pub fn print_usage() {
   let usage = build_options().usage("Usage: img-dup scan [options]");
   println!("{}", usage);
}

fn get_search_dir(args: &Matches) -> PathBuf {
    args.opt_str("dir")
        .map_or_else(
            // I know what you're thinking, but this is correct.
            // github.com/rust-lang/rfcs/issues/1025
            || env::current_dir().unwrap(),             
            |dir| From::from(dir),
        )
}

