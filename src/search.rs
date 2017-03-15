use std::borrow::ToOwned;
use std::convert::AsRef;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

pub const DEFAULT_EXTS: &'static [&'static str] = &["jpg", "png", "gif"];

/// A helper struct for searching for image files within a directory.
pub struct ImageSearch<'a> {
    /// The directory to search
    pub dir: &'a Path,
    /// If the search should be recursive (visit subdirectories)
    pub recursive: bool,
    /// The extensions to match.
    pub exts: Vec<&'a str>,
}

impl<'a> ImageSearch<'a> {
    /// Initiate a search builder with the base search directory.
    /// Starts with a copy of `DEFAULT_EXTS` for the list of file extensions,
    /// and `recursive` set to `false`.
    pub fn with_dir<P: AsRef<Path>>(dir: &'a P) -> ImageSearch<'a> {
        ImageSearch {
            dir: dir.as_ref(),
            recursive: false,
            exts: DEFAULT_EXTS.to_owned(),
        }
    }

    pub fn recursive(&mut self, recursive: bool) -> &mut ImageSearch<'a> {
        self.recursive = recursive;
        self
    }

    /// Add an extension to the list on `self`,
    /// returning `self` for method chaining
    pub fn ext(&mut self, ext: &'a str) -> &mut ImageSearch<'a> {
        self.exts.push(ext);
        self
    }

    /// Add all the extensions from `exts` to `self,
    /// returning `self` for method chaining
    pub fn exts(&mut self, exts: &[&'a str]) -> &mut ImageSearch<'a> {
        self.exts.extend(exts);
        self
    }

    /// Searche `self.dir` for images with extensions contained in `self.exts`,
    /// recursing into subdirectories if `self.recursive` is set to `true`.
    ///
    /// Returns a vector of all found images as paths.
    pub fn search(&self, out: &mut Vec<PathBuf>) -> Result<(), WalkDirErr> {
        walk_dir(self.dir, self.recursive, &self.exts, out)
    }
}

macro_rules! try_with_path {
    ($path:expr; $try:expr) => {
        match $try {
            Ok(val) => val,
            Err(e) => return Err(WalkDirErr{
                path: $path.into(),
                err: e,
            }),
        }
    };
}

fn walk_dir(path: &Path, recurse: bool, exts: &[&str], out: &mut Vec<PathBuf>) -> Result<(), WalkDirErr> {
    for res in try_with_path!(path; fs::read_dir(path)) {
        let entry = try_with_path!(path; fs::read_dir(path));
        let ftype = try_with_path!(entry.path(); entry.file_type());

        if ftype.is_dir() && recurse {
            let path = entry.path();
            try_with_path!(path; walk_dir(&path, recurse, exts, out));
        } else if let Some(ext) = entry.extension() {
            if exts.contains(ext) { out.push(path); }
        }
    }

    Ok(())
}

pub struct WalkDirErr {
    path: PathBuf,
    err: io::Error,
}