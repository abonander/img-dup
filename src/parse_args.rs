extern crate getopts;

use std::os;
use self::getopts::{OptGroup, optopt, optmulti, optflag, Matches, 
    usage, getopts};

use std::fmt::{Show, FormatError, Formatter};

pub struct ProgramSettings {
    pub threads: uint,
    pub dir: Path,
    pub recurse: bool,
    pub exts: Vec<String>,    
    pub hash_size: u32,
    pub threshold: f32,
    pub fast: bool,
    pub outfile: Option<Path>,
    pub dup_only: bool,
    pub limit: uint,
}

impl ProgramSettings {

    fn opts() -> Vec<OptGroup> {
        vec!(
            optopt("t", "threads",
                   "How many threads the program should use to process images.
                   Defaults to the number of cores reported by the OS.",
                   "[1+]"),
            optopt("d", "dir",
                   "The directory the program should search in. 
                   Default is the current working directory.",
                   "[directory]"),
            optflag("r", "recurse",
                    "If present, the program will search subdirectories."),
            optopt("h", "hash-size",
                   "Helps the program decide the number of bits to use for the hash.
                   A higher number means more detail, but greater memory usage.
                   Default is 8",
                   "[1+]"),
            optopt("s", "threshold",
                   "The amount in percentage that an image must be different from
                   another to qualify as unique. Default is 3",
                   "[0.01 - 99.99]"),
            optflag("f", "fast",
                    "Use a faster, less accurate algorithm.
                    Really only useful for finding duplicates.
                    Using a low threshold and/or a larger hash is recommended."),
            optmulti("e", "ext",
                     "Search for filenames with the given extension.
                     Defaults are jpeg, jpg, png, and gif.",
                     "[extension]"), 
            optopt("o", "outfile",
                   "Output to the given file. If omitted, will print to stdout.
                   If not absolute, it will be relative to the search directory.",
                   "[file]"),
            optflag("", "help",
                   "Display this help."),
            optflag("u", "dup-only",
                    "Only output images with similars or duplicates."),
            optopt("l", "limit",
                   "Only process the given number of images.",
                   "[1+]"),
        )
    }

    pub fn hash_settings(&self) -> HashSettings {
        HashSettings {
            hash_size: self.hash_size,
            fast: self.fast,
        }          
    }
}

impl Show for ProgramSettings {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        writeln!(fmt, "Threads: {}", self.threads);
        writeln!(fmt, "Directory: {}", &self.dir.display());
        writeln!(fmt, "Recursive: {}", self.recurse);
        writeln!(fmt, "Extensions: {}", self.exts.as_slice());
        writeln!(fmt, "Hash size: {}", self.hash_size);
        writeln!(fmt, "Threshold: {0:.2f}%", self.threshold * 100f32);
        writeln!(fmt, "Fast: {}", self.fast);
        Ok(())
    }
}

pub struct HashSettings {
    pub hash_size: u32,
    pub fast: bool,
}

pub fn parse_args(args: &[String]) -> ProgramSettings {
    let settings_opts = ProgramSettings::opts();
    
    let ref opts = getopts(args, settings_opts.as_slice()).unwrap();
    
    if opts.opt_present("help") {
        print_help_and_exit(settings_opts.as_slice());    
    }

    let exts_default = vec!("jpeg", "jpg", "png");

    ProgramSettings {
        threads: uint_arg(opts, "threads", os::num_cpus()),
        dir: dir_arg(opts, "dir", os::getcwd()),
        recurse: opts.opt_present("recurse"),
        hash_size: uint_arg(opts, "hash-size", 8) as u32,
        threshold: pos_f32_arg(opts, "threshold", 3f32) / 100f32,
        fast: opts.opt_present("fast"),
        exts: exts_args(opts, "ext", exts_default),
        outfile: opts.opt_str("outfile").map(|path| Path::new(path.as_slice())),
        dup_only: opts.opt_present("dup-only"),
        limit: uint_arg(opts, "limit", 0),
    }    
}

fn dir_arg(args: &Matches, arg: &str, default: Path) -> Path {
    let dir = args.opt_str(arg).map_or(default, |path| Path::new(path) );

    assert!(dir.is_dir(), "Value passed to {} is not a directory: {}", 
            arg, dir.display());

    dir
}

fn uint_arg(args: &Matches, arg: &str, default: uint) -> uint {
    let val = args.opt_str(arg).map_or(default, |arg_str|   
                from_str::<uint>(arg_str.as_slice()).unwrap()
        );

    val
}

fn pos_f32_arg(args: &Matches, arg: &str, default: f32) -> f32 {
    let val = args.opt_str(arg)
        .map_or(default, |arg_str|
                from_str::<f32>(arg_str.as_slice()).unwrap()
        );
    
    assert!(val > 0f32 && val < 100f32, 
            "Value of {} must be a decimal between 0 and 100", arg);

    val
}

fn exts_args<'a>(args: &'a Matches, arg: &'a str, default: Vec<&'static str>) 
    -> Vec<String> {
    if args.opt_present(arg) {
        args.opt_strs(arg)
    } else {
        default.iter().map(|str_slice| str_slice.into_string()).collect()
    }
}

fn print_help_and_exit(opts: &[OptGroup]) {
    println!("{}", usage("Duplicate Image Finder", opts));

    fail!("Exiting...");
}
