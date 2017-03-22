//! As a library crate, `img_dup` provides tools for searching for images, hashing them in
//! parallel, and collating their hashes to find near or complete duplicates.

extern crate futures;
extern crate img_hash;
extern crate image;
extern crate rayon;
extern crate vec_vp_tree as vp_tree;

#[macro_use]
extern crate serde_derive;

pub mod model;
pub mod hash;
pub mod search;

mod work;

use futures::IntoFuture;
use futures::executor;

use hash::HashSettings;
use model::{Image, HashedImage};

use work::LoadedImage;

use search::SearchSettings;

use image::ImageError;

use rayon::ThreadPool;

use rayon::par_iter::from_par_iter::FromParallelIterator;
use rayon::par_iter::{IntoParallelIterator, ParallelIterator};

use std::path::{Path, PathBuf};

pub struct Error {
    pub path: PathBuf,
    pub error: ImageError,
}

#[derive(Default)]
pub struct WorkResults {
    pub success: Vec<HashedImage>,
    pub error: Vec<Error>,
}

pub type WorkResult = Result<HashedImage, Error>;

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
    pub fn load_and_hash<Fd, Fl, Fh, Fe>(self, settings: HashSettings, during: Fd, on_load: Fl,
                                         on_hash: Fh, on_error: Fe) -> WorkResults
    where Fd: IntoFuture<Item = (), Error = ()> + Send, Fl: Fn(&Image) + Send + Sync,
          Fh: Fn(&HashedImage) + Send + Sync, Fe: Fn(&Error) + Send + Sync
    {
        let pool = self.pool;
        let threads = pool.num_threads();
        let paths = self.paths;

        pool.install(||
            rayon::join(
                || executor::spawn(during.into_future()).wait_future(),
                move || {
                    // Precompute DCT matrix on every thread
                    (0 .. threads).into_par_iter().weight_max()
                        .for_each(|_| settings.prepare());

                    paths.into_par_iter().map(|path|
                        match work::load(path) {
                            Ok(image) => {
                                on_load(&image.image);
                                let hashed = image.hash(&settings);
                                on_hash(&hashed);
                                Ok(hashed)
                            },
                            Err(e) => {
                                on_error(&e);
                                Err(e)
                            }
                        }
                    ).collect()
                }
            ).1
        )
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
