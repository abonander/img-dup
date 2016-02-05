use super::prelude::*;

use std::thread::Thread;

pub fn confirm(title: &'static str, message: &'static str) -> bool {
	Thread::spawn(move || {

	    let (mut uic, mut gl, mut events) = create_window(title, [450, 60]);
        let ref mut answer = ConfirmResponse::Waiting;

        for event in events {
            if *answer != ConfirmResponse::Waiting {
                return *answer == ConfirmResponse::Yes;
            }

            uic.handle_event(&event);
            match event {
                Event::Render(args) => {
                    gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
                        draw_confirm_dialog(gl, &mut uic, message, answer);
                    });
                },
                _ => (),
            }
        }

        false
    }).join().unwrap_or(false)
}

fn draw_confirm_dialog(
	gl: &mut Gl,
	uic: &mut UiContext,
	message: &str,
	answer: &mut ConfirmResponse,
) {
	background(gl, uic);

	uic.label(message)
		.position(5.0, 0.0)
		.size(18)
		.draw(gl);

	uic.button(1)
		.label("Yes")
		.position(155.0, 25.0)
		.dimensions(70.0, 30.0)
		.callback(|| *answer = ConfirmResponse::Yes)
		.draw(gl);

	uic.button(2)
		.label("No")
        .right_from(1, 5.0)
		.dimensions(70.0, 30.0)
		.callback(|| *answer = ConfirmResponse::No)
		.draw(gl);
}

#[derive(Eq, PartialEq)]
enum ConfirmResponse {
	Waiting,
	Yes,
	No,
}

