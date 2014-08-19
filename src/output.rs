extern crate time;

use processing::{ProcessingError, ImageResult, Total};
use img::{Image, UniqueImage};
use hash::ImageHash;
use parse_args::{ProgramSettings, JsonSettings, NoJson, Json, PrettyJson};

use self::time::{Tm, now};

use std::io::fs::File;
use std::io::stdio::stdout;
use std::io::IoResult;
use std::fmt::{Show, Formatter, FormatError};

macro_rules! json_insert {
    ($map:expr, $key:expr, $val:expr) => {
        $map.insert($key.into_string(), $val.to_json())
    }
}

pub fn output_results(results: &ProcessingResults) -> IoResult<()>{
    let out = open_output(results.settings);


}

fn open_output(&settings: ProgramSettings) -> Option<Writer> {
    match settings.outfile {
        Some(ref file) => File::open(file).ok(),
        None => Some(stdout()),
    }
}


