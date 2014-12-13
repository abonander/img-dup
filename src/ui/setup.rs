use conrod::*;
use event::{Event, Events, Ups, MaxFps, WindowSettings};
use opengl_graphics::OpenGL;
use sdl2_window::Window;

use config::ProgramSettings;

fn show_setup_ui(settings: ProgramSettings) {
	const GL_VER: OpenGL = OpenGL::_3_2;
	
	let ref mut state = ConfigState::from_settings(settings);

	let window = Window::new(GL_VER, WindowSettings {
		title: "img-dup configuration".into_string(),
		width: 640,
		height: 480,
		fullscreen: false,
		exit_on_esc: false,
		samples: 4,
	});
	
	let mut events = Events::new(window).set(Ups(120)).set(MaxFps(60));
	let mut gl = Gl::new(GL_VER);

	let theme = Theme::default();
	let font = GlyphCache::new(&super::font()).unwrap();
	
	let ref mut uic = UiContext::new(font, theme);

	for event in events {
		uic.handle_event(&event);
		match event {
			Event::Render(args) => {
				gl.draw([0, 0, args.width as u32, args.height as u32], |_, gl| {
					draw_setup_dialog(gl, uic, state);				
				});
			}
			_ => (),
		}
	}
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
	const DIR: u64 = 1;
	
	uic.text_box(DIR, state.buffers.dir)
		.position(5.0, 30.0)
		.dimensions(140.0, 20.0)
		.callback(|dir| state.set_dir(dir))
		.draw(gl);
		
	uic.label("Search Directory")
		.up_from(DIR, 5.0)
		.draw(gl);
}
