use opengl_graphics::OpenGL;

use sdl2_window::Window;

use config::ProgramSettings;

fn show_setup_ui(settings: ProgramSettings) {	
	let ref mut state = ConfigState::from_settings(settings);

	
	  
}

struct ConfigState {
	settings: ProgramSettings,
	buffers: Buffers
}

impl ConfigState {
	fn from_settings(settings: ProgramSettings) -> ConfigState {
		let mut buffers: Buffers = Default::default();

		write_str!(buffers.threads, "{}", settings.threads);
		write_str!(buffers.dir, "{}", settings.dir.display());

		ConfigState {
			settings: settings,
			buffers: Buffers,	
		}
	}

	fn set_threads(&mut self, threads: &mut String) {
		if let Some(threads) = from_str::<uint>(&*threads) {
			self.settings.threads = threads;
		} else {
			threads.clear();
			write_str!(threads, "{}", self.settings.threads);	
		}
	}

	fn set_dir(&mut self, dir: Path) {
		self.buffers.dir.clear();
		write_str!(self.buffers.dir, "{}", dir.display());
	
		self.settings.dir = dir;
	}
}

#[deriving(Default)]
struct Buffers {
	threads: String,
	dir: String,
}

fn draw_setup_dialog(gl: &mut Gl, uic: &mut UiContext, state: &mut ConfigState) {
		
}
