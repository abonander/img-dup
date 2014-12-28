use ui::prelude::*;

use config::ProgramSettings;

use std::default::Default;

pub fn show_setup_ui(settings: ProgramSettings) -> Option<ProgramSettings> {	
	let (mut state, mut buf) = ConfigState::from_settings(settings);

	let (mut uic, mut gl, mut events) = create_window("img-dup configuration", [740, 120]);
		
	for event in events {
        if state.canceled { break; }
        else if state.confirmed { return Some(state.settings); }

		uic.handle_event(&event);
		match event {
			Event::Render(args) => {
				gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
					draw_setup_dialog(gl, &mut uic, &mut state, &mut buf);				
				});
			}
			_ => (),
		}
	}

    None
}

struct ConfigState {
	settings: ProgramSettings,
    canceled: bool,
    confirmed: bool
}

impl ConfigState {
	fn from_settings(settings: ProgramSettings) -> (ConfigState, Buffers) {
		let mut buffers: Buffers = Default::default();

		write_str!(buffers.threads, "{}", settings.threads);
		write_str!(buffers.dir, "{}", settings.dir.display());
        write_str!(buffers.hash_size, "{}", settings.hash_size);
        buffers.set_threshold(settings.threshold); 

		(
            ConfigState {
			    settings: settings,
                canceled: false,
                confirmed: false,
		    }, 
            buffers
        )
	}

	fn set_threads(&mut self, threads: &mut String) {
		if let Some(threads) = threads.parse() {
			self.settings.threads = threads;
		} else {
			threads.clear();
			write_str!(threads, "{}", self.settings.threads);	
		}
	}

	fn update_dir(&mut self, dir: &mut String) {	
		self.settings.dir = Path::new(&**dir);
	}

    fn set_dir(&mut self, buf: &mut Buffers, dir: Path) {
        buf.dir.clear();
        write_str!(buf.dir, "{}", dir.display());
        self.settings.dir = dir;
    }

    fn add_threads(&mut self, buf: &mut Buffers, threads: int) {
        let new_threads = (self.settings.threads as int) + threads;

        if new_threads < 1 { return; }

        self.settings.threads = new_threads as uint;
        buf.threads.clear();
        write_str!(buf.threads, "{}", self.settings.threads); 
    }
    
    fn set_hash_size(&mut self, hash_size: &mut String) {
		if let Some(hash_size) = hash_size.parse() {
			self.settings.hash_size = hash_size;
		} else {
			hash_size.clear();
			write_str!(hash_size, "{}", self.settings.hash_size);	
		}
	}

    fn add_hash_size(&mut self, buf: &mut Buffers, hash_size: int) {
        let new_hash_size = (self.settings.hash_size as int) + hash_size;
        
        if new_hash_size < 1 { return; }
        
        self.settings.hash_size = new_hash_size as u32;
        buf.hash_size.clear();
        write_str!(buf.hash_size, "{}", self.settings.hash_size);
    }

    fn set_threshold(&mut self, buf: &mut Buffers, threshold: f32) {
        self.settings.threshold = threshold;

        buf.set_threshold(threshold);
    }
}

#[deriving(Default)]
struct Buffers {
	threads: String,
    hash_size: String,
    threshold: String,
	dir: String,
}

impl Buffers {
    #[inline]
    fn set_threshold(&mut self, threshold: f32) {       
        self.threshold.clear();
        write_str!(self.threshold, "Threshold: {:.2}%", threshold * 100.0);
    }
}

