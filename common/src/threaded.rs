use img::{Image, ImgResults, HashSettings};

use std::mem;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::vec;

pub struct ThreadedSession {
	queue: Arc<HashQueue>,
}

impl ThreadedSession {
	pub fn process_multithread(
		threads: Option<usize>, 
		settings: HashSettings, 
		images: Vec<PathBuf>
	) -> ThreadedSession {
		let threads = threads.unwrap_or_else(::num_cpus::get);
		assert!(threads > 0, "`threads` parameter must be nonzero if provided!");

		let queue = Arc::new(HashQueue::from_vec(images, settings));

        for _ in 0..threads {
            let loc_queue = queue.clone();
    
            thread::spawn(move|| loc_queue.hash_all());
        }

		ThreadedSession {
			queue: queue,
		}
	}
	
	pub fn wait_for_update(&self) -> RunningStatus {
		self.queue.wait_for_update()	
    }

    pub fn is_done(&self) -> bool {
        self.queue.is_done()
    }

    pub fn wait(self) -> ImgResults {
        while !self.queue.is_done() {
            let _ = self.queue.wait_for_update();
        }
        
        self.queue.take_out() 
    }
}

struct HashQueue {
    in_: Mutex<vec::IntoIter<PathBuf>>,
    out: Mutex<ImgResults>,
    total: usize,
    cvar: Condvar,
	settings: HashSettings,
}

impl HashQueue {
    pub fn from_vec(vec: Vec<PathBuf>, settings: HashSettings) -> HashQueue {
        let total = vec.len();

        HashQueue {
            in_: Mutex::new(vec.into_iter()),
            out: Mutex::new(ImgResults::empty()),
            total: total,
            cvar: Condvar::new(),
			settings: settings,
        }
    }

    fn next_in(&self) -> Option<PathBuf> {
        self.in_.lock().unwrap().next()
    } 

	fn hash_all(&self) {
		while let Some(path) = self.next_in() {
            let result = Image::load_and_hash(path, self.settings);
            self.out.lock().unwrap().push_result(result);
            self.cvar.notify_all();
		}
    }

    fn is_done(&self) -> bool {
        self.out.lock().unwrap().total() == self.total - 1 
    }
    
    fn take_out(&self) -> ImgResults {
        let mut lock = self.out.lock().unwrap();
        mem::replace(&mut *lock, ImgResults::empty())
    }

    fn wait_for_update(&self) -> RunningStatus {
        let (out, _) = self.cvar.wait_timeout_ms(self.out.lock().unwrap(), 1000).unwrap();
        RunningStatus::from_img_results(&*out)
    }    
}

#[derive(Copy, Clone)]
pub struct RunningStatus {
    /// The number of images completed, including errors.
    pub done: usize,
    /// The number of images that have failed to load or hash.
    pub errors: usize,
}

impl RunningStatus {
    fn from_img_results(results: &ImgResults) -> RunningStatus {
        let errors = results.errors.len();
        let done = results.images.len() + errors;

        RunningStatus { done: done, errors: errors }
    }
}

