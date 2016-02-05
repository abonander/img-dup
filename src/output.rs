use config::{ProgramSettings, JsonSettings};
use processing::Results;

use serialize::Encodable;

use serialize::json::Encoder as JsonEncoder;
use serialize::json::{Json, PrettyJson, ToJson};

use std::collections::BTreeMap;

use std::fs::File;
use std::io::{Stdout, stdout};
use std::io::{Write, Result as IoResult};
use std::path::Path;

pub fn newline_before_after<F: FnOnce(&mut Write) -> IoResult<()>>(out: &mut Write, what: F) -> IoResult<()> {
    try!(writeln!(out, ""));
    try!(what(out));
    writeln!(out, "")
}

pub fn output_results(settings: &ProgramSettings, results: &Results) -> IoResult<()>{
    let ref mut out_writer = open_output(settings);

    if settings.json.is_json() {
        json_output(settings, results, out_writer)
    } else {
        write_output(settings, results, out_writer)
    }
}

fn json_output(settings: &ProgramSettings, results: &Results, out: &mut Write) -> IoResult<()> {
    let output = {
        let mut json = BTreeMap::new();

        let dir = &settings.dir;

        json_insert!(json, "settings", settings);
        json_insert!(json, "info", results.info_json());
        json_insert!(json, "images", results.uniques_json(dir, settings.dup_only));
        json_insert!(json, "errors", results.errors_json(dir));

        Json::Object(json)
    };

    try!(json_encode(&settings.json, output, out));
    //Blank line at the end of the file
    writeln!(out, "")
}

fn json_encode(json_config: &JsonSettings, json: Json, out: &mut Write) -> IoResult<()> {
    unimplemented!()
    // match *json_config {
    //     JsonSettings::PrettyJson(indent) => {
    //         let ref mut encoder = PrettyEncoder::new(out);
    //         encoder.set_indent(indent);
    //         json.encode(encoder)
    //     },
    //     JsonSettings::CompactJson => {
    //         let ref mut encoder = JsonEncoder::new(out);
    //         json.encode(encoder)
    //     },
    //     JsonSettings::NoJson => return Ok(()),
    // }
}

fn write_output(settings: &ProgramSettings, results: &Results, out: &mut Write) -> IoResult<()> {
    try!(writeln!(out, "img-dup results follow.\nStats:"));
    try!(results.write_info(out));
    try!(writeln!(out, "\nImages:\n"));
    try!(results.write_uniques(out, &settings.dir, settings.dup_only));
    try!(writeln!(out, "\nErrors:\n"));
    results.write_errors(out, &settings.dir)
}

fn open_output(settings: &ProgramSettings) -> Either<File, Stdout> {
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

impl<T, U> Write for Either<T, U> where T: Write, U: Write {
    fn flush(&mut self) -> IoResult<()> {
        match *self {
            Either::Left(ref mut wrt) => wrt.flush(),
            Either::Right(ref mut wrt) => wrt.flush(),
        }
    }

    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
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

