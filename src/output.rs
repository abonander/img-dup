#![macro_escape]

use config::{ProgramSettings, JsonSettings, NoJson, Json, PrettyJson};
use processing::{ProcessingError, Results, ImageResult, Total};
use img::{Image, UniqueImage};
use hash::ImageHash;

use std::io::fs::File;
use std::io::stdio::stdout;
use std::io::IoResult;
use std::fmt::{Show, Formatter, FormatError};

enum 

#[macro_export]
macro_rules! json_insert {
    ($map:expr, $key:expr, $val:expr) => {
        $map.insert($key.into_string(), $val.to_json())
    }
}

pub fn output_results(settings: &ProgramSettings, results: &Results) -> IoResult<()>{
    let out = open_output(settings);

    let output = 

}

fn open_output(&settings: ProgramSettings) -> Box<Writer> {
    match settings.outfile {
        Some(ref file) => box File::open(file).ok(),
        None => box stdout(),
    }
}


