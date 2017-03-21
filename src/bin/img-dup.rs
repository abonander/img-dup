#[macro_use]
extern crate clap;
extern crate img_dup as common;

use clap::{App, ArgMatches, Error, ErrorKind};

use common::hash_type;
use common::Settings;

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

fn is_float(float: String) -> Result<(), String> {
    match float.parse::<f64>() {
        Err(_) => Err("must be ")
    }
}

fn app() -> App<'static, 'static> {
    clap_app! {
        @app (app_from_crate!())

        (description: "Finds and collates duplicate image files, reporting them in a JSON file.")

        (@arg threads: -t --threads [integer] {is_nonzero}
              "The number of worker threads to use for loading and hashing; \
               defaults to the number of logical CPUs (cores).")

        (@arg ext: -e --ext ... [extension]
              "Add one or more file extensions to the search parameters; 'gif', 'png', and 'jpg' \
               are included by default.")

        (@arg no_default_exts: --("no-default-exts") "Don't include the default extensions \
                                                      ('gif', 'png', 'jpg').")

        (@arg recursive: -r --recursive "If supplied, recursively searches subdirectories.")

        (@arg outfile: -o --outfile [path] "The path for the results output; defaults to \
                                            'img-dup.json' in the current directory.")

        (@arg hash_size: -s --("hash-size") [integer] {is_nonzero}
              "The square of this number will be the number bits to use in the hash; \
               defaults to 8 (64).")

        (@arg hash_type: -h --("hash-type") [string] {hash_types::validate}
              "The hash type to use. Defaults to `grad`. Run `img-dup --list-hash-types` to list \
               all the currently supported hash types.")

        (@arg list_hash_types: --("list-hash-types") "Print all the currently supported hash types \
                                                      and exit.")

        (@arg pretty_indent: --("pretty-indent") [integer] {is_nonzero}
              "Pretty-print the outputted JSON by the given number of spaces per indent level.")

        (@arg directory: "The directory to search; if not given, searches the current directory.\
                          Can be relative or absolute.")
    }
}

fn args_to_settings(args: ArgMatches) -> Settings {
    let mut settings = Settings::default();

    macro_rules! opt_val {
        ($name:ident => $map:expr) => {
            if let Some($name) = args.value_of(stringify!($name)) {
                $map;
            }
        };
    }

    opt_val!(directory => settings.dir = directory.as_ref());
    opt_val!(outfile => settings.outfile = outfile.as_ref());
    opt_val!(threads => settings.threads = threads.parse().unwrap());
    opt_val!(hash_size => settings.hash_size = hash_size);
    opt_val!(hash_type => settings.hash_type = hash_type.parse().unwrap());
    opt_val!(k_nearest => settings.k_nearest = k_nearest.parse().unwrap());
    opt_val!(pretty_ident => settings.pretty_indent = Some(pretty_indent.parse().unwrap()));

    if args.is_present("no_default_exts") {
        settings.exts.clear();
    }

    settings.recursive = args.is_present("recursive");

    if Some(exts) = args.values_of("ext") {
        settings.exts.extend(exts);

        if settings.exts.is_empty() {
            Error::with_description("`--no-default-exts` was supplied but no extensions \
                                     were given with `--ext`", ErrorKind::MissingRequiredArgument)
                .exit();
        }
    }

    settings
}

fn main() {
    let args = app().get_matches();

    if args.is_present("list_hash_types") {
        hash_type::print_all();
        return;
    }

    let settings = args_to_settings(args);
}
