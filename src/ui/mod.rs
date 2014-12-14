use config::ProgramSettings;

macro_rules! write_str(
	($s:expr, $fmt:expr, $($args:expr),+) => (
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

pub fn show_gui(settings: ProgramSettings) {
	if let Some(results) = setup::show_setup_ui(settings).map(running::start_processing) {
            results::show_results(results)
    }
}

fn font() -> Path {
	Path::new("assets/FreeSerif.otf")
}

