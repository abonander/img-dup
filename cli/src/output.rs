extern crate img_dup_common as common;

use config::{ProgramSettings, JsonSettings};
use processing::Results;

use serialize::Encodable;

use serialize::json::Encoder as JsonEncoder;
use serialize::json::{Json, PrettyEncoder, ToJson};

use std::collections::BTreeMap;

use std::io::fs::File;
use std::io::stdio::{stdout, StdWriter};
use std::io::{IoResult, LineBufferedWriter};

pub fn newline_before_after<F: FnOnce(&mut Writer) -> IoResult<()>>(out: &mut Writer, what: F) -> IoResult<()> {
    out.write_line("")
        .and_then(|| what(out))
        .and_then(|| out.write_line(""))
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
        JsonSettings::NoJson => return Ok(()),
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

fn open_output(settings: &ProgramSettings) -> Box<Writer> {
    match settings.outfile {
        Some(ref file) => Box::new(File::create(file).unwrap()),
        None => Box::new(stdout()),
    }
}

/// Test if the outfile is writable by trying to open it in write mode.
pub fn test_outfile(outfile: &Path) -> IoResult<()> {
    File::create(outfile).map(|_| ())
}
