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
pub mod serialize;
pub mod work;

use futures::IntoFuture;
use futures::executor;

use hash::HashSettings;
use model::{Image, HashedImage};

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
