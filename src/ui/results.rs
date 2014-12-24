use super::prelude::*;
use super::running::Results;
use super::util::FormatBytes;

use img::UniqueImage;
use processing::{mod, TimedImageResult, ProcessingError, Total};

use graphics::{Context, Image, Rect};
use opengl_graphics::{Gl, Texture};

use image::{
    mod, 
    DynamicImage,
    ImageBuffer, 
    ImageResult, 
    Rgba
};

use std::iter::Peekable;
use std::io::fs;
use std::vec::IntoIter;

pub fn show_results(results: Results) -> bool { 
	let (mut uic, mut gl, mut events) = create_window("img-dup results", [1024, 768]);
   
    let mut done = 

    let mut state = ResultsState {
        done: results.done.into_iter().peekable(),
         
    

    for event in events {
		uic.handle_event(&event);

		match event {
			Event::Render(args) => {
				gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
					draw_results_ui(gl, &mut uic, &mut buf);				
				});
			}
			_ => (),
		}
	}

}

fn results_window(state: ResultsState, consts: &Constants) {
        
}


struct ResultsState {
    done: MoveItems<UniqueImage>,
    current: UniqueImage,
    next: Option<UniqueImage>,
    go_next: bool,
    compare_select: Option<uint>,        
}

impl ResultsState {    
       
}

struct Buffers {
    current: ImageBuf,
    preview_next: ImageBuf,
}

struct Constants {
    avg_load: String,
    avg_hash: String,
    elapsed: String,
}

struct MutBuffers {
    
    }

fn draw_results_ui(gl: &mut Gl, uic: &mut UiContext, state: &mut ResultsState, consts: &Constants) {
    background(gl, uic);

    
    uic.button(
        
}



struct ImageBuf {
    image: Texture,
    name: String,
    size: String,


}

impl DrawImage {
    fn open(path: &Path) -> ImageResult<DrawImage> {
        let image = try!(image::open(path));
        
        let name = path.filename_display().to_string();
        let (width, height) = image.dimensions();
        let file_size = try!(fs::stat(path)).size;

        let size = format!("{} x {} ({})", width, height, file_size);

         
        Ok(DrawImage {
                image: image,
                name: 
        })

    }

    fn draw(&self, rect: [f64; 4], gl: &mut Gl) {
        Image::color([0.0, 0.0, 0.0, 1.0])
            .set(Rect(rect))
            .draw(self.texture, &Context::new(), gl);             
    }  
}
