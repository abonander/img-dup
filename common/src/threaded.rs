use img::{ImgResults, HashSettings};

use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
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
			queue: queue,
		}
	}
	
	pub fn status(&self) -> &RunningStatus {
		&self.queue.status	
    }

    pub fn wait(self) -> ImgResults {
        ImgResults::from_statuses(self.queue.wait())
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
        let vec: Vec<_> = vec.into_iter().map(|path| ImgStatus::Unhashed(path)).collect();
        let end = vec.len();

        HashQueue {
            vec: vec,
            curr: AtomicUsize::new(0),
			status: RunningStatus::new(end),
			settings: settings,
        }
    }

    fn next(&self) -> Option<&mut ImgStatus> {
        let idx = self.curr.fetch_add(1, Relaxed);
        self.vec.get(idx).map(|img| 
			unsafe { &mut *(img as *const ImgStatus as *mut ImgStatus) }
		)
    }

	fn hash_all(&self) {
        let mut guard = PanicGuard { status: &self.status, active: true };

		while let Some(status) = self.next() {
			status.hash(self.settings);
			self.status.add_done(status.is_err());
		}

        guard.active = false;
	}

    fn wait(&self) -> Vec<ImgStatus> {
        while !self.status.is_done() {
            self.status.wait_for_update();
        }

        self.vec.clone()
    }
}

struct PanicGuard<'a> {
    status: &'a RunningStatus,
    active: bool,
}

impl<'a> Drop for PanicGuard<'a> {
    fn drop(&mut self) {
        if self.active {
            self.status.add_done(true);
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
    end: usize,
    mutex: Mutex<()>,
    cvar: Condvar,
}

impl RunningStatus {
    fn new(end: usize) -> Self {
        RunningStatus {
            done: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            end: end,
            mutex: Mutex::new(()),
            cvar: Condvar::new(),
        }
    }

    pub fn done(&self) -> usize {
        self.done.load(Relaxed)
    }

    pub fn errors(&self) -> usize {
        self.errors.load(Relaxed)
    }

    pub fn wait_for_update(&self) {
        use std::time::Duration;
        let _ = self.cvar.wait_timeout(
            self.mutex.lock().unwrap(),
            Duration::seconds(1)
        );
    }

    pub fn is_done(&self) -> bool {
        self.done() == self.end
    }

    fn add_done(&self, was_error: bool) {
        self.done.fetch_add(1, Relaxed);

        if was_error {
            self.errors.fetch_add(1, Relaxed);
        }

        self.cvar.notify_all();
    }
}
