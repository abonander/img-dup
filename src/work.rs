use futures::sync::{mpsc, oneshot};
use futures::executor::{self, Unpark, Spawn};
use futures::{future, Future, Stream};

use image::{self, DynamicImage, GenericImage};

use rayon::{self, ThreadPool};
use rayon::prelude::*;

use vp_tree::VpTree;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{BufReader, BufRead};
use std::sync::Arc;
use std::thread::{self, Thread};
use std::time::{Instant, Duration};
use std::mem;

use model::{Image, HashedImage, CollatedResults};
use hash::HashSettings;

struct LoadedImage {
    data: DynamicImage,
    image: Image,
}

impl LoadedImage {
    fn hash(self, settings: &HashSettings) -> HashedImage {
        let (hash, hash_time) = time_span_ms(|| settings.hash(&self.data));

        HashedImage {
            image: self.image,
            hash: hash,
            hash_time: hash_time,
        }
    }
}

fn time_span_ms<T, F: FnOnce() -> T>(f: F) -> (T, u64) {
    let start = Instant::now();
    let val = f();
    (val, duration_millis(start.elapsed()))
}

fn duration_millis(duration: Duration) -> u64 {
    let ms_secs = duration.as_secs() * 1000;
    // 1 ms == 1M ns
    let ms_nanos = duration.subsec_nanos() as u64 / 1_000_000;

    ms_secs + ms_nanos
}

fn load_image(path: PathBuf) -> Result<LoadedImage, ::Error> {
    let (res, load_time) = time_span_ms(|| {
        let mut reader = BufReader::new(File::open(&path)?);
        // Guess the format based on magic bytes instead of file extension since
        // the extension isn't always correct; this is a surprisingly common issue
        // with images from the web.
        let fmt = image::guess_format(reader.fill_buf()?)?;
        image::load(reader, fmt)
    });

    match res {
        Ok(data) => {
            let image = Image {
                path: path,
                dimensions: data.dimensions(),
                loaded_size: loaded_size_dyn(&data),
                load_time: load_time,
            };

            Ok(LoadedImage {
                data: data,
                image: image,
            })
        },
        Err(e) => Err(::Error {
            path: path,
            error: e.into(),
        })
    }
}

fn loaded_size_dyn(image: &DynamicImage) -> u64 {
    use self::DynamicImage::*;

    match *image {
        ImageLuma8(ref img) => loaded_size(&img),
        ImageLumaA8(ref img) => loaded_size(&img),
        ImageRgb8(ref img) => loaded_size(&img),
        ImageRgba8(ref img) => loaded_size(&img),
    }
}

fn loaded_size<SubPx>(subpx: &[SubPx]) -> u64 {
    subpx.len() as u64 * (mem::size_of::<SubPx>() as u64)
}

#[derive(Default)]
struct WorkResults {
    images: Vec<HashedImage>,
    errors: Vec<::Error>,
}

#[derive(Clone, Default)]
pub struct WorkStatus {
    pub count: usize,
    pub errors: usize,
    pub total_bytes: u64,
    // milliseconds spent loading
    pub load_time: u64,
    // milliseconds spent hashing
    pub hash_time: u64,
}

impl WorkStatus {
    fn add(&mut self, other: Self) {
        self.count += other.count;
        self.errors += other.errors;
        self.total_bytes += other.total_bytes;
        self.load_time += other.load_time;
        self.hash_time += other.hash_time;
    }
}

struct CollectWork {
    images: Vec<HashedImage>,
    errors: Vec<::Error>,
    unpark: Arc<Unpark>,
    status: WorkStatus,
    sender: Spawn<mpsc::Sender<WorkStatus>>,
}

impl CollectWork {
    fn new(sender: mpsc::Sender<WorkStatus>) -> Self {
        CollectWork {
            images: vec![],
            errors: vec![],
            // We'll be polling again shortly, no need to know
            unpark: Arc::new(IgnoreUnpark),
            status: WorkStatus::default(),
            sender: executor::spawn(sender),
        }
    }

