use image::{
    mod, 
    ImageResult,
    RgbaImage,
};

use std::collections::{HashSet, HashMap};
use std::fmt::{mod, Show, Formatter};
use std::thread::Thread;


pub fn print_err<T, E>(result: Result<T, E>) where E: Show {
    if let Err(err) = result {
        println!("Encountered nonfatal error: {}", err);    
    }
}

#[deriving(Copy)]
pub struct FormatBytes(pub u64);

impl FormatBytes { 
    #[inline]
    fn to_kb(self) -> f64 {
        (self.0 as f64) / 1.0e3   
    }

    #[inline]
    fn to_mb(self) -> f64 {
        (self.0 as f64) / 1.0e6
    }

    #[inline]
    fn to_gb(self) -> f64 {
        (self.0 as f64) / 1.0e9
    }
}

impl Show for FormatBytes {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self.0 {
            0 ... 999 => fmt.write_fmt(format_args!("{} B", self.0)),
            1_000 ... 999_999 => fmt.write_fmt(format_args!("{:.02} KB", self.to_kb())),
            1_000_000 ... 999_999_999 => fmt.write_fmt(format_args!("{:.02} MB", self.to_mb())),
            _ => fmt.write_fmt(format_args!("{:.02} GB", self.to_gb())),
        }
    }        
}

pub struct ImgLoader {
    waiting: HashSet<Path>,
    ready: HashMap<Path, ImageResult<RgbaImage>>,
    in_tx: Sender<Path>,
    out_rx: Receiver<(Path, ImageResult<RgbaImage>)>,       
}

impl ImgLoader {
    pub fn new() -> ImgLoader {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = channel();

        Thread::spawn(move || {
            for path in in_rx.iter() {
                let result = image::open(&path).map(|img| img.to_rgba());
                out_tx.send((path, result));
            }
        }).detach();
        
        ImgLoader {
            waiting: HashSet::new(),
            ready: HashMap::new(),
            in_tx: in_tx,
            out_rx: out_rx,
        }        
    }
    
    pub fn begin_load(&mut self, path: &Path) {
        self.waiting.insert(path.clone());
        self.in_tx.send(path.clone());            
    }

    fn rollup_results(&mut self) {
        while let Ok((path, result)) = self.out_rx.try_recv() {
            self.waiting.remove(&path);
            self.ready.insert(path, result);
        }    
    }

    pub fn get_result(&mut self, path: &Path) -> ImageResult<RgbaImage> {
        if !self.waiting.contains(path) {
            self.begin_load(path);    
        }
         
        loop {
            self.rollup_results();

            if let Some(ready) = self.ready.remove(path) {
                return ready;
            }
        }
    }
}
