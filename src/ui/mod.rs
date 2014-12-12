extern crate conrod;
extern crate event;
extern crate opengl_graphics;
extern crate sdl2_window;

use config::ProgramSettings;

macro_rules! write_str(
	($s:expr, $fmt:expr, $($args),+) => (
		{
			let vec = unsafe { $s.as_mut_vec() };
			// `Err` would be rare here, and means something *very* bad happened.
			(write!(vec, $fmt, $($args),+)).unwrap();
		}
	)
)

mod setup;
mod running;
mod results;

fn show_ui(settings: ProgramSettings) {
	setup::show_setup_ui(settings);				
}

fn font() -> Path {
	Path::new("../assets/FreeSerif.otf")
}

