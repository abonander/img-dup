use getopts::{Options, Matches};

fn build_options() -> Options {
    let mut options = Options::new();

    options
        .optopt(
            "i", "infile",
            "The filename of the `results.json` from `img-dup scan`.
            Searches the current directory if not given.",
            "FILENAME/PATH",
        )        
        .optopt(
            "o", "outfile",
            "The filename/relative path for the results output.
            Defaults to 'collated.json' in the search directory.
            File will be truncated if present.",
            "FILENAME/PATH",
        )
        .optopt(
            "t", "threshold",
            "The number of bits an image's hash has to be different 
            from another's to count as unique. Defaults to 3.",
            "[1+]",
        )
        .optopt(
            "", "pretty-indent",
            "Pretty-print the outputted JSON with the given number of spaces.",
            "[0+]",
        );

    options
}

pub fn execute<I: Iterator<Item=String>>(args: I) {  
    let args = build_options().parse(args);

    match args {
        Ok(args) => execute_with_args(args),
        Err(msg) => {
            println!("{}", msg);
            print_usage();
        },
    }
}

fn execute_with_args(ref args: Matches) {
    
}

pub fn print_usage() {
   let usage = build_options().usage("Usage: img-dup collate [options]");
   println!("{}", usage);
}