fn draw_setup_dialog(gl: &mut Gl, uic: &mut UiContext, state: &mut ConfigState, buf: &mut Buffers) {
    background(gl, uic);
	
    const DIR: u64 = 1;
	uic.text_box(DIR, &mut buf.dir)
        .font_size(18)
		.position(5.0, 25.0)
		.dimensions(665.0, 30.0)
		.callback(|dir| state.update_dir(dir))
		.draw(gl);
		
	uic.label("Search Directory")
        .size(18)
		.up_from(DIR, 25.0)
		.draw(gl);

    const BROWSE: u64 = DIR + 1;
    uic.button(BROWSE)
        .label("Browse")
        .dimensions(60.0, 30.0)
        .label_font_size(18)
        .right_from(DIR, 5.0)
        .callback(|| 
            if let Some(dir) = open_folder_dialog(&state.settings.dir) {
                state.set_dir(buf, dir)
            }
        )
        .draw(gl);

    const THREADS: u64 = BROWSE + 1;
    uic.text_box(THREADS, &mut buf.threads)
        .down_from(DIR, 30.0)
        .dimensions(40.0, 30.0)
        .callback(|threads| state.set_threads(threads))
        .draw(gl);        

    uic.label("Threads")
        .size(18)
        .up_from(THREADS, 20.0)
        .draw(gl);

    draw_spinner(gl, uic, THREADS, |inc| state.add_threads(buf, inc));
    
    const HASH_SIZE: u64 = THREADS + 3;
    uic.text_box(HASH_SIZE, &mut buf.hash_size)
        .right_from(THREADS, 30.0)
        .dimensions(60.0, 30.0)
        .callback(|hash_size| state.set_hash_size(hash_size))
        .draw(gl);

    uic.label("Hash Size")
        .size(18)
        .up_from(HASH_SIZE, 20.0)
        .draw(gl);

    draw_spinner(gl, uic, HASH_SIZE, |inc| state.add_hash_size(buf, inc));
    
    const RECURSE: u64 = HASH_SIZE + 3;
    // Invert boolean so the toggle is dark when true
    uic.toggle(RECURSE, !state.settings.recurse)
        .right_from(HASH_SIZE, 30.0)
        .dimensions(30.0, 30.0)
        .callback(|recurse| state.settings.recurse = !recurse)
        .draw(gl);
        
    uic.label("Recurse")
        .size(18)
        .up_from(RECURSE, 20.0)
        .draw(gl);

    const USE_DCT: u64 = RECURSE + 1;
    // Already inverted
    uic.toggle(USE_DCT, state.settings.fast)
        .right_from(RECURSE, 35.0)
        .dimensions(30.0, 30.0)
        .callback(|use_dct| state.settings.fast = use_dct)
        .draw(gl);

    uic.label("Use DCT")
        .size(18)
        .up_from(USE_DCT, 20.0)
        .draw(gl);
        
    const THRESHOLD: u64 = USE_DCT + 1;
    uic.slider(THRESHOLD, state.settings.threshold, 0.01, 0.10)
        .right_from(USE_DCT, 50.0)
        .dimensions(240.0, 30.0)
        .callback(|threshold| state.set_threshold(buf, threshold))
        .draw(gl);

    uic.label(&*buf.threshold)
        .up_from(THRESHOLD, 20.0)
        .size(18)
        .draw(gl);
    
    const GO: u64 = THRESHOLD + 1;
    uic.button(GO)
        .label("Go!")
        .down_from(BROWSE, 30.0)
        .dimensions(60.0, 30.0)
        .callback(|| state.confirmed = true)
        .draw(gl);

    const CANCEL: u64 = GO + 1;
    uic.button(CANCEL)
        .label("Cancel")
        .left_from(GO, 85.0)
        .dimensions(80.0, 30.0)
        .callback(|| state.canceled = true)
        .draw(gl);
}

const UP_LBL: &'static str = "▲";
const DWN_LBL: &'static str = "▼";

#[inline]
fn draw_spinner(gl: &mut Gl, uic: &mut UiContext, id: u64, callback: |int|) {
    uic.button(id + 1)
        .right_from(id, 2.0)
        .dimensions(15.0, 14.0)
        .label(UP_LBL)
        .label_font_size(9)
        .callback(|| callback(1))
        .draw(gl);
        
    uic.button(id + 2)
        .down_from(id + 1, 2.0)
        .dimensions(15.0, 14.0)
        .label(DWN_LBL)
        .label_font_size(9)
        .callback(|| callback(-1))
        .draw(gl);     
}

fn open_folder_dialog(start_path: &Path) -> Option<Path> {
    use ui::file_dialog::{FileDialog, SelectType};

    let font = GlyphCache::new(&super::font()).unwrap();

    let promise = FileDialog::new("Select Search Directory", font)
        .set_select(SelectType::Folder)
        .set_starting_path(start_path.clone())
        .show(OpenGL::_3_2);

    promise.join().unwrap_or(None)
}
