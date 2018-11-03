use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::env;

pub use crate::DEBUG;
macro_rules! DEBUG {
    () => { unsafe{ DEBUG } };
}

pub use crate::VERBOSE;
macro_rules! VERBOSE {
    () => { unsafe{ VERBOSE } };
}


pub type VertexSet = Vec<bool>;

macro_rules! new_set {
	($len: expr) => {{
		let mut temp = Vec::with_capacity($len);
		temp.resize($len, false);
		temp
	}};
}

macro_rules! cup_set {
	($lhs: expr, $rhs: expr) => {{
		for i in 0..$rhs.len() {
			if $rhs[i] { $lhs[i] = true }
		}
	}};
}

macro_rules! push_sets {
	($sets: expr) => {{
		let len = $sets.len();
		for set in $sets.iter_mut() {
			set.push(false);
		}
		$sets.push(new_set!(len + 1));
		len + 1
	}};
}

macro_rules! print_indent {
    ($indent: expr) => {
        for _ in 0..$indent {
            print!(" ");
        }  
    };
}


macro_rules! set(
    { $($key:expr),+ } => {
        {
            let mut m = ::std::collections::HashSet::new();
            $(
                m.insert($key);
            )+
            m
        }
     };
);

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

// #[macro_export]
macro_rules! check_default {
	($id: ident, $expr: expr, $default: expr) => {
		let $id = if $id == $expr { $default } else { $id };
	};
	(&$id: ident, $expr: expr, $default: expr) => {
		let $id = if $id == $expr { $default } else { $id.clone() };
	};
}

pub const EPSILON_TOKEN: &'static str = "\0";
pub const FINISH_TOKEN: &'static str = "$";


pub fn get_env_var(varname: &str, default: &str) -> String {
	if let Ok(s) = env::var(varname) { s } else { default.to_string() }
}

pub fn read_file(path: &str) -> Result<String, io::Error> {
	let mut file = File::open(path)?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;
	Ok(contents)
}

pub fn write_file<S: Into<String>>(path: &str, contents: S) -> Result<(), io::Error> {
	let mut file = File::create(path)?;
	file.write_all(contents.into().as_bytes())?;
	Ok(())
}

pub fn print_with_indent<S: Debug>(s: S) {
	let mut indent = 0;
	for c in format!("{:?}", s).chars() {
		match c {
			'{' | '[' | '(' => {
				print!("{:}\n", c);
				indent += 4;
				for _ in 0..indent-1 {
					print!(" ");
				}
			},
			'}' | ']' | ')' => {
				indent -= 4;
				println!();
				for _ in 0..indent {
					print!(" ");
				}
				print!("{:}", c);
			},
			',' => {
				println!(",");
				for _ in 0..indent-1 {
					print!(" ");
				}
			},
			_ => {
				print!("{:}", c)
			}
		}
	}
	println!();
}
