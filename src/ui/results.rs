use super::dialogs;
use super::prelude::*;
use super::running::Results;
use super::util::FormatBytes;

use img::UniqueImage;
use processing::{mod, TimedImageResult, ProcessingError, Total};

use graphics::{
	mod,
	BackEnd, 
	Context, 
	Image, 
	ImageSize, 
	Rectangle,
	RelativeTransform,
};
use gl;
use opengl_graphics::Texture;

use image::{
    mod, 
    DynamicImage,
    GenericImage,
    ImageBuffer, 
    ImageResult, 
    Rgba
};

use sdl2::mouse::{Cursor, SystemCursor};

use std::borrow::ToOwned;
use std::cell::Cell;
use std::iter::Peekable;
use std::io::fs;
use std::mem;
use std::vec::IntoIter;

pub fn show_results(results: Results) -> bool {  
    let ref consts = Constants {
        avg_load: results.avg_load,
        avg_hash: results.avg_hash,
        elapsed: results.elapsed,
        errors: results.errors,
    };

    debug!("Done: {}", results.done.len());

	const WINDOW_SIZE: [u32; 2] = [1024, 768];
	let (mut uic, mut gl, mut events) = create_window("img-dup results", WINDOW_SIZE);

	let mut state = match ResultsState::new(results.done.into_iter()) {
		Some(state) => state,
		None => return false,
    };

	if state.current.similars.is_empty() {
		state.move_to_next();
	}

    for event in events {
		if state.exit { break; }

		uic.handle_event(&event);

		match event {
			Event::Render(args) => {
				gl.draw([0, 0, args.width as i32, args.height as i32], |ref ctx, gl| {
					draw_results_ui(gl, ctx, &mut uic, &mut state, consts);
				});
			}
			_ => (),
		}
	}

	dialogs::confirm("img_dup again?", "No duplicates left!", "Scan again?")
}

struct Constants {
    avg_load: String,
    avg_hash: String,
    elapsed: String,
    errors: Vec<ProcessingError>,
}

struct ResultsState {
    done: IntoIter<UniqueImage>,
    current: UniqueImage,
    next: Option<UniqueImage>,
    compare_select: Option<uint>,
    exit: bool,
    buf: Buffers,
	wait_cursor: Cursor,
	reg_cursor: Cursor, 
}

impl ResultsState {
    fn new(mut done: IntoIter<UniqueImage>) -> Option<ResultsState> {
        match done.next() {
            Some(current) => {
                let next = done.next();
				
				let wait_cursor = Cursor::from_system(SystemCursor::Wait)
					.unwrap();
				wait_cursor.set();

                let buf = Buffers::create(&current, next.as_ref());

				let reg_cursor = Cursor::from_system(SystemCursor::Arrow)
					.unwrap();
				reg_cursor.set();
            
                Some(
                    ResultsState {
                        done: done,
                        current: current,
                        next: next,
                        compare_select: None,
                        exit: false,
						buf: buf,
						wait_cursor: wait_cursor,
						reg_cursor: reg_cursor,
					}
                )
            },
            None => None,
        }
    }
       
    fn move_to_next(&mut self) {
        self.current = match mem::replace(&mut self.next, self.done.next()) {
            Some(ref next) if next.similars.is_empty() => {
				self.move_to_next(); 
				return;
			},
			Some(next) => next,
            _ => {
				self.exit = true; 
				return; 
			},
        };
                       
        self.update_buffers();
		self.compare_select = None;
    }

    fn update_buffers(&mut self) {
		self.wait_cursor.set(); 
        self.buf = Buffers::create(&self.current, self.next.as_ref());
		self.reg_cursor.set(); 
    }

    fn promote(&mut self, idx: uint) {
        self.current.promote(idx);
        mem::swap(&mut self.buf.current, &mut self.buf.compares[idx]); 
    }

	fn delete(&mut self, idx: uint) {
		fs::unlink(&self.current.similars[idx].img.path);
		self.remove_compare(idx);
	}

	fn symlink(&mut self, idx: uint) {
		{
			let ref path = self.current.similars[idx].img.path;
			fs::unlink(path);
			fs::symlink(&self.current.img.path, path);
		}

		self.remove_compare(idx);	
	}

	fn remove_compare(&mut self, idx: uint) {
		self.current.similars.remove(idx);
		self.buf.compares.remove(idx);
		self.compare_select = None;
		
		if self.buf.compares.is_empty() {
			self.move_to_next();
		}	
	}
}

struct Buffers {
    current: ImageBuf,
    preview_next: Option<ImageBuf>,
    compares: Vec<ImageBuf>,
}

impl Buffers {
    fn create(current: &UniqueImage, next: Option<&UniqueImage>) -> Buffers {
        Buffers {
            current: ImageBuf::open(&current.img.path, 0.0).unwrap(),
            preview_next: next.map(|img| ImageBuf::open(&img.img.path, 0.0).unwrap()),
            compares: current.similars
                .iter()
                .map(|similar| ImageBuf::open(&similar.img.path, similar.dist_ratio).unwrap())
                .collect(),
        }            
    }    
}

fn draw_results_ui(
    gl: &mut Gl, ctx: &Context, 
    uic: &mut UiContext, 
    state: &mut ResultsState, consts: &Constants
) {
    background(gl, uic);
	
	uic.label("Next Image (Click)")
		.position(5.0, 5.0)
		.size(18)
		.draw(gl);

	const PREVIEW_IMG_POS: [f64; 2] = [5.0, 30.0];
	const PREVIEW_IMG_SIZE: [f64; 2] = [150.0, 150.0];

	const NEXT: u64 = 1;
	uic.button(NEXT)
		.color(Color::black())
		.point(PREVIEW_IMG_POS)
		.dim(PREVIEW_IMG_SIZE)
		.label(if state.buf.preview_next.is_none() { "None" } else { "" })
		.label_font_size(24)
		.label_color(Color::white())
		.callback(||
			if state.buf.preview_next.is_some() {
				state.move_to_next(); 
			}
		)
		.draw(gl);

	state.buf.preview_next.as_ref().map(
		|next| next.draw(PREVIEW_IMG_POS, PREVIEW_IMG_SIZE, gl, ctx)
	);

	const IMG_SIZE: [f64; 2] = [480.0, 578.0];	
 
	{
		let ref current = state.buf.current;
		current.draw([5.0, 190.0], IMG_SIZE, gl, ctx);

		uic.label(&*current.name)
			.position(160.0, 145.0)
			.size(18)
			.draw(gl);

		uic.label(&*current.size)
			.position(160.0, 165.0)
			.size(18)
			.draw(gl);
	}

	const COMPARE_POS: [f64; 2] = [539.0, 190.0];
	const SHRINK_COMPARE: u64 = NEXT + 1;

	if let Some(idx) = state.compare_select {
		if idx >= state.buf.compares.len() { 
			state.compare_select = None; 
		} else {
			uic.button(SHRINK_COMPARE)
				.color(Color::black())
				.point(COMPARE_POS)
				.dim(IMG_SIZE)
				.callback(|| state.compare_select = None)
				.draw(gl);

			state.buf.compares[idx].draw(COMPARE_POS, IMG_SIZE, gl, ctx);

			const BUTTON_DIM: [f64; 2] = [70.0, 30.0];

			const IGNORE: u64 = SHRINK_COMPARE + 1;
			uic.button(IGNORE)
				.label("Ignore")
				.label_font_size(18)
				.up_from(SHRINK_COMPARE, 35.0)
				.dim(BUTTON_DIM)
				.callback(|| state.remove_compare(idx))
				.draw(gl);

			const PROMOTE: u64 = IGNORE + 1;
			uic.button(PROMOTE)
				.label("Promote")
				.label_font_size(18)
				.up_from(IGNORE, 35.0)
				.dim(BUTTON_DIM)
				.callback(|| state.promote(idx))
				.draw(gl);

			const SYMLINK: u64 = PROMOTE + 1;
			uic.button(SYMLINK)
				.label("Symlink")
				.label_font_size(18)
				.right_from(IGNORE, 5.0)
				.dim(BUTTON_DIM)
				.callback(|| state.symlink(idx))
				.draw(gl);

			const DELETE: u64 = SYMLINK + 1;
			uic.button(DELETE)
				.label("Delete")
				.label_font_size(18)
				.up_from(SYMLINK, 35.0)
				.dimensions(70.0, 30.0)
				.callback(|| state.delete(idx))
				.draw(gl);

			if let Some(similar) = state.buf.compares.get(idx) {
				uic.label(&*similar.name)
					.position(699.0, 145.0)
					.size(18)
					.draw(gl);

				uic.label(&*similar.size)
					.position(699.0, 165.0)
					.size(18)
					.draw(gl);

				uic.label(&*similar.percent)
					.position(699.0, 125.0)
					.size(18)
					.draw(gl);
			}
		}
	} else {
		const COLS: uint = 5;
		const ROWS: uint = 5;
		const LABEL_SIZE: u32 = 12;

		uic.widget_matrix(COLS, ROWS)
			.point(COMPARE_POS)
			.dim(IMG_SIZE)
			.cell_padding(5.0, 15.0)
			.each_widget(|uic, id, x, y, pt, dim| {
				let idx = y * COLS + x;

				if idx >= state.buf.compares.len() { return; }
		
				uic.button(id as u64 + 30)
					.color(Color::black())
					.point(pt)
					.dim(dim)
					.callback(|| state.compare_select = Some(idx))
					.draw(gl);
					 
				let ref similar = state.buf.compares[idx];

				similar.draw(pt, dim, gl, ctx);

				let name_y = pt[1] + dim[1] + 5.0;

				uic.label(&*similar.percent)
					.position(pt[0], name_y)
					.size(LABEL_SIZE)
					.draw(gl);

				uic.label(&*similar.name)
					.position(pt[0], name_y + 15.0)
					.size(LABEL_SIZE)
					.draw(gl);

				uic.label(&*similar.size)
					.position(pt[0], name_y + 30.0)
					.size(LABEL_SIZE)
					.draw(gl);
			});
	}
} 

