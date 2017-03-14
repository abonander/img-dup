#[macro_use]
extern crate clap;
extern crate img_dup as common;

use clap::App;

fn is_nonzero(int: String) -> Result<(), String> {
    if int.parse::<u64>().is_err() {
        Err("must be an integer greater than zero.".into())
    } else {
        Ok(())
    }
}

fn app() -> App<'static, 'static> {
    clap_app! {
        @app (app_from_crate!())

        (@arg threads: -t --threads <integer> {is_nonzero} "The number of worker threads to use; \
                                              defaults to the number of logical CPUs (cores).")

        (@arg ext: -e --ext ... <extension> "Add one or more file extensions to the search parameters; \
                                        'gif', 'png', and 'jpg' are included by default.")

        (@arg no_default_exts: --("no-default-exts") "Don't include the default extensions \
                                                  ('gif', 'png', 'jpg').")

        (@arg recursive: -r --recursive "If supplied, recursively searches subdirectories.")

        (@arg outfile: -o --outfile <path> "The path for the results output; defaults to \
                                        'img-dup.json' in the current directory.")

        (@arg hash_size: -s --("hash-size") <integer> {is_nonzero}
              "The square of this number will be the number bits to use in the hash; \
               defaults to 8 (64).")

        (@arg hash_type: -h --("hash-type") <string> {validate_hash_type}
              "The hash type to use. Defaults to `gradient`. Run `img-dup --list-hash-types` to list all \
               the currently supported hash types.")

        (@arg list_hash_types: --("list-hash-types") "Print all the currently supported hash types \
                                                      and exit.")

        (@arg pretty_indent: --("pretty-indent") <integer> {is_nonzero}
              "Pretty-print the outputted JSON by the given number of spaces per indent level.")

        (@arg directory: "The directory to search; if not given, searches the current directory.")
    }
}

fn validate_hash_type(hash_type: String) -> Result<(), String> {
    unimplemented!();
}

fn main() {
    app().get_matches();
}
