use config::{ProgramSettings, JsonSettings, NoJson, Json, PrettyJson};
use processing::Results;

use serialize::Encodable;
use serialize::json::Encoder as JsonEncoder;
use serialize::json::{Json, ToJson, Object, PrettyEncoder};

use std::collections::TreeMap;
use std::io::fs::File;
use std::io::stdio::stdout;
use std::io::{IoResult};

pub fn newline_before_after(out: &mut Writer, what: |&mut Writer| -> IoResult<()>) -> IoResult<()> {
    try!(out.write_line(""));
    try!(what(out));
    out.write_line("")
}

pub fn output_results(settings: &ProgramSettings, results: &Results) -> IoResult<()>{
    let mut out_writer = open_output(settings);

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
        json_insert!(json, "images", results.uniques_json(dir));
        json_insert!(json, "errors", results.errors_json(dir));

        Object(json)
    };

    try!(json_encode(&settings.json, output, out));
    //Blank line at the end of the file
    out.write_line("")
}

fn json_encode<'a>(json_config: &JsonSettings, json: Json, out: &'a mut Writer) -> IoResult<()> {
    match *json_config {
        PrettyJson(indent) => { 
            let ref mut encoder = PrettyEncoder::with_indent(out, indent);
            json.encode(encoder)
        },
        Json => {
            let ref mut encoder = JsonEncoder::new(out);
            json.encode(encoder)
        },
        NoJson => unreachable!(),
    }
}

fn write_output(settings: &ProgramSettings, results: &Results, out: &mut Writer) -> IoResult<()> {
    try!(out.write_line("img-dup results follow.\nStats:"));
    try!(results.write_info(out));
    try!(out.write_line("\nImages:\n"));
    try!(results.write_uniques(out, &settings.dir));
    try!(out.write_line("\nErrors:\n"));
    results.write_errors(out, &settings.dir)    
}

fn open_output(settings: &ProgramSettings) -> Box<Writer> {
    match settings.outfile {
        Some(ref file) => box File::open(file).unwrap() as Box<Writer>,
        None => box stdout() as Box<Writer>,
    }
}

