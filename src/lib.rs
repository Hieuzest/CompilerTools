#![feature(euclidean_division)]
extern crate argparse;
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate serde_yaml;

pub static mut DEBUG: bool = false;
pub static mut VERBOSE: bool = false;

#[macro_use]
pub mod utils;

pub mod lexer;
pub mod parser;
pub mod cool;
pub mod scheme;