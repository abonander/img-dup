extern crate conrod;
extern crate quack;
extern crate event;
extern crate file_dialog;
extern crate graphics;
extern crate gl;
extern crate opengl_graphics;
extern crate sdl2;
extern crate sdl2_window;

use config::ProgramSettings;

macro_rules! write_str(
	($s:expr, $fmt:expr, $($args:expr),+) => (
		{
			let vec = unsafe { $s.as_mut_vec() };
			// `Err` would be rare here, and means something *very* bad happened.
			(write!(vec, $fmt, $($args),+)).unwrap();
		}
	)
);

pub mod dialogs;
mod setup;
mod running;
mod results;
mod errors;
mod util;

pub fn show_gui(mut settings: ProgramSettings) {
    let mut again = true;

	while again {
        again = setup::show_setup_ui(settings.clone())
            .and_then(|config| { 
                settings = config.clone(); 
                running::start_processing(config)
            })
            .map(results::show_results)
            .unwrap_or(false);
    }
}

fn font() -> Path {
	Path::new("assets/FreeSerif.otf")
}

/// Everything that should be needed to create a UI.
pub mod prelude {
    pub use ui::conrod::*;
    pub use ui::quack::Set;
    pub use ui::event::{Event, Events, Ups, MaxFps, WindowSettings};
    pub use ui::opengl_graphics::{Gl, OpenGL};
    pub use ui::opengl_graphics::glyph_cache::GlyphCache;
    pub use ui::sdl2_window::Sdl2Window;

    use std::borrow::ToOwned;

    pub type UiEvents = Events<Sdl2Window>;

    const GL_VER: OpenGL = OpenGL::_3_2;

    pub fn create_window(name: &str, dimen: [u32, ..2]) -> (UiContext, Gl, UiEvents) {
	    let window = Sdl2Window::new(GL_VER, WindowSettings {
		    title: name.to_owned(),
		    size: dimen,
		    fullscreen: false,
		    exit_on_esc: false,
		    samples: 4,
	    });
	
	    let events = Events::new(window).set(Ups(120)).set(MaxFps(60));
	    let gl = Gl::new(GL_VER);

	    let theme = Theme::default();
	    let font = GlyphCache::new(&super::font()).unwrap();
	
	    let uic = UiContext::new(font, theme);
        
        (uic, gl, events)    
    }

    #[inline]
    pub fn background(gl: &mut Gl, uic: &mut UiContext) {
        uic.background().color(Color([0.9, 0.9, 0.9, 1.0])).draw(gl);    
    }
}

