use super::prelude::*;

pub fn confirm(title: &str, line_1: &str, line_2: &str) -> bool {
	let (mut uic, mut gl, mut events) = create_window(title, [300, 90]);

	let ref mut answer = ConfirmResponse::Waiting;

	for event in events {
		if *answer != ConfirmResponse::Waiting {
			return *answer == ConfirmResponse::Yes;
		}

		uic.handle_event(&event);
		match event {
			Event::Render(args) => {
				gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
					draw_confirm_dialog(gl, &mut uic, line_1, line_2, answer);
				});
			},
			_ => (),
		}
	}

	false
}

fn draw_confirm_dialog(
	gl: &mut Gl, 
	uic: &mut UiContext, 
	line_1: &str, line_2: &str,
	answer: &mut ConfirmResponse,
) {
	background(gl, uic);

	uic.label(line_1)
		.position(5.0, 5.0)
		.size(18)
		.draw(gl);

	uic.label(line_2)
		.position(5.0, 30.0)
		.size(18)
		.draw(gl);

	uic.button(1)
		.label("Yes")
		.position(75.0, 55.0) 
		.dimensions(70.0, 30.0)
		.callback(|| *answer = ConfirmResponse::Yes)
		.draw(gl);

	uic.button(2)
		.label("No")
		.position(150.0, 55.0)
		.dimensions(70.0, 30.0)
		.callback(|| *answer = ConfirmResponse::No)
		.draw(gl);
}

#[deriving(Eq, PartialEq)]
enum ConfirmResponse {
	Waiting,
	Yes,
	No,
}

