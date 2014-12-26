use ui::dialogs;
use ui::errors::{show_errors_list, ErrorBuf};
use ui::prelude::*;
use ui::running::Results;

use img::UniqueImage;

use graphics::{
	mod,
	BackEnd, 
	Context, 
	ImageSize, 
	RelativeTransform,
};
use opengl_graphics::Texture;

use image::{
    mod, 
    GenericImage,
    ImageResult, 
};

use sdl2::mouse::{Cursor, SystemCursor};

use std::fmt::{mod, Show, Formatter};
use std::io::fs;
use std::mem;
use std::sync::Arc;

pub fn show_results(results: Results) -> bool {  
    let ref consts = Constants {
        avg_load: results.avg_load,
        avg_hash: results.avg_hash,
        elapsed: results.elapsed,
        total: format!("Total Images Processed: {}", results.total),
        view_errors: format!("View Errors ({})", results.errors.len()),
        errors: ErrorBuf::arc_vec(results.errors, &results.search_path),
    };

	const WINDOW_SIZE: [u32; 2] = [1024, 768];
	let (mut uic, mut gl, mut events) = create_window("img-dup results", WINDOW_SIZE);

    draw_loading_message(&mut gl, &mut uic, &mut events);

    let mut done = results.done;
    done.retain(|unique| !unique.similars.is_empty());

	let mut state = match ResultsState::new(done) {
		Some(state) => state,
		None => return scan_again(),
    };
	
    for event in events {
		if state.exit { return scan_again(); }

		uic.handle_event(&event);

		match event {
			Event::Render(args) => {
				gl.draw([0, 0, args.width as i32, args.height as i32], |ref ctx, gl| {
					draw_results_ui(gl, ctx, &mut uic, &mut state, consts);
				});
			},
			_ => (),
		}
	}
    
    // User clicked exit
    false	
}

// Draw the message on a single frame, then return.
fn draw_loading_message(gl: &mut Gl, uic: &mut UiContext, events: &mut UiEvents) {
    let mut drawn = false;

    for event in *events {
        uic.handle_event(&event);

		match event {
			Event::Render(args) => {
                if drawn { break; }

				gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
				    background(gl, uic);
                    uic.label("Loading results, please wait...")
                        .position(360.0, 324.0)
                        .size(24)
                        .draw(gl);                 
                });

                drawn = true;
			},
			_ => (),
		}
    }            
}

struct Constants {
    avg_load: String,
    avg_hash: String,
    elapsed: String,
    total: String,
    view_errors: String,
    errors: Arc<Vec<ErrorBuf>>,
}

struct ResultsState {
    done: Vec<UniqueImage>,
    current: UniqueImage,
    next: Option<UniqueImage>,
    compare_select: Option<uint>,
    exit: bool,
    buf: Buffers,
    next_str: String,
	wait_cursor: Cursor,
	reg_cursor: Cursor, 
}

impl ResultsState {
    fn new(mut done: Vec<UniqueImage>) -> Option<ResultsState> {
        match done.pop() {
            Some(current) => {
                let next = done.pop();
				
				let wait_cursor = Cursor::from_system(SystemCursor::Wait)
					.unwrap();
				wait_cursor.set();

                let buf = Buffers::create(&current, next.as_ref());

				let reg_cursor = Cursor::from_system(SystemCursor::Arrow)
					.unwrap();
				reg_cursor.set();

                let mut next_str = String::new();

                fmt_next_str(&mut next_str, done.len(), next.is_some());
            
                Some(
                    ResultsState {
                        done: done,
                        current: current,
                        next: next,
                        compare_select: None,
                        exit: false,
						buf: buf,
                        next_str: next_str,						
                        wait_cursor: wait_cursor,
						reg_cursor: reg_cursor,
					}
                )
            },
            None => None,
        }
    }
       
