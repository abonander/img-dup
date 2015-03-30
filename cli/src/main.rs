extern crate common;

use std::borrow::ToOwned;
use std::env;
use std::io;
use std::str::FromStr;

mod dedup;
mod scan;

#[derive(Copy)]
enum Action {
	Scan,
	Dedup,
}

impl Action {
	fn execute<I: Iterator<Item=String>>(self, args: I) {
		match self {
			Scan => scan::execute(args),
			Dedup => dedup::execute(args),
		}
	}
}

impl FromStr for Action {
	type Err = String;

	fn from_str(action: &str) -> Result<Self, String> {
		let action = match action.trim() {
			"scan" => Action::Scan,
			"dedup" => Action::Dedup,
			unk => return Err(format!("Unknown action: {:?}", unk))
		};

		Some(action)
	}
}

fn main() {
	// The first argument is the executable name
	let mut args = env::args().skip(1);
	
	let action = args.next()
		.map_or_else(
			|arg| arg.parse::<Action>(),
			|| Err("Please enter an action!".to_owned())
		);
	
	match action {
		Ok(action) => action.execute(args),
		Err(msg) => io::println(msg),
	}	
}
 