struct ImageBuf {
    image: Texture,
    name: String,
    size: String,
	percent: String,
}

impl ImageBuf {
    fn open(path: &Path, percent: f32) -> ImageResult<ImageBuf> {
        let image = try!(image::open(path));
        
        let name = truncate_name(path, 24);
        let (width, height) = image.dimensions();
        let file_size = try!(fs::stat(path)).size;

        let size = format!("{} x {} ({})", width, height, FormatBytes(file_size));

		let percent = format!("{:.02}%", (100.0 - percent * 100.0));

        debug!("Name: {} Size: {} Percent: {}", name, size, percent);
 
        let tex = Texture::from_image(&image.to_rgba());
         
        Ok(ImageBuf {
                image: tex,
                name: name,
                size: size,
				percent: percent,
        })
    }

    fn draw(
		&self, 
		pos: [f64; 2], size: [f64; 2], 
		gl: &mut Gl, ctx: &Context,
	) {
		let (width, height) = self.image.get_size();
		
		let scale = if (width as f64 - size[0]) > (height as f64 - size[1]) {
			size[0] / (width as f64)
		} else {
			size[1] / (height as f64)
		};
	
		let ref ctx = ctx.trans(pos[0], pos[1]);

		graphics::Rectangle::new([0.0, 0.0, 0.0, 1.0])
			.draw([0.0, 0.0, size[0], size[1]], ctx, gl);

		let ref ctx = ctx.zoom(scale);

		graphics::image(&self.image, ctx, gl);		
    }  
}

fn truncate_name(path: &Path, len: uint) -> String {
	const TRUNC_STR: &'static str = "[..]";

	let ext = path.extension_str().unwrap_or("");

	let max_len = len - (ext.len() + 1);
	let trunc_len = max_len - TRUNC_STR.len();

	let filestem = path.filestem_str().unwrap_or("");

	if filestem.len() > max_len { 
			format!("{}{}.{}", filestem.slice_to(trunc_len), TRUNC_STR, ext) 
	} else { 
		path.filename_display().to_string()
	}
}