    fn move_to_next(&mut self) {
        self.current = match mem::replace(&mut self.next, self.done.pop()) {
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

        fmt_next_str(&mut self.next_str, self.done.len(), self.next.is_some());
    }

    fn promote(&mut self, idx: uint) {
        self.current.promote(idx);
        mem::swap(&mut self.buf.current, &mut self.buf.compares[idx]); 
    }

	fn delete(&mut self, idx: uint) {
		print_err(fs::unlink(&self.current.similars[idx].img.path));
		self.remove_compare(idx);
	}

	fn symlink(&mut self, idx: uint) {
		{
			let ref path = self.current.similars[idx].img.path;

            print_err(
                fs::unlink(path).and_then(|_| fs::symlink(&self.current.img.path, path))
            );
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

#[inline]
fn fmt_next_str(s: &mut String, remaining: uint, add_one: bool) {
    s.clear();
    write_str!(s, "Next ({} left)", remaining + if add_one { 1 } else { 0 });    
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
	
	uic.label(&*state.next_str)
		.position(5.0, 5.0)
		.size(18)
		.draw(gl);

	const PREVIEW_IMG_POS: [f64; 2] = [5.0, 30.0];
	const PREVIEW_IMG_SIZE: [f64; 2] = [75.0, 75.0];

	const NEXT: u64 = 1;
	uic.button(NEXT)
		.color(Color::black())
		.point(PREVIEW_IMG_POS)
		.dim(PREVIEW_IMG_SIZE)
		.label(if state.buf.preview_next.is_none() { "None" } else { "" })
		.label_font_size(24)
		.label_color(Color::white())
		.callback(
            || if state.buf.preview_next.is_some() && confirm_skip() {
				state.move_to_next(); 
			}
		)
		.draw(gl);

	state.buf.preview_next.as_ref().map(
		|next| next.draw(PREVIEW_IMG_POS, PREVIEW_IMG_SIZE, gl, ctx)
	);

    const AVGS_X: f64 = 225.0;
    uic.label(&*consts.avg_load)
        .position(AVGS_X, 5.0)
        .size(18)
        .draw(gl);

    uic.label(&*consts.avg_hash)
        .position(AVGS_X, 25.0)
        .size(18)
        .draw(gl);

    uic.label(&*consts.elapsed)
        .position(AVGS_X + 165.0, 5.0)
        .size(18)
        .draw(gl);

    uic.label(&*consts.total)
        .position(AVGS_X + 300.0, 5.0)
        .size(18)
        .draw(gl);
    
    const VIEW_ERRORS: u64 = NEXT + 1;
    uic.button(VIEW_ERRORS)
        .position(869.0, 20.0)
        .dimensions(150.0, 30.0)
        .label(&*consts.view_errors)
        .label_font_size(20)
        .callback(|| show_errors_list(consts.errors.clone()))
        .draw(gl);

    const SCAN_AGAIN: u64 = VIEW_ERRORS + 1;
    uic.button(SCAN_AGAIN)
        .down_from(VIEW_ERRORS, 5.0)
        .dimensions(150.0, 30.0)
        .label("Scan Again")
        .label_font_size(20)
        .callback(|| state.exit = true)
        .draw(gl);

	const IMG_SIZE: [f64; 2] = [500.0, 660.0];
    const IMG_Y: f64 = 115.0;	
 
	{
		let ref current = state.buf.current;
		current.draw([5.0, IMG_Y], IMG_SIZE, gl, ctx);

		uic.label(&*current.name)
			.position(160.0, IMG_Y - 45.0)
			.size(18)
			.draw(gl);

		uic.label(&*current.size)
			.position(160.0, IMG_Y - 25.0)
			.size(18)
			.draw(gl);
	}

	const COMPARE_POS: [f64; 2] = [519.0, IMG_Y];
	const SHRINK_COMPARE: u64 = SCAN_AGAIN + 1;

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
				.callback(
                    || if confirm_symlink() { state.symlink(idx); })
				.draw(gl);

			const DELETE: u64 = SYMLINK + 1;
			uic.button(DELETE)
				.label("Delete")
				.label_font_size(18)
				.up_from(SYMLINK, 35.0)
				.dimensions(70.0, 30.0)
				.callback(|| if confirm_delete() { state.delete(idx); })
				.draw(gl);

			if let Some(similar) = state.buf.compares.get(idx) {
				uic.label(&*similar.name)
					.position(699.0, IMG_Y - 45.0)
					.size(18)
					.draw(gl);

				uic.label(&*similar.size)
					.position(699.0, IMG_Y - 25.0)
					.size(18)
					.draw(gl);

				uic.label(&*similar.percent)
					.position(699.0, IMG_Y - 65.0)
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
			.cell_padding(15.0, 15.0)
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

		let percent = format!("Diff: {:.02}%", percent * 100.0);
 
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

fn confirm_symlink() -> bool {
    dialogs::confirm(
        "Symlink image?", 
        "Image will be replaced. This cannot be undone!"
    ) 
}

fn confirm_delete() -> bool {
    dialogs::confirm(
        "Delete image permanently?",
        "This cannot be undone!"
    )  
}

fn confirm_skip() -> bool {
    dialogs::confirm(
        "Skip to next image?",
        "You won't be able to go back!"
    )  
}

fn scan_again() -> bool {
    dialogs::confirm("No matches remaining or found!", "Scan again?")    
}

fn print_err<T, E>(result: Result<T, E>) where E: Show {
    if let Err(err) = result {
        println!("Encountered nonfatal error: {}", err);    
    }
}

#[deriving(Copy)]
struct FormatBytes(pub u64);

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
            0 ... 999 => format_args!(|args| fmt.write_fmt(args), "{} B", self.0),
            1_000 ... 999_999 => format_args!(|args| fmt.write_fmt(args), "{:.02} KB", self.to_kb()),
            1_000_000 ... 999_999_999 => format_args!(|args| fmt.write_fmt(args), "{:.02} MB", self.to_mb()),
            _ => format_args!(|args| fmt.write_fmt(args), "{:.02} GB", self.to_gb()),
        }
    }        
}

