#[macro_use]
extern crate clap;
extern crate img_dup as common;

use clap::{App, ArgMatches, Error, ErrorKind};

use common::hash::{self, HashSettings};
use common::search::SearchSettings;
use common::serialize::SerializeSettings;

fn is_nonzero(int: String) -> Result<(), String> {
    match int.parse::<u64>() {
        Ok(0) | Err(_) => Err("must be an integer greater than zero".into()),
        _ => Ok(())
    }
}

fn is_int(int: String) -> Result<(), String> {
    match int.parse::<u64>() {
        Err(_) => Err("must be an integer".into()),
        _ => Ok(())
    }
}

fn app() -> App<'static, 'static> {
    clap_app! {
        @app (app_from_crate!())

        (about: "Finds and collates duplicate and similar image files, reporting them in a \
                       JSON file.")

        (@arg threads: -t --threads [integer] {is_nonzero}
              "The number of worker threads to use for loading and hashing; \
               defaults to the number of logical CPUs (cores).")

        (@arg ext: -e --ext ... [extension]
              "Add one or more file extensions to the search parameters; 'gif', 'png', and 'jpg' \
               are included by default.")

        (@arg no_default_exts: --("no-default-exts") requires[ext]
              "Don't include the default extensions ('gif', 'png', 'jpg').")

        (@arg recursive: -r --recursive "If supplied, recursively searches subdirectories.")

        (@arg outfile: -o --outfile [path]
              "The path for the results output; defaults to 'img-dup.json' in the \
               current directory.")

        (@arg hash_size: -s --("hash-size") [integer] {is_nonzero}
              "The square of this number will be the number bits to use in the hash; \
               defaults to 8 (64).")

        (@arg hash_type: -h --("hash-type") [string] {hash::validate_type}
              "The hash type to use. Defaults to `grad`. Run `img-dup --list-hash-types` to list \
               all the currently supported hash types.")

        (@arg k_nearest: -k [integer] --("k-nearest") {is_int} conflicts_with[distance]
              "Set the number of similar images to collect for each image; defaults to 5, \
               can be zero. Conflicts with `--distance`")

        (@arg distance: -d [integer] --distance {is_int}
              "Set the maximum number of bits between hashes to consider two images similar; \
               can be zero (match exact duplicates). Conflicts with `--k-nearest`.")

        (@arg list_hash_types: --("list-hash-types")
              "Print all the currently supported hash types and exit.")

        (@arg pretty_indent: --("pretty-indent") [integer] {is_nonzero}
              "Pretty-print the outputted JSON by the given number of spaces per indent level.")

        (@arg directory: "The directory to search; if not given, searches the current directory. \
                          Can be relative or absolute.")
    }
}

fn args_to_settings<'a>(args: &'a ArgMatches) -> AppSettings<'a> {
    use CompareType::*;

    let mut settings = AppSettings::default();

    macro_rules! opt_val {
        ($name:ident => $map:expr) => {
            if let Some($name) = args.value_of(stringify!($name)) {
                $map;
            }
        };
    }

    opt_val!(directory => settings.search.dir = directory.as_ref());
    opt_val!(outfile => settings.serialize.outfile = outfile.as_ref());
    opt_val!(threads => settings.threads = Some(threads.parse().unwrap()));
    opt_val!(hash_size => settings.hash.hash_size = hash_size.parse().unwrap());
    opt_val!(hash_type => settings.hash.hash_type = hash_type.parse().unwrap());
    opt_val!(k_nearest => settings.compare_type = KNearest(k_nearest.parse().unwrap()));
    opt_val!(distance => settings.compare_type = MaxDist(distance.parse().unwrap()));
    opt_val!(pretty_indent => settings.serialize.pretty_indent = Some(pretty_indent.parse().unwrap()));

    if args.is_present("no_default_exts") {
        settings.search.exts.clear();
    }

    settings.search.recursive = args.is_present("recursive");

    if let Some(exts) = args.values_of("ext") {
        settings.search.exts.extend(exts);

        assert!(!settings.search.exts.is_empty());
    }

    settings
}

#[derive(Default)]
struct AppSettings<'a> {
    search: SearchSettings<'a>,
    hash: HashSettings,
    serialize: SerializeSettings<'a>,
    threads: Option<usize>,
    compare_type: CompareType,
}

enum CompareType {
    KNearest(usize),
    MaxDist(u64),
}

impl Default for CompareType {
    fn default() -> Self {
        CompareType::KNearest(5)
    }
}

fn main() {
    let args = app().get_matches();

    if args.is_present("list_hash_types") {
        hash::print_types();
        return;
    }

    let settings = args_to_settings(&args);
}
