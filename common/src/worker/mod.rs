use image::ImageResult;

use img::{Image, HashSettings};

use std::path::PathBuf;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

#[cfg(windows)]
#[path = "win32.rs"]
mod imp;

pub struct WorkManager {
   rx: Receiver<ImageResult<Image>>,
   imp: imp::WorkManager,
}

impl WorkManager {
    pub fn start_threads(thread_count: usize, hash_cfg: HashSettings) -> WorkManager {
        let (res_tx, res_rx) = mpsc::channel();
        let manager = imp::WorkManager::new(thread_count);

        for _ in (0 .. thread_count) {
            let worker = manager.worker(&res_tx, hash_cfg);        

            thread::spawn(move || worker.work());
        }

        WorkManager {
            rx: res_rx,
            imp: manager,
        }
    }

    pub fn enqueue(&self, path: PathBuf) {
        self.imp.send_msg(Message::Load(path));
    }

    pub fn finish(&self) {
        self.imp.send_msg(Message::Quit);
    }

    pub fn rx(&self) -> &Receiver<ImageResult<Image>> {
        &self.rx
    }
}

enum Message {
    Load(PathBuf),
    Loaded(PathBuf, Vec<u8>),
    Quit,
}

impl Message {
    fn clone_quit(&self) -> Option<Self> {
        match *self {
            Message::Quit => Some(Message::Quit),
            _ => None,
        }            
    }
}

