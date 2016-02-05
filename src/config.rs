use getopts::{OptGroup, optopt, optmulti, optflag, optflagopt, Matches, usage, getopts};

use serialize::json::{ToJson, Json};

use std::borrow::ToOwned;

use std::collections::BTreeMap;

use std::fmt::{Display, Formatter};
use std::fmt::Result as FormatResult;

use std::path::{Path, PathBuf};

use std::env;

#[derive(Clone)]
pub struct ProgramSettings {
    pub threads: usize,
    pub dir: PathBuf,
    pub recurse: bool,
    pub exts: Vec<String>,
    pub hash_size: u32,
    pub threshold: f32,
    pub fast: bool,
    pub outfile: Option<PathBuf>,
    pub dup_only: bool,
    pub limit: usize,
    pub json: JsonSettings,
	pub gui: bool,
}

unsafe impl Send for ProgramSettings {}

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
            optflagopt("j", "json",
                       "Output the results in JSON format.
                       If outputting to stdout, normal output is suppressed.
                       An integer may optionally be passed with this flag,
                       indicating the number of spaces to indent per level.
                       Otherwise, the JSON will be in compact format.
                       See the README for details.",
                       "[1+] (optional)"),
			optflag("g", "gui",
				"Open the GUI. Given command-line flags will be set
				in the configuration dialog."),
        )
    }

    pub fn hash_settings(&self) -> HashSettings {
        HashSettings {
            hash_size: self.hash_size,
            fast: self.fast,
        }
    }

    pub fn silent_stdout(&self) -> bool {
        self.outfile.is_none() && self.json.is_json()
    }
}

impl Display for ProgramSettings {
    fn fmt(&self, fmt: &mut Formatter) -> FormatResult {
        try!(writeln!(fmt, "Threads: {}", self.threads));
        try!(writeln!(fmt, "Directory: {}", &self.dir.display()));
        try!(writeln!(fmt, "Recursive: {}", self.recurse));
        try!(writeln!(fmt, "Extensions: {:?}", self.exts));
        try!(writeln!(fmt, "Hash size: {}", self.hash_size));
        try!(writeln!(fmt, "Threshold: {0:.2}%", self.threshold * 100f32));
        writeln!(fmt, "Fast: {}", self.fast)
    }
}

impl ToJson for ProgramSettings {

    fn to_json(&self) -> Json {
        let mut my_json = BTreeMap::new();
        json_insert!(my_json, "threads", self.threads);
        json_insert!(my_json, "dir", self.dir.display().to_string());
        json_insert!(my_json, "recurse", self.recurse);
        json_insert!(my_json, "exts", self.exts.as_slice());
        json_insert!(my_json, "hash_size", self.hash_size);
        json_insert!(my_json, "threshold", self.threshold);
        json_insert!(my_json, "fast", self.fast);
        json_insert!(my_json, "limit", self.limit);

        Json::Object(my_json)
    }
}

#[derive(Copy, Clone)]
pub struct HashSettings {
    pub hash_size: u32,
    pub fast: bool,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum JsonSettings {
    NoJson,
    CompactJson,
    PrettyJson(usize),
}

impl JsonSettings {

    pub fn is_json(&self) -> bool {
        *self != JsonSettings::NoJson
    }
}

pub fn parse_args(args: &[String]) -> ProgramSettings {
    let settings_opts = ProgramSettings::opts();

    let ref opts = getopts(args, settings_opts.as_slice()).unwrap();

    if opts.opt_present("help") {
        print_help_and_exit(settings_opts.as_slice());
    }

    let exts_default = vec!("jpeg", "jpg", "png");

    let dir = dir_arg(opts, "dir", env::current_dir().unwrap());

    ProgramSettings {
        threads: usize_arg(opts, "threads", 4 /*FIXME os::num_cpus()*/),
        dir: dir.clone(),
        recurse: opts.opt_present("recurse"),
        hash_size: usize_arg(opts, "hash-size", 8) as u32,
        threshold: pos_f32_arg(opts, "threshold", 3f32) / 100f32,
        fast: opts.opt_present("fast"),
        exts: exts_args(opts, "ext", exts_default),
        outfile: outfile_arg(opts, "outfile", &dir),
        dup_only: opts.opt_present("dup-only"),
        limit: usize_arg(opts, "limit", 0),
        json: json_arg(opts, "json", JsonSettings::NoJson),
		gui: opts.opt_present("gui"),
    }
}

fn dir_arg<'a>(args: &'a Matches, arg: &'a str, default: PathBuf) -> PathBuf {
    let dir = args.opt_str(arg).map_or(default, |path| PathBuf::from(path) );

    assert!(dir.is_dir(), "Value passed to {} is not a directory: {}",
            arg, dir.display());

    dir
}

fn outfile_arg(args: &Matches, arg: &str, dir: &Path) -> Option<PathBuf> {
    args.opt_str(arg).map(|path| {
        let path = PathBuf::from(path);
        if path.is_relative() {
            dir.join(path)
        } else {
            path
        }
    })
}

fn usize_arg(args: &Matches, arg: &str, default: usize) -> usize {
    let val = args.opt_str(arg).map_or(default, |arg_str|
                arg_str.parse::<usize>().unwrap()
        );

    val
}

fn pos_f32_arg(args: &Matches, arg: &str, default: f32) -> f32 {
    let val = args.opt_str(arg)
        .map_or(default, |arg_str|
                arg_str.parse::<f32>().unwrap()
        );

    assert!(val > 0f32 && val < 100f32,
            "Value of {} must be a decimal between 0 and 100", arg);

    val
}

fn exts_args<'a>(args: &'a Matches, arg: &'a str, default: Vec<&'static str>) -> Vec<String> {
    if args.opt_present(arg) {
        args.opt_strs(arg)
    } else {
        default.into_iter().map(ToOwned::to_owned).collect()
    }
}

fn json_arg(args: &Matches, arg: &str, default: JsonSettings) -> JsonSettings {
    if args.opt_present(arg) {
        match args.opt_str(arg) {
            Some(indent) => JsonSettings::PrettyJson(indent.parse::<usize>().unwrap()),
            None => JsonSettings::CompactJson,
        }
    } else {
        default
    }
}

fn print_help_and_exit(opts: &[OptGroup]) {
    println!("{}", usage("Duplicate Image Finder", opts));
    super::exit();
}
