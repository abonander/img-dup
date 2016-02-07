extern crate crossbeam;
extern crate libc;

use self::crossbeam::sync::MsQueue;

use image;

use img::{Image, HashSettings};

use super::{HashResult, HashError};

use std::fs::File;
use std::io::prelude::*;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::io;

enum Message {
    Load(PathBuf),
    Advised(PathBuf, File),
    Quit,
}

pub struct WorkManager {
   queue: Arc<MsQueue<Message>>,
}

impl WorkManager {
    pub fn new(_: usize) -> Self {
        WorkManager {
            queue: Arc::new(MsQueue::new()),
        }
    }

    pub fn enqueue_load(&self, path: PathBuf) {
        self.queue.push(Message::Load(path)); 
    }

    pub fn quit(&self) {
        self.queue.push(Message::Quit);
    }

    pub fn worker(&self, tx: &Sender<HashResult>, hash_cfg: HashSettings) -> Worker {
       Worker {
           queue: self.queue.clone(),
           tx: tx.clone(),
           hash_cfg: hash_cfg
        }
    }
}

pub struct Worker {
    queue: Arc<MsQueue<Message>>,
    tx: Sender<HashResult>,
    hash_cfg: HashSettings,
}

impl Worker {
    pub fn work(self) {
        use self::Message::*;

        loop {
            match self.queue.pop() {
                Load(path) => if let Err(err) = self.advise(path) {
                    self.tx.send(Err(err)).unwrap();
                },
                Advised(path, file) => self.tx.send(self.load(path, file)).unwrap(),
                Quit => {
                    self.queue.push(Message::Quit);
                    break;
                }
            }
        }
    }

    fn advise(&self, path: PathBuf) -> Result<(), HashError>  {
        let file = try_with_path!(path; File::open(&path));
        
        try_with_path!(path; advise_willneed(&file));

        self.queue.push(Message::Advised(path, file)); 

        Ok(())
    }

    fn load(&self, path: PathBuf, mut file: File) -> HashResult {
        let mut data = Vec::new();

        try_with_path!(path; file.read_to_end(&mut data));

        let len = data.len();

        let img = try_with_path!(path; image::load_from_memory(&data));
        Ok(Image::hash(path, img, self.hash_cfg, len as u64))
    } 
}

fn advise_willneed(file: &File) -> io::Result<()> {
    let len = try!(file.metadata()).len();
    
    let res = unsafe {
        self::libc::posix_fadvise(
            file.as_raw_fd(),
            0, len as self::libc::off_t,
            self::libc::POSIX_FADV_WILLNEED
        )
    };

    if res != 0 {
        Err(io::Error::from_raw_os_error(res))
    } else {
        Ok(())
    }
}
