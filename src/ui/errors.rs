use ui::prelude::*;

use processing::ProcessingError;

use std::sync::Arc;
use std::thread::Thread;

pub fn show_errors_list(errors: Arc<Vec<ErrorBuf>>) {
    Thread::spawn(move || {
        let (mut uic, mut gl, mut events) = create_window("img-dup errors", [512, 512]);

        let mut buf: Buffers = Buffers::new(errors.len());

        for event in events {
            if buf.exit { break; }

            uic.handle_event(&event);

            match event {
                Event::Render(args) => {
				    gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
					    draw_errors_ui(gl, &mut uic, &mut buf, &**errors);
				    });
			    },
			    _ => (),
            }
        }
    }).join().unwrap_or(());
}

struct Buffers {
    current: usize,
    curr_str: String,
    total: usize,
    exit: bool,
}

impl Buffers {
    fn new(total: usize) -> Buffers {
        let mut buf = Buffers {
            current: if total > 0 { 1 } else { 0 },
            curr_str: String::new(),
            total: total,
            exit: false,
        };

        buf.update_curr_str();

        buf
    }

    fn prev(&mut self) {
        if self.current > 1 {
            self.current -= 1;
            self.update_curr_str();
        }
    }

    fn next(&mut self) {
        if self.current > 0 && self.current < self.total {
            self.current += 1;
            self.update_curr_str();
        }
    }

    fn update_curr_str(&mut self) {
        self.curr_str.clear();
        write_str!(self.curr_str, "{} / {}", self.current, self.total);
    }

    #[inline]
    fn idx(&self) -> usize {
        self.current - 1
    }
}


fn draw_errors_ui(gl: &mut Gl, uic: &mut UiContext, buf: &mut Buffers, errors: &[ErrorBuf]) {
    background(gl, uic);

    const PREV: u64 = 1;
    uic.button(PREV)
        .position(5.0, 5.0)
        .dimensions(70.0, 30.0)
        .label("Prev")
        .label_font_size(18)
        .callback(|| buf.prev())
        .draw(gl);

    uic.label(&*buf.curr_str)
        .position(95.0, 7.0)
        .size(18)
        .draw(gl);

    const NEXT: u64 = PREV + 1;
    uic.button(NEXT)
        .right_from(PREV, 120.0)
        .dimensions(70.0, 30.0)
        .label("Next")
        .label_font_size(18)
        .callback(|| buf.next())
        .draw(gl);

    const DONE: u64 = NEXT + 1;
    uic.button(DONE)
        .right_from(NEXT, 40.0)
        .dimensions(70.0, 30.0)
        .label("Done")
        .label_font_size(18)
        .callback(|| buf.exit = true)
        .draw(gl);

    uic.label("Image:")
            .position(5.0, 40.0)
            .size(20)
            .draw(gl);

    uic.label("Message:")
            .position(5.0, 90.0)
            .size(20)
            .draw(gl);


    const LINE_X: f64 = 10.0;
    const PATH_POS: [f64; 2] = [10.0, 65.0];

    let mut line_y: f64 = 115.0;

    if let Some(error) = errors.get(buf.idx()) {
        uic.label(&*error.path)
            .point(PATH_POS)
            .size(18)
            .draw(gl);

        for line in error.message_lines.iter() {
            uic.label(&**line)
                .position(LINE_X, line_y)
                .size(18)
                .draw(gl);

            line_y += 25.0;
        }
    } else {
        uic.label("N/A")
            .point(PATH_POS)
            .size(18)
            .draw(gl);

        uic.label("N/A")
            .position(LINE_X, line_y)
            .size(18)
            .draw(gl);
    }
}

pub struct ErrorBuf {
    path: String,
    message_lines: Vec<String>,
}

impl ErrorBuf {
    fn new(error: ProcessingError, relative_to: &Path) -> ErrorBuf {
        ErrorBuf {
            path: error.relative_path(relative_to).display().to_string(),
            message_lines: lines(&*error.err_msg(), 80),
        }
    }

    pub fn arc_vec(errors: Vec<ProcessingError>, relative_to: &Path) -> Arc<Vec<ErrorBuf>> {
        Arc::new(errors.into_iter().map(|error| ErrorBuf::new(error, relative_to)).collect())
    }
}

fn lines(parent: &str, line_len: usize) -> Vec<String> {
    use std::cmp::min;
    use std::borrow::ToOwned;

    let mut strs = Vec::new();

    let mut start = 0usize;
    let len = parent.len();

    while start < len {
        let end = min(start + line_len, len);
        strs.push(parent.slice(start, end).to_owned());
        start += end;
    }

    strs
}
