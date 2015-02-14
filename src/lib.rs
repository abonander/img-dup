extern crate "rustc-serialize" as serialize;
extern crate img_hash;

mod config;
mod img;
mod processing;

use img_hash::HashType;

use std::borrow::ToOwned;
use std::fs::{self, DirEntry, PathExt};
use std::io;
use std::path::{AsPath, Path, PathBuf};

pub static DEFAULT_EXTS: &[&str] = &["jpg", "png", "gif"];

/// A helper struct for searching for image files within a directory.
pub struct ImageSearch<'a> {
    /// The directory to search
    pub dir: &'a Path,
    /// If the search should be recursive (visit subdirectories)
    pub recursive: bool,
    /// The extensions to match.
    pub exts: Vec<&'a str>,
}

impl ImageSearch {
    /// Initiate a search builder with the base search directory.
    /// Starts with a copy of `DEFAULT_EXTS` for the list of file extensions,
    /// and `recursive` set to `false`.
    pub fn with_dir<P: AsPath>(dir: &P) -> ImageSearch<'a> {
        ImageSearch {
            dir: dir.as_path(),
            recurse: false,
            exts: DEFAULT_EXTS.to_owned(),
        }
    }

    /// Set the `recursive` flag on `self`,
    /// returns `self` for method chaining
    pub fn recursive(mut self, recursive: bool) -> ImageSearch<'a> {
        self.recursive = recursive;
        self
    }

    /// Add an extension to the list on `self`, 
    /// returning `self` for method chaining
    pub fn ext(mut self, ext: &str) -> ImageSearch<'a> {
        self.exts.push(ext);
        self
    }

    /// Add all the extensions from `exts` to `self,
    /// returning `self` for method chaining
    pub fn exts(mut self, exts: &[&str]) -> ImageSearch<'a> {
        self.exts.push_all(exts.iter());
        self
    }

    /// Search `self.dir` for images with extensions matching one in `self.exts`,
    /// recursing into subdirectories if `self.recursive` is set to `true`.
    ///
    /// Any I/O errors during searching are safely filtered out.
    pub fn search(mut self) -> Vec<PathBuf> {
        /// Generic to permit code reuse
        fn do_filter<I: Iterator<Item=io::Result<DirEntry>>(mut iter: I, exts: &[&str])
        -> Vec<PathBuf> {
            iter.filter_map(Result::ok)
                .map(DirEntry::path)
                .filter(|path|
                    path.extension()
                        .map(|ext| exts.contains(ext))
                        .unwrap_or(false)
                )
                .collect()
        }

        match self.recursive {
            false => do_filter(try!(fs::read_dir(self.dir))),
            true => do_filter(try!(fs::walk_dir(self.dir))),
        }
    }
}


pub struct SessionBuilder {
    pub images: Vec<PathBuf>,
    pub threads: uint,
    pub hash_size: u32,
    pub hash_type: 
    pub threshold: f32,
    pub limit: uint,
}
