use config::{ProgramSettings, JsonSettings};
use processing::Results;

use serialize::Encodable;

use serialize::json::Encoder as JsonEncoder;
use serialize::json::{Json, ToJson, Object, PrettyEncoder};

use std::collections::TreeMap;

use std::io::fs::File;
use std::io::stdio::{stdout, StdWriter};
use std::io::{IoResult, LineBufferedWriter};

pub fn newline_before_after(out: &mut Writer, what: |&mut Writer| -> IoResult<()>) -> IoResult<()> {
    try!(out.write_line(""));
    try!(what(out));
    out.write_line("")
}

pub fn output_results(settings: &ProgramSettings, results: &Results) -> IoResult<()>{
    let ref mut out_writer = open_output(settings);

    if settings.json.is_json() {
        json_output(settings, results, out_writer)
    } else {
        write_output(settings, results, out_writer)
    }
}

fn json_output(settings: &ProgramSettings, results: &Results, out: &mut Writer) -> IoResult<()> { 
    let output = {
        let mut json = TreeMap::new();
 
        let dir = &settings.dir;

        json_insert!(json, "settings", settings);
        json_insert!(json, "info", results.info_json());
        json_insert!(json, "images", results.uniques_json(dir, settings.dup_only));
        json_insert!(json, "errors", results.errors_json(dir));

        Object(json)
    };

    try!(json_encode(&settings.json, output, out));
    //Blank line at the end of the file
    out.write_line("")
}

fn json_encode(json_config: &JsonSettings, json: Json, out: &mut Writer) -> IoResult<()> {
    match *json_config {
        JsonSettings::PrettyJson(indent) => { 
            let ref mut encoder = PrettyEncoder::new(out);
            encoder.set_indent(indent);
            json.encode(encoder)
        },
        JsonSettings::CompactJson => {
            let ref mut encoder = JsonEncoder::new(out);
            json.encode(encoder)
        },
        JsonSettings::NoJson => Ok(()),
    }
}

fn write_output(settings: &ProgramSettings, results: &Results, out: &mut Writer) -> IoResult<()> {
    try!(out.write_line("img-dup results follow.\nStats:"));
    try!(results.write_info(out));
    try!(out.write_line("\nImages:\n"));
    try!(results.write_uniques(out, &settings.dir, settings.dup_only));
    try!(out.write_line("\nErrors:\n"));
    results.write_errors(out, &settings.dir)    
}

fn open_output(settings: &ProgramSettings) -> Either<File, LineBufferedWriter<StdWriter>> {
    match settings.outfile {
        Some(ref file) => Either::Left(File::create(file).unwrap()),
        None => Either::Right(stdout()),
    }
}

/// Polymorphic `Writer` impl, to get around Rust issue #17322
enum Either<T, U> {
    Left(T),
    Right(U),
}

impl<T, U> Writer for Either<T, U> where T: Writer, U: Writer {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match *self {
            Either::Left(ref mut wrt) => wrt.write(buf),
            Either::Right(ref mut wrt) => wrt.write(buf),
        }
    }
}


/// Test if the outfile is writable by trying to open it in write mode.
pub fn test_outfile(outfile: &Path) -> IoResult<()> {
    File::create(outfile).map(|_| ())
}

