use ui::prelude::*;

use config::ProgramSettings;
use img::UniqueImage;
use processing::{mod, TimedImageResult, ProcessingError, Total};

use std::default::Default;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Relaxed};

use time::precise_time_ns;

pub fn start_processing(settings: ProgramSettings) -> (Total, Vec<UniqueImage>, Vec<ProcessingError>){	
	let (mut uic, mut gl, mut events) = create_window("img-dup running", [640, 480]);

    let paths = processing::find_images(&settings);

    let stop = Arc::new(AtomicBool::new(false));

    let mut buf: Buffers = Default::default();
    buf.total = paths.len();
    buf.slider_max = buf.total as f64;

    let start = precise_time_ns();
    let img_rx = processing::spawn_threads(&settings, paths);

    let status_rx = receive_images(img_rx, settings, stop.clone());
   		
	for event in events {
        buf.set_elapsed(precise_time_ns() - start);

        if buf.stop == 2 { stop.store(true, Relaxed); }

        match status_rx.try_recv() {
            Ok(Message::Update(status)) => buf.status_update(status),
            Ok(Message::Finished(result)) => { return result; },
            Err(_) => (),
        }

		uic.handle_event(&event);
		match event {
			Event::Render(args) => {
				gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
					draw_running_dialog(gl, &mut uic, &mut buf);				
				});
			}
			_ => (),
		}
	}

    unreachable!();
}

#[deriving(Default)]
struct Buffers {
    percent: String,
    avg_load: String,
    avg_hash: String,
    est_time_rem: String,
    count: String,
    elapsed: String,
    elapsed_ns: u64,
    total: uint,
    // stop == 0 -> continue
    // stop == 1 -> confirm
    // stop == 2 -> stop!
    stop: u8,
    slider_cur: f64,
    slider_max: f64,
}

impl Buffers {
    fn status_update(&mut self, status: Status) {
        self.clear_buffers();

        write_str!(self.avg_load, "Avg Load (ms): {}", ns_to_ms(status.avg_load));
        write_str!(self.avg_hash, "Avg Hash (ms): {}", ns_to_ms(status.avg_hash));

        self.set_est_time(status.count + status.errors);

        write_str!(self.count, 
            "Current (Errors) / Total: {} ({}) / {}", 
            status.count, status.errors, self.total
        );

        self.slider_cur = status.count as f64 + status.errors as f64;

        write_str!(self.percent, "{:.02}%", (self.slider_cur / self.slider_max) * 100.0);
    }

    fn set_est_time(&mut self, done: uint) {
        let est_secs = ns_to_secs(self.elapsed_ns / done as u64 * self.total as u64); 

        let (hr, min, sec) = secs_to_hr_min_sec(est_secs);

        write_str!(self.est_time_rem, "ETA: {}:{:02}:{:02}", hr, min, sec);
    }

    fn set_elapsed(&mut self, elapsed_ns: u64) {
        let elapsed_secs = ns_to_secs(elapsed_ns);

        if elapsed_secs == ns_to_secs(self.elapsed_ns) {
            // Not even a second has passed, don't update
            return; 
        }

        self.elapsed_ns = elapsed_ns;
        
        let (hr, min, sec) = secs_to_hr_min_sec(elapsed_secs);

        self.elapsed.clear();
        write_str!(self.elapsed, "Elapsed: {}:{:02}:{:02}", hr, min, sec); 
    }

    fn clear_buffers(&mut self) {
        self.avg_load.clear();
        self.avg_hash.clear();
        self.est_time_rem.clear();
        self.count.clear();    
        self.percent.clear();
    }
}

#[inline]
fn ns_to_secs(ns: u64) -> u64 {
    ns / 1_000_000_000
}

#[inline]
fn ns_to_ms(ns: u64) -> u64 {
    ns / 1_000_000
}

#[inline]
fn secs_to_hr_min_sec(total_secs: u64) -> (u64, u8, u8) {
    let secs = (total_secs % 60) as u8;
    let total_mins = total_secs / 60;
    let min = (total_mins % 60) as u8;
    let hrs = total_mins / 60;

    (hrs, min, secs)
}

#[deriving(Copy)]
struct Status {
    avg_load: u64,
    avg_hash: u64,
    count: uint,
    errors: uint,
}

pub type ProcResult = (Total, Vec<UniqueImage>, Vec<ProcessingError>);

enum Message {
    Update(Status),
    Finished(ProcResult),
}


fn draw_running_dialog(gl: &mut Gl, uic: &mut UiContext, buf: &mut Buffers) {
    background(gl, uic);

    const PROGRESS: u64 = 1;
    uic.slider(PROGRESS, buf.slider_cur, 0.0, buf.slider_max)
        .position(5.0, 5.0)
        .dimensions(470.0, 30.0)
        .draw(gl);

    uic.label(&*buf.percent)
        .right(5.0)
        .size(18)
        .draw(gl);

    uic.label(&*buf.count)
        .position(5.0, 35.0) 
        .size(18)
        .draw(gl);

    uic.label(&*buf.elapsed)
        .position(330.0, 35.0)
        .size(18)
        .draw(gl);
         
    uic.label(&*buf.avg_load)
        .position(5.0, 55.0)
        .size(18)
        .draw(gl);

    uic.label(&*buf.avg_hash)
        .position(170.0, 55.0)
        .size(18)
        .draw(gl);

    uic.label(&*buf.est_time_rem)
        .position(340.0, 55.0)
        .size(18)
        .draw(gl);

    const STOP: u64 = 2;
    uic.button(STOP)
        .position(485.0, 35.0)
        .dimensions(80.0, 30.0)
        .label(match buf.stop {
            0 => "Stop",
            1 => "Really?",
            2 => "Stopping",
            _ => unreachable!(),
        })
        .label_font_size(18)
        .callback(|| buf.stop += 1)
        .draw(gl);
}

fn receive_images(img_rx: Receiver<TimedImageResult>, settings: ProgramSettings, stop: Arc<AtomicBool>) 
-> Receiver<Message> {
    let (status_tx, status_rx) = channel();
    
    spawn(move |:| {        
        let mut unique_images = Vec::new();
        let mut errors = Vec::new();

        let mut total_load = 0u64;
        let mut total_hash = 0u64;
        let mut count = 0u64;

        for img_result in img_rx.iter() {
            if stop.load(Relaxed) { break; }

            match img_result {
                Ok((image, load, hash)) => {
                    processing::manage_images(&mut unique_images, image, &settings);
                    count += 1;
                    total_load += load;
                    total_hash += hash;
                },
                Err(img_err) => errors.push(img_err),
            }
            
            if status_tx.send_opt(Message::Update(Status {
                avg_load: total_load / count,
                avg_hash: total_hash / count,
                count: count as uint,
                errors: errors.len(),
            })).is_err() { return; };
        }
      
        status_tx.send(Message::Finished((count as uint, unique_images, errors)))
    });
    
    status_rx
}

