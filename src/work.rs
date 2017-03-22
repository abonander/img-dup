use futures::sync::{mpsc, oneshot};
use futures::executor::{self, Unpark, Spawn};
use futures::{future, Future, Sink};

use image::{self, DynamicImage, GenericImage};

use img_hash::ImageHash;

use rayon::{self, ThreadPool};
use rayon::prelude::*;

use vp_tree::VpTree;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{self, BufReader, BufRead, Read, Seek, SeekFrom};
use std::sync::Arc;
use std::thread::{self, Thread};
use std::time::{Instant, Duration};
use std::mem;

use model::{Image, HashedImage, CollatedResults};
use hash::HashSettings;
use search::SearchSettings;

struct LoadedImage {
    data: DynamicImage,
    image: Image,
}

impl LoadedImage {
    pub fn hash(self, settings: &HashSettings) -> HashedImage {
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
    settings: HashSettings,
}

#[derive(Clone)]
pub struct WorkStatus {
    pub elapsed: Duration,
    pub count: usize,
    pub errors: usize,
    pub total_bytes: u64,
    // milliseconds spent loading
    pub load_time: u64,
    // milliseconds spent hashing
    pub hash_time: u64,
}

type WorkResult = Result<HashedImage, ::Error>;

impl FromParallelIterator<WorkResult> for WorkResults {
    fn from_par_iter<P>(par_iter: P) -> Self where P: IntoParallelIterator<Item=WorkResult> {
        par_iter.into_par_iter().fold(Self::default, |mut results, result| {
            match result {
                Ok(success) => results.success.push(success),
                Err(error) => results.error.push(error),
            }

            results
        }).reduce(Self::default, |mut left, mut right| {
            left.success.append(&mut right.success);
            left.error.append(&mut right.error);
            left
        })
    }
}

pub struct Worker {
    pool: ThreadPool,
}

impl Worker {
    pub fn search<F>(self, settings: SearchSettings, mut with_path: F) -> WorkerReady
        where F: FnMut(&Path) {
        let mut paths = vec![];

        settings.search(
            |path| {
                with_path(&path);
                paths.push(path);
            },
            // Continue on errors for now
            |_| true
        );

        WorkerReady {
            pool: self.pool,
            paths: paths,
        }
    }
}

pub struct WorkerReady {
    pool: ThreadPool,
    paths: Vec<PathBuf>,
}

impl WorkerReady {
    pub fn load_and_hash<F>(self, settings: HashSettings, during: F) -> LoadedAndHashed
    where F: FnMut(WorkStatus) + Send {
        let Self { pool, paths } = self;
        let threads = pool.num_threads();

        let (tx, rx) = mpsc::channel(1);

        let start = Instant::now();

        let mut curr_status = WorkStatus {
            elapsed: Duration::from_secs(0),
            count: 0,
            errors: 0,
            total_bytes: 0,
            load_time: 0,
            hash_time: 0,
        };

        let during_fut = rx.for_each(|mut status|{
            status.elapsed = start.elapsed();
            during(status);
            Ok(())
        });

        let mut status_update = |res: &WorkResult| {
            curr_status.count += 1;

            match *res {
                Ok(ref hashed) => {
                    curr_status.total_bytes += hashed.image.loaded_size;
                    curr_status.load_time += hashed.image.load_time;
                    curr_status.hash_time += hashed.hash_time;
                },
                Err(_) => curr_status.errors += 1,
            }

            tx.send(curr_status.clone());
        };

        let mut results = pool.install(||
            rayon::join(
                || executor::spawn(during_fut).wait_future(),
                move || {
                    // Precompute DCT matrix on every thread
                    (0 .. threads).into_par_iter().weight_max()
                        .for_each(|_| settings.prepare());

                    paths.into_par_iter().map(|path| {
                        let res = load_image(path).map(|image| image.hash(&settings));
                        status_update(&res);
                        res
                    }).collect()
                }
            ).1
        );

        results.settings = settings;

        LoadedAndHashed {
            pool: pool,
            results: results,
        }
    }
}

pub struct LoadedAndHashed {
    pool: ThreadPool,
    results: WorkResults,
}

impl LoadedAndHashed {
    pub fn collate<F, Fut>(self, interval: Option<Duration>, during: F) -> CollatedResults
    where F: FnMut(Duration) + Send {
        let Self { pool, results } = self;

        let images = results.images;

        let (tx, rx) = oneshot::channel();

        let start = Instant::now();

        let during_fut = future::poll_fn(|| {
            during(start.elapsed());
            rx.poll()
        });

        let (collated, collate_time) = pool.install(||
            rayon::join(
                move || poll_on_interval(interval, during_fut),
                move || {
                    let ret = time_span_ms(move || VpTree::from_vec(images));
                    tx.send(());
                    ret
                }
            ).1
        );

        CollatedResults {
            tree: collated,
            collate_time: collate_time,
            settings: results.settings,
            errors: results.errors,
        }
    }
}

fn send_ignore<T>(sender: &mpsc::Sender<T>, val: T) {

}

fn poll_on_interval<F>(interval: Option<Duration>, mut fut: F) where F: Future {
    use futures::Async::*;

    let mut spawn = executor::spawn(fut);
    let unpark = UnparkTimeout::new(interval);

    while let Ok(NotReady) = spawn.poll_future(unpark.clone()) {
        unpark.park();
    }
}

struct UnparkTimeout {
    thread: Thread,
    timeout: Option<Duration>,
}

impl UnparkTimeout {
    fn new(timeout: Option<Duration>) -> Arc<UnparkTimeout> {
        Arc::new(UnparkTimeout {
            thread: thread::current(),
            timeout: timeout,
        })
    }

    fn park(&self) {
        if let Some(dur) = self.timeout {
            thread::park_timeout(dur);
        } else {
            thread::park();
        }
    }
}

impl Unpark for UnparkTimeout {
    fn unpark(&self) {
        self.thread.unpark();
    }
}

pub fn worker(threads: Option<usize>) -> Worker {
    use rayon::Configuration;

    let config = if let Some(threads) = threads {
        Configuration::new().set_num_threads(threads)
    } else {
        Configuration::new()
    };

    Worker {
        pool: ThreadPool::new(config).expect("Error initializing thread pool"),
    }
}
