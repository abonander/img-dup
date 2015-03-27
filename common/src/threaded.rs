use img::HashSettings;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;

use img::ImgStatus;

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
		let move_queue = queue.clone();

		RestartableJob::spawn(threads, move || move_queue.hash_all());

		ThreadedSession {
			queue: queue
		}
	}
	
	pub fn status(&self) -> &RunningStatus {
		&self.queue.status	
	}	
}

struct HashQueue {
    vec: Vec<ImgStatus>,
    curr: AtomicUsize,
	status: RunningStatus,
	settings: HashSettings,
}

impl HashQueue {
    pub fn from_vec(vec: Vec<PathBuf>, settings: HashSettings) -> HashQueue {
        HashQueue {
            vec: vec.into_iter().map(|path| ImgStatus::Unhashed(path)).collect(),
            curr: AtomicUsize::new(0),
			status: RunningStatus::new(),
			settings: settings,
        }
    }

    fn next(&self) -> Option<&mut ImgStatus> {
        let idx = self.curr.fetch_add(1, Relaxed);
        self.vec.get(idx).map(|img| 
			unsafe { &mut *(img as *const ImgStatus as *mut ImgStatus) }
		)
    }

	pub fn hash_all(&self) {
		while let Some(status) = self.next() {
			status.hash(self.settings);
			
			if status.is_err() {
				self.status.add_error();
			}
						
			self.status.add_done();	
		}
	}
}

type JobFn = Box<Fn() + Send + Sync + 'static>;

#[derive(Clone)]
struct RestartableJob {
	job: Arc<JobFn>,
}

impl RestartableJob {
	fn spawn<F>(threads: usize, job: F) where F: Fn() + Send + Sync + 'static {
		let job = RestartableJob {
			job: Arc::new(Box::new(job)),
		};

		for _ in 0 .. threads {
			let move_job = job.clone();
			thread::spawn(move || move_job.execute());
		}
	}
	
	fn execute(self) {
		(self.job)()	
	} 	
}

impl Drop for RestartableJob {
	fn drop(&mut self) {
		if thread::panicking() {
			let move_self = self.clone();
			thread::spawn(move || move_self.execute());
		}
	}	
}

pub struct RunningStatus {
    done: AtomicUsize,
    errors: AtomicUsize,
}

impl RunningStatus {
    fn new() -> Self {
        RunningStatus {
            done: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
        }
    }

    pub fn done(&self) -> usize {
        self.done.load(Relaxed)
    }

    pub fn errors(&self) -> usize {
        self.errors.load(Relaxed)
    }

    pub fn total(&self) -> usize {
        self.done() + self.errors()
    }

    fn add_done(&self) {
        self.done.fetch_add(1, Relaxed);
    }

    fn add_error(&self) {
        self.errors.fetch_add(1, Relaxed);
    }
}
