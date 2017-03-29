#[macro_use]
extern crate clap;
extern crate img_dup as common;

use clap::{App, ArgMatches};

use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use common::hash::{self, HashSettings};
use common::search::SearchSettings;
use common::serialize::SerializeSettings;
use common::work::{self, WorkStatus};

use format::*;

mod format;

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

        (@arg no_default_exts: --no_default_exts requires[ext]
              "Don't include the default extensions ('gif', 'png', 'jpg').")

        (@arg recursive: -r --recursive "If supplied, recursively searches subdirectories.")

        (@arg outfile: -o --outfile [path]
              "The path for the results output; defaults to 'img-dup.json' in the \
               current directory.")

        (@arg hash_size: -s --hash_size [integer] {is_nonzero}
              "The square of this number will be the number bits to use in the hash; \
               defaults to 8 (64).")

        (@arg hash_type: -h --hash_type [string] {hash::validate_type}
              "The hash type to use. Defaults to `grad`. Run `img-dup --list-hash-types` to list \
               all the currently supported hash types.")

        (@arg k_nearest: -k [integer] --k_nearest {is_int} conflicts_with[distance]
              "Set the number of similar images to collect for each image; defaults to 5, \
               can be zero. Conflicts with `--distance`")

        (@arg distance: -d [integer] --distance {is_int}
              "Set the maximum number of bits between hashes to consider two images similar; \
               can be zero (match exact duplicates). Conflicts with `--k-nearest`.")

        (@arg list_hash_types: --list_hash_types
              "Print all the currently supported hash types and exit.")

        (@arg pretty_indent: --pretty_indent [integer] {is_nonzero}
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

impl Display for CompareType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CompareType::KNearest(k) => write!(f, "nearest {} images", k),
            CompareType::MaxDist(dist) => write!(f, "within {} bits", dist),
        }
    }
}

fn main() {
    let args = app().get_matches();

    if args.is_present("list_hash_types") {
        hash::print_types();
        return;
    }

    let settings = args_to_settings(&args);

    println!("Searching for images...");

    let paths = SearchUi::new().find_images(&settings.search);

    println!("Hashing images (byte counts are after decompression)...");

    let mut hash_ui = HashUi::new();

    let results = work::worker(settings.threads, paths)
        .load_and_hash(settings.hash, |status| hash_ui.status_update(status));

    println!("\nHashing complete. Collating ({})...", settings.compare_type);

    let start = Instant::now();

    let collated = results.collate(Some(Duration::from_secs(1)), ||
        print!("\rCollating Elapsed: {}", Time(start.elapsed()))
    );

    println!("\nCollating Complete. Elapsed: {}", Time(start.elapsed()));
}

struct StatusUpdater {
    interval: Duration,
    last: Instant,
    stdout: io::Stdout,
}

impl StatusUpdater {
    fn new(interval: Duration) -> Self {
        let last = Instant::now() - interval;

        StatusUpdater {
            interval: interval,
            last: last,
            stdout: io::stdout(),
        }
    }

    fn update<F: FnOnce()>(&mut self, print: F) {
        use std::io::Write;

        if self.last.elapsed() < self.interval {
            return;
        }

        self.last = Instant::now();

        write!(self.stdout, "\r").expect("stdout has been closed");

        print();

        self.stdout.flush().expect("stdout has been closed");
    }
}

struct SearchUi {
    paths: Vec<PathBuf>,
    dirs_visited: u32,
    status: StatusUpdater,
}

impl SearchUi {
    fn new() -> Self {
        SearchUi {
            paths: vec![],
            dirs_visited: 1,
            status: StatusUpdater::new(Duration::from_millis(1)),
        }
    }

    fn find_images(mut self, settings: &SearchSettings) -> Vec<PathBuf> {
        settings.search(|event| {
            use common::search::WalkEvent::*;

            match event {
                File(pathbuf) => self.paths.push(pathbuf),
                Dir(_) => self.dirs_visited += 1,
                Error(_) => (),
            }

            self.status_update();

            true
        });

        let dirs_visited = Number(self.dirs_visited);
        let img_count = Number(self.paths.len());

        println!("\rDirectories Visited: {} Images Found: {}", dirs_visited, img_count);

        self.paths
    }

    fn status_update(&mut self) {
        let dirs_visited = Number(self.dirs_visited);
        let img_count = Number(self.paths.len());

        self.status.update(|| print!("Directories Visited: {} Images Found: {}",
                                     dirs_visited, img_count));
    }
}

struct HashUi {
    start: Instant,
    status: StatusUpdater
}

impl HashUi {
    fn new() -> Self {
        HashUi {
            start: Instant::now(),
            status: StatusUpdater::new(Duration::from_millis(250))
        }
    }

    fn status_update(&mut self, status: WorkStatus) {
        let elapsed = self.start.elapsed();
        self.status.update(|| print_work_status(status, elapsed));
    }
}

fn print_work_status(status: WorkStatus, elapsed: Duration) {
    // bytes / ms = kb / s
    let load_kbs = status.total_bytes.checked_div(status.load_time).unwrap_or(0) * 1000;
    let hash_kbs = status.total_bytes.checked_div(status.hash_time).unwrap_or(0) * 1000;

    let avg_load = status.load_time.checked_div(status.count as u64).unwrap_or(0);
    let avg_hash = status.hash_time.checked_div(status.count as u64).unwrap_or(0);

    print!("Elapsed: {} Processed: {} ({}) Load: {} ({} ms avg) Hash: {} ({} ms avg) Errors: {}",
           Time(elapsed), Number(status.count), Bytes(status.total_bytes), ByteRate(load_kbs),
           Number(avg_load), ByteRate(hash_kbs), Number(avg_hash), Number(status.errors))
}
