#![feature(convert, path_relative_from)]

extern crate img_dup_common as common;
extern crate getopts;

use std::convert::From;
use std::env;
use std::str::FromStr;

mod dedup;
mod scan;

#[derive(Copy)]
enum Action {
	Scan,
	Dedup,
    List,
    Help,
}

impl Action {
	fn execute<I: Iterator<Item=String>>(self, args: I) {
		match self {
			Action::Scan => scan::execute(args),
			Action::Dedup => dedup::execute(args),
            Action::Help => print_usage(args),
            Action::List => Action::list(),
		}
	}

    fn print_usage(self) {
        match self {
            Action::Scan => scan::print_usage(),
            Action::Dedup => dedup::print_help(),
            Action::Help => println!("{}", "Get help for an action."),
            Action::List => println!("{}", "List available actions."),
        }
    }

    fn list() {
        println!("{}", "Supported actions: scan dedup");
    }
}

static CALLING_FORMAT: &'static str = "Expected: `img_dup {action} [options]`";

impl FromStr for Action {
	type Err = String;

	fn from_str(action: &str) -> Result<Self, String> {
		let action = match action.trim() {
			"scan" => Action::Scan,
			"dedup" => Action::Dedup,
            "help" => Action::Help,
            "list" => Action::List,
			unk => return Err(format!("unknown action: {:?}", unk))
		};

		Ok(action)
	}
}

fn main() {
	// The first argument is the executable name
	let mut args = env::args().skip(1);
	
	let action = match args.next() {
        Some(action) => action.parse::<Action>(),
        None => {
            println!("{}", CALLING_FORMAT);
            return;
        },
	};
	
	match action {
		Ok(action) => action.execute(args),
		Err(msg) => {
            println!("Error: {}", msg);
            Action::list();
        }
	}	
}

fn print_usage<I: Iterator<Item=String>>(mut args: I) {
    let action = match args.next() {
        Some(action) => action.parse::<Action>(),
        None => {
            println!("{}", "Expected: `img_dup help {action}`");
            return;
        }
    };

    match action {
        Ok(action) => action.print_usage(),
        Err(msg) => {
            println!("{}", msg);
            return;
        }
    }    
}

pub enum GetOptResult<T> {
    Some(T),
    None,
    Err(String),
}

impl<T> GetOptResult<T> {
    fn map<U, F>(self, f: F) -> GetOptResult<U> where F: FnOnce(T) -> U {
        use GetOptResult::*;

        match self {
            Some(val) => Some(f(val)),
            None => None,
            Err(msg) => Err(msg),
        }
    }

    fn and_then<U, F>(self, f: F) -> GetOptResult<U> where F: FnOnce(T) -> GetOptResult<U> {
        use GetOptResult::*;

        match self {
            Some(val) => f(val),
            None => None,
            Err(msg) => Err(msg),
        }
    }
}

impl<T> From<Option<T>> for GetOptResult<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(val) => GetOptResult::Some(val),
            None => GetOptResult::None,
        }
    }
}

impl<T> Into<Option<T>> for GetOptResult<T> {
    fn into(self) -> Option<T> {
        match self {
            GetOptResult::Some(val) => Some(val),
            _ => None,
        }
    }
}
