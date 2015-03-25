pub static DEFAULT_EXTS: &'static [&'static str] = &["jpg", "png", "gif"];

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
    pub fn with_dir<P: AsPath>(dir: &'a P) -> ImageSearch<'a> {
        ImageSearch {
            dir: dir.as_path(),
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
        self.exts.push_all(exts);
        self
    }

    /// Searche `self.dir` for images with extensions contained in `self.exts`,
    /// recursing into subdirectories if `self.recursive` is set to `true`.
    ///
    /// Returns a vector of all found images as paths.
    ///
    /// Any I/O errors during searching are safely filtered out.
    pub fn search(self) -> io::Result<Vec<PathBuf>> {
        /// Generic to permit code reuse
        fn do_filter<I: Iterator<Item=io::Result<DirEntry>>>(iter: I, exts: &[&str]) -> Vec<PathBuf> {
                iter.filter_map(|res| res.ok())
                    .map(|entry| entry.path())
                    .filter(|path|
                        path.extension()
                            .and_then(|s| s.to_str())
                            .map(|ext| exts.contains(&ext))
                            .unwrap_or(false)
                    )
                    .collect()
        }

        // `match` instead of `if` for clarity
        let paths = match self.recursive {
            false => do_filter(try!(fs::read_dir(self.dir)), &self.exts),
            true => do_filter(try!(fs::walk_dir(self.dir)), &self.exts),
        };

        Ok(paths)
    }
}
