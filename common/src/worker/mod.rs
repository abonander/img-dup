use image::ImageError;

use img::{Image, HashSettings};

use std::fmt;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

macro_rules! try_with_path(
    ($path:ident; $expr:expr) => (
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(::worker::HashError::path_and_err($path, err)),
        }
    )
);
#[cfg(windows)]
#[path = "win32.rs"]
mod imp;

#[cfg(unix)]
#[path = "unix.rs"]
mod imp;

pub type HashResult = Result<Image, HashError>;

pub struct WorkManager {
   rx: Receiver<HashResult>,
   imp: imp::WorkManager,
}

impl WorkManager {
    pub fn start_threads(thread_count: usize, hash_cfg: HashSettings) -> WorkManager {
        let (res_tx, res_rx) = mpsc::channel();
        let manager = imp::WorkManager::new(thread_count);

        for _ in 0 .. thread_count {
            let worker = manager.worker(&res_tx, hash_cfg);        

            thread::spawn(move || worker.work());
        }

        WorkManager {
            rx: res_rx,
            imp: manager,
        }
    }

    pub fn enqueue(&self, path: PathBuf) {
        self.imp.enqueue_load(path);
    }

    pub fn finish(&self) {
        self.imp.quit();
    }

    pub fn rx(&self) -> &Receiver<HashResult> {
        &self.rx
    }
}

#[derive(Debug)]
pub struct HashError {
    pub path: PathBuf,
    pub err: ImageError,
}

impl HashError {
    fn path_and_err<E: Into<ImageError>>(path: PathBuf, err: E) -> Self {
        HashError {
            path: path,
            err: err.into(),
        }
    }
}

impl Error for HashError {
    fn description(&self) -> &str {
        self.err.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.err.cause()
    }
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Failed to hash image at `{}`, reason: {}",
            self.path.display(),
            self.err
        ))
    }
}
