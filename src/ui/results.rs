use super::prelude::*;
use super::running::Results;
use super::util::FormatBytes;

use img::UniqueImage;
use processing::{mod, TimedImageResult, ProcessingError, Total};

use graphics::{mod,BackEnd, Context, Image, Rect};
use opengl_graphics::Texture;

use image::{
    mod, 
    DynamicImage,
    GenericImage,
    ImageBuffer, 
    ImageResult, 
    Rgba
};

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

    if let Some(state) = ResultsState::new(results.done.into_iter()) {
        results_window(state, consts)        
    } else {
        false
    } 
}

fn results_window(mut state: ResultsState, consts: &Constants) -> bool {
	let (mut uic, mut gl, mut events) = create_window("img-dup results", [1024, 768]);

    for event in events {
        if state.go_again { return true; }

		uic.handle_event(&event);

		match event {
			Event::Render(args) => {
				gl.draw([0, 0, args.width as i32, args.height as i32], |ctx, gl| {
					draw_results_ui(gl, &ctx, &mut uic, &mut state, consts);				
				});
			}
			_ => (),
		}
	}

    false
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
    go_next: bool,
    compare_select: Option<uint>,
    go_again: bool,
    buf: Buffers,       
}

impl ResultsState {
    fn new(mut done: IntoIter<UniqueImage>) -> Option<ResultsState> {
        match done.next() {
            Some(current) => {
                let next = None; //done.next();
                let buf = Buffers::create(&current, next.as_ref());
            
                Some(
                    ResultsState {
                        done: done,
                        current: current,
                        next: next,
                        go_next: false,
                        compare_select: None,
                        go_again: false,
                        buf: buf,
                    }
                )
            },
            None => None,
        }
    }
       
    fn move_to_next(&mut self, buf: &mut Buffers) -> bool {
        self.current = match mem::replace(&mut self.next, self.done.next()) {
            Some(next) => next,
            _ => return false,
        };
                       
        self.update_buffers();

        true
    }

    fn update_buffers(&mut self) { 
        self.buf = Buffers::create(&self.current, self.next.as_ref()); 
    }
    
    fn promote(&mut self, idx: uint) {
        self.current.promote(idx);
        self.update_buffers(); 
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
            current: ImageBuf::open(&current.img.path).unwrap(),
            preview_next: next.map(|img| ImageBuf::open(&img.img.path).unwrap()),
            compares: Vec::new(), /*current.similars
                .iter()
                .map(|similar| ImageBuf::open(&similar.img.path).unwrap())
                .collect(),*/
        }            
    }    
}

fn draw_results_ui(
    gl: &mut Gl, ctx: &Context, 
    uic: &mut UiContext, 
    state: &mut ResultsState, consts: &Constants
) {
    //background(gl, uic);
    state.buf.current.draw(gl, ctx);
} 
struct ImageBuf {
    image: Texture,
    name: String,
    size: String,
}

impl ImageBuf {
    fn open(path: &Path) -> ImageResult<ImageBuf> {
        let image = try!(image::open(path));
        
        let name = path.filename_display().to_string();
        let (width, height) = image.dimensions();
        let file_size = try!(fs::stat(path)).size;

        let size = format!("{} x {} ({})", width, height, FormatBytes(file_size));

        debug!("Name: {} Size: {}", name, size);
 
        let tex = Texture::from_image(&image.to_rgba());
         
        Ok(ImageBuf {
                image: tex,
                name: name,
                size: size,
        })
    }

    fn draw(&self, gl: &mut Gl, ctx: &Context) {
        graphics::image(&self.image, ctx, gl);             
    }  
}
