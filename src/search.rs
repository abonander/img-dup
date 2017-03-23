use std::convert::AsRef;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SearchSettings<'a> {
    pub dir: &'a Path,
    pub recursive: bool,
    pub exts: Vec<&'a str>,
}

impl<'a> SearchSettings<'a> {
    /// Search `self.dir` for images with extensions contained in `self.exts`,
    /// recursing into subdirectories if `self.recursive` is set to `true`.
    pub fn search<F, Fe>(&self, mut with_path: F, mut with_err: Fe)
    where F: FnMut(PathBuf), Fe: FnMut(::Error) -> bool {
        walk_dir(self.dir, self.recursive, &self.exts, &mut with_path, &mut with_err)
    }
}

impl Default for SearchSettings<'static> {
    fn default() -> Self {
        SearchSettings {
            dir: "./".as_ref(),
            recursive: false,
            exts: vec!["jpg", "png", "gif"],
        }
    }
}

macro_rules! map_path (
    ($path:expr; $try:expr) => {
        match $try {
            Ok(val) => Ok(val),
            Err(e) => Err(::Error {
                path: $path.into(),
                error: e.into(),
            }),
        }
    };
);

fn walk_dir<F, Fe>(path: &Path, recurse: bool, exts: &[&str], with_path: &mut F, with_err: &mut Fe)
where F: FnMut(PathBuf), Fe: FnMut(::Error) -> bool {
    macro_rules! try_with_path (
        ($path:expr; $res:expr) => (
            match $res {
                Ok(val) => val,
                Err(e) => {
                    let e = ::Error {
                        path: $path.into(),
                        error: e.into(),
                    };

                    if with_err(e) {
                        continue;
                    } else {
                    return;
                    }
                }
            }
        )
    );

    let iter = match map_path!(path; fs::read_dir(&path)) {
        Ok(iter) => iter,
        Err(e) => { with_err(e); return },
    };

    for res in iter {
        let entry = try_with_path!(path; res);
        let entry_path = entry.path();
        let ftype = try_with_path!(entry_path; entry.file_type());

        if ftype.is_dir() {
            if recurse {
                walk_dir(&entry_path, recurse, exts, with_path, with_err);
            }

            continue;
        }

        if let Some(ext) = entry_path.extension() {
            // Can't push the path if it's borrowed so we push if this falls through
            if !exts.iter().any(|ext_ | ext == AsRef::<OsStr>::as_ref(ext_)) { continue }
        }

        with_path(entry_path)
    }
}
