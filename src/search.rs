use std::borrow::ToOwned;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

impl<'a> ::Settings<'a> {
    /// Search `self.dir` for images with extensions contained in `self.exts`,
    /// recursing into subdirectories if `self.recursive` is set to `true`.
    pub fn search<F, Fe>(&self, mut with_path: F, mut with_err: Fe)
    where F: FnMut(PathBuf), Fe: FnMut(WalkDirErr) -> bool {
        walk_dir(self.dir, self.recursive, self.exts, &mut with_path, &mut with_err)
    }
}

macro_rules! map_path (
    ($path:expr; $try:expr) => {
        match $try {
            Ok(val) => Ok(val),
            Err(e) => Err(WalkDirErr {
                path: $path.into(),
                e: e,
            })
        }
    };
);

fn walk_dir<F, Fe>(path: &Path, recurse: bool, exts: &[&str], with_path: &mut F, with_err: &mut Fe)
where F: FnMut(PathBuf), Fe: FnMut(WalkDirErr) -> bool {
    macro_rules! try_with_path (
        ($path:expr; $res:expr) => (
            match map_path!($path; $res) {
                Ok(val) => val,
                Err(e) => if with_err(e) {
                    continue;
                } else {
                    return;
                }
            }
        )
    );

    let iter = match map_path!(path; fs::read_dir()) {
        Ok(iter) => iter,
        Err(e) => { with_err(e); return },
    };

    for res in iter {
        let entry = try_with_path!(path; res);
        let entry_path = entry.path();
        let ftype = try_with_path!(entry_path; entry.file_type());

        if ftype.is_dir() {
            if recurse {
                walk_dir(&entry_path, recurse, exts, with_path, with_err)?;
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

pub struct WalkDirErr {
    path: PathBuf,
    err: io::Error,
}