    fn add_result(&mut self, res: WorkResult) {
        self.status.count += 1;

        match res {
            Ok(hashed) => {
                self.status.total_bytes += hashed.image.loaded_size;
                self.status.load_time += hashed.image.load_time;
                self.status.hash_time += hashed.hash_time;

                self.images.push(hashed);
            },
            Err(error) => {
                self.status.errors += 1;
                self.errors.push(error);
            },
        }
    }

    fn try_send_update(&mut self) {
        use futures::AsyncSink::Ready;

        match self.sender.start_send(self.status.clone(), &self.unpark) {
            Ok(Ready) => self.status = WorkStatus::default(),
            // If the other end is dropped or the queue isn't ready yet, we don't care
            _ => (),
        }
    }
}

type WorkResult = Result<HashedImage, ::Error>;

impl FromParallelIterator<CollectWork> for WorkResults {
    fn from_par_iter<P>(par_iter: P) -> Self where P: IntoParallelIterator<Item=CollectWork> {
        par_iter.into_par_iter().map(|collect|
            WorkResults {
                images: collect.images,
                errors: collect.errors
            }
        ).reduce(Self::default, |mut left, mut right| {
            left.images.append(&mut right.images);
            left.errors.append(&mut right.errors);
            left
        })
    }
}

pub struct Worker {
    pool: ThreadPool,
    paths: Vec<PathBuf>,
}

impl Worker {
    pub fn load_and_hash<F>(self, settings: HashSettings, mut during: F) -> LoadedAndHashed
    where F: FnMut(WorkStatus) + Send {
        let Self { pool, paths } = self;
        let threads = pool.num_threads();

        let (tx, rx) = mpsc::channel(1);

        let mut curr_status = WorkStatus::default();

        during(curr_status.clone());

        let during_fut = rx.for_each(|status|{
            curr_status.add(status);
            during(curr_status.clone());
            Ok(())
        });

        let results = pool.install(||
            rayon::join(
                || executor::spawn(during_fut).park_timeout(Duration::from_secs(1)).wait_future(),
                move || {
                    // Precompute DCT matrix on every thread
                    (0 .. threads).into_par_iter().weight_max()
                        .for_each(|_| settings.prepare());

                    paths.into_par_iter().fold(|| CollectWork::new(tx.clone()),
                                               |mut collect, path| {
                        let res = load_image(path).map(|image| image.hash(&settings));
                        collect.add_result(res);
                        collect.try_send_update();
                        collect
                    }).collect()
                }
            ).1
        );

        LoadedAndHashed {
            pool: pool,
            results: results,
            settings: settings
        }
    }
}

pub struct LoadedAndHashed {
    pool: ThreadPool,
    results: WorkResults,
    settings: HashSettings,
}

impl LoadedAndHashed {
    pub fn collate<F>(self, interval: Option<Duration>, mut during: F) -> CollatedResults
    where F: FnMut() + Send {
        let Self { pool, results, settings } = self;

        let images = results.images;

        let (tx, mut rx) = oneshot::channel();

        let during_fut = future::poll_fn(|| {
            during();
            rx.poll()
        });

        let (collated, collate_time) = pool.install(||
            rayon::join(
                move || poll_on_interval(interval, during_fut),
                move || {
                    let ret = time_span_ms(move || VpTree::from_vec(images));
                    let _ = tx.send(());
                    ret
                }
            ).1
        );

        CollatedResults {
            tree: collated,
            collate_time: collate_time,
            settings: settings,
            errors: results.errors,
        }
    }
}

fn poll_on_interval<F>(interval: Option<Duration>, fut: F) where F: Future {
    let _ = executor::spawn(fut).park_timeout(interval).wait_future();
}

struct IgnoreUnpark;

impl Unpark for IgnoreUnpark {
    fn unpark(&self) {}
}

pub fn worker(threads: Option<usize>, paths: Vec<PathBuf>) -> Worker {
    use rayon::Configuration;

    let config = if let Some(threads) = threads {
        Configuration::new().set_num_threads(threads)
    } else {
        Configuration::new()
    };

    Worker {
        pool: ThreadPool::new(config).expect("Error initializing thread pool"),
        paths: paths,
    }
}
