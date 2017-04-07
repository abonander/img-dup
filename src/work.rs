use futures::sync::{mpsc, oneshot};
use futures::executor::{self, Unpark, Spawn};
use futures::{future, Async, AsyncSink, Future, Stream};

use image::{self, DynamicImage, GenericImage};

use rayon::{self, ThreadPool};
use rayon::prelude::*;

use vp_tree::VpTree;
use vp_tree::dist::KnownDist;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{BufReader, BufRead};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
    fn hash(self, settings: HashSettings) -> HashedImage {
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

type Sender = mpsc::Sender<WorkResult>;

struct CollectWork {
    settings: HashSettings,
    unpark: Arc<Unpark>,
    sender: Spawn<mpsc::Sender<WorkResult>>,
    overflow: Option<WorkResult>,
    closed: bool,
}

impl CollectWork {
    fn new(sender: mpsc::Sender<WorkResult>, settings: HashSettings) -> Self {
        CollectWork {
            settings: settings,
            // We'll be polling again shortly, no need to know
            unpark: Arc::new(IgnoreUnpark),
            sender: executor::spawn(sender),
            overflow: None,
            closed: false,
        }
    }

    fn load_and_hash(mut self, path: PathBuf) -> Self {
        if self.closed { return self; }

        let res = load_image(path).map(|image| image.hash(self.settings));

        if let Some(overflow) = self.overflow.take() {
            self.wait_send(overflow);
        }

        if let AsyncSink::NotReady(overflow) = self.try_send(res) {
            self.overflow = Some(overflow);
        }

        self
    }

    fn try_send(&mut self, res: WorkResult) -> AsyncSink<WorkResult> {
        match self.sender.start_send(res, &self.unpark) {
            Ok(res) => res,
            Err(_) => {
                self.closed = true;
                AsyncSink::Ready
            },
        }
    }

    fn wait_send(&mut self, res: WorkResult) {
        if let Err(_) = self.sender.wait_send(res) {
            self.closed = true;
        }
    }
}

struct IgnoreUnpark;

impl Unpark for IgnoreUnpark {
    fn unpark(&self) {}
}

pub type WorkResult = Result<HashedImage, ::Error>;

pub struct Worker {
    pool: ThreadPool,
}

impl Worker {
    fn join_pool<Fl, Fr, Rl, Rr>(&self, left: Fl, right: Fr) -> (Rl, Rr)
    where Fl: FnOnce() -> Rl + Send, Fr: FnOnce() -> Rr + Send, Rl: Send, Rr: Send {
        self.pool.install(
            || rayon::join(left, right)
        )
    }

    pub fn load_and_hash<F>(self, paths: Vec<PathBuf>, settings: HashSettings, mut collect: F)
    where F: FnMut(WorkResult) + Send {
        let threads = self.pool.current_num_threads();

        // Use a capacity of log2(paths.len())
        let (tx, rx) = mpsc::channel(paths.len().leading_zeros() as usize);

        let during_fut = rx.for_each(|result| {
            collect(result);
            Ok(())
        });

        self.join_pool(
            || poll_on_interval(Duration::from_secs(1), during_fut),
            move || {
                use rayon::iter::IndexedParallelIterator;

                // Precompute DCT matrix on every thread
                (0..threads).into_par_iter().with_max_len(1)
                    .for_each(|_| settings.prepare());

                paths.into_par_iter().fold(|| CollectWork::new(tx.clone(), settings),
                                           CollectWork::load_and_hash)
                    // Kill the workers early if the queue is closed.
                    .any(|collect| collect.closed)
            }
        );
    }

    pub fn collate<F, T: KnownDist + Send>(self, images: Vec<T>, interval: Duration, mut during: F)
    -> Collated<T> where F: FnMut() + Send, T::DistFn: Send {
        let (tx, mut rx) = oneshot::channel();

        let during_fut = future::poll_fn(|| {
            during();
            rx.poll()
        });

        let (collated, collate_time) = self.join_pool(
            move || poll_on_interval(interval, during_fut),
            move || {
                let ret = time_span_ms(move || VpTree::from_vec(images));
                let _ = tx.send(());
                ret
            }
        ).1;

        Collated {
            tree: collated,
            time: collate_time,
        }
    }
}

pub struct Collated<T: KnownDist> {
    pub tree: VpTree<T, T::DistFn>,
    pub time: u64,
}

fn poll_on_interval<F>(interval: Duration, fut: F) where F: Future {
    let unpark = ThreadUnpark::new_arc();

    let mut spawn = executor::spawn(fut);

    while let Ok(Async::NotReady) = spawn.poll_future(unpark.clone()) {
        unpark.park_timeout(interval);
    }
}

struct ThreadUnpark {
    thread: thread::Thread,
    ready: AtomicBool,
}

impl ThreadUnpark {
    fn new_arc() -> Arc<ThreadUnpark> {
        Arc::new(ThreadUnpark {
            thread: thread::current(),
            ready: AtomicBool::new(false),
        })
    }

    fn park_timeout(&self, timeout: Duration) {
        if !self.ready.swap(false, Ordering::Acquire) {
            thread::park_timeout(timeout);
        }
    }
}

impl Unpark for ThreadUnpark {
    fn unpark(&self) {
        self.ready.store(true, Ordering::Release);
        self.thread.unpark();
    }
}

pub fn worker(threads: Option<usize>) -> Worker {
    use rayon::Configuration;

    let config = if let Some(threads) = threads {
        Configuration::new().num_threads(threads)
    } else {
        Configuration::new()
    };

    Worker {
        pool: ThreadPool::new(config).expect("Error initializing thread pool"),
    }
}
