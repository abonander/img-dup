use common::{ 
    HashType,
    ImageSearch,
    ImgResults,
    SessionBuilder,
    ThreadedSession
};

use common::serialize::SerializeSession;

use getopts::{Options, Matches};

use std::env;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf}; 

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
            "The number of threads to use for hashing.
            Defaults to the number of CPUs (cores).",
            "[1+]",
        )
        .optmulti(
            "", "ext",
            "Add a file extension to search parameters.
            'gif', 'png', and 'jpg' are included by default.",
            "EXTENSION",
        )
        .optflag(
            "r", "recursive",
            "If supplied, recursively search subdirectories.",
        )
        .optopt(
            "o", "outfile",
            "The filename/relative path for the results output.
            Defaults to 'results.json' in the search directory.
            File will be truncated if present.",
            "FILENAME/PATH",
        )
        .optopt(
            "i", "hash-size",
            "The number of bits to use in the hash when squared. Defaults to 8 (64)",
            "[1+]",
        )
        .optopt(
            "h", "hash-type",
            "The type of the hash to use. Defaults to `gradient`.",
            "mean|gradient|dbl-gradient|dct|fftw",
        )
        .optopt(
            "", "pretty-indent",
            "Pretty-print the outputted JSON with the given number of spaces.",
            "[0+]",
        );

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

    println!("Opening output file (create|truncate)...");
    
    let mut outfile = match get_outfile(args, &search_dir) {
        Ok(outfile) => outfile,
        Err(msg) => {
            println!("An error occurred while opening the output file: {}", msg);
            return;
        }
    };

    println!("Output file opened.");

    println!("Searching for images...");

    let images = match image_search.search() {
        Ok(images) => images,
        Err(msg) => {
            println!("An error occurred while searching: {}", msg);
            return;
        },
    };

    println!("{} images found.", images.len());

    let mut builder = SessionBuilder::from_images(images);

    if !(
        set_hash_size(args, &mut builder) && 
        set_hash_type(args, &mut builder)
    ) { return; }

    let hash_size = builder.hash_size;

    let threads = match get_threads(args) {
        ::GetOptResult::Some(threads) => Some(threads),
        ::GetOptResult::None => None,
        ::GetOptResult::Err(msg) => {
            println!("{}", msg);
            return;
        },
    };

    let pretty_indent = match get_pretty_indent(args) {
        ::GetOptResult::Some(pretty_indent) => Some(pretty_indent),
        ::GetOptResult::None => None,
        ::GetOptResult::Err(msg) => {
            println!("{}", msg);
            return;
        },
    };

    let session = builder.process_multithread(threads);
    
    let results = monitor_session(session);

    println!("\nProcessing complete. Errored images follow.");

    for (path, msg) in results.errors {
        let path = path.relative_from(&search_dir);
        println!("{} -> {}", path.unwrap().display(), msg);
    }

    println!("Writing results to file.");

    SerializeSession::from_images(results.images.iter(), hash_size)
        .write_json(&mut outfile, pretty_indent)
        .unwrap()      
}

fn monitor_session(session: ThreadedSession) -> ImgResults {
    let ref mut stdout = io::stdout();
    
    loop {
        write!(
            stdout, 
            "\r{} images processed. ({} errors)",
            session.status().done(),
            session.status().errors()
        ).unwrap();
        stdout.flush().unwrap();
        
        if !session.status().is_done() {
            session.status().wait_for_update();
        } else {
            break;
        }
    }

    session.wait()
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

static DEFAULT_OUTFILE: &'static str = "results.json";

fn get_outfile(args: &Matches, dir: &Path) -> io::Result<File> {
    let ref path = args.opt_str("outfile")
        .map_or_else(
            || dir.join(DEFAULT_OUTFILE),
            |outfile| dir.join(outfile),
        );

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
}

fn set_hash_size(args: &Matches, builder: &mut SessionBuilder) -> bool {
    let arg = match args.opt_str("hash-size") {
        Some(arg) => arg,
        None => return true,
    };

    match arg.parse::<u32>() {
        Ok(hash_size) if hash_size > 0 => {
            builder.hash_size(hash_size);
            return true;
        },
        Ok(_) => println!("Please enter an integer greater than 0 for 'hash-size'."),
        Err(_) => println!("Unknown input for 'hash-size': {:?}", arg),
    }

    false
}

fn set_hash_type(args: &Matches, builder: &mut SessionBuilder) -> bool {
    let arg = match args.opt_str("hash-type") {
        Some(arg) => arg,
        None => return true,
    };

    let hash_type = match arg.trim() {
        "mean" => HashType::Mean,
        "gradient" => HashType::Gradient,
        "dbl-gradient" => HashType::DoubleGradient,
        "dct" => HashType::DCT,
        "fftw" => unimplemented!(),
        hash_type => {
            println!("Unknown value for 'hash-type': {:?}", hash_type);
            return false;
        }
    };

    builder.hash_type(hash_type);

    true
}

fn get_threads(args: &Matches) -> ::GetOptResult<usize> {
    use ::GetOptResult::*;

    let arg: ::GetOptResult<_> = From::from(args.opt_str("threads"));

    arg.and_then(|arg| match arg.parse::<usize>() {
            Ok(threads) => Some(threads),
            Result::Err(_) => Err(format!("Unknown value for 'threads': {}", arg)),
        })
        .and_then(|threads| 
            if threads == 0 {
                Err("Value for 'threads' must be greater than 0!".to_string())
            } else {
                Some(threads)
            }
        )
}

fn get_pretty_indent(args: &Matches) -> ::GetOptResult<u32> {
   use ::GetOptResult::*;

    let arg: ::GetOptResult<_> = From::from(args.opt_str("pretty-indent"));

    arg.and_then(|arg| match arg.parse::<u32>() {
        Ok(threads) => Some(threads),
        Result::Err(_) => Err(format!("Unknown value for 'pretty-indent': {}", arg)),
    })
}
