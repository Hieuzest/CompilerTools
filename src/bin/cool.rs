
use coolc::parser::*;
use coolc::cool::beam::*;
use coolc::utils::*;
use argparse::{ArgumentParser, Store, StoreTrue};


fn main() {

    let mut debug = false;
    let mut verbose = false;
    let mut input_file = String::new();
    let mut input_model = String::new();
    let mut output_model = String::new();
    let mut output_file = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Cool Compiler");
        ap.refer(&mut debug)
            .add_option(&["-d", "--debug"], StoreTrue, "Show debug messages");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Show more info");
        ap.refer(&mut input_model)
            .add_option(&["-l", "--load"], Store, "Input graph model file");
        ap.refer(&mut output_model)
            .add_option(&["-s", "--save"], Store, "Output graph model file");
        ap.refer(&mut output_file)
            .add_option(&["-o", "--output"], Store, "Output file");
        ap.refer(&mut input_file)
            .add_argument("input file", Store, "Source file to process");
        ap.parse_args_or_exit();
    }

    unsafe {
        DEBUG = debug;
        VERBOSE = verbose;
    }

    let node: Node = serde_yaml::from_str(&read_file(input_file.as_str()).unwrap()).expect("Deserialize error");


    println!("{:?}", CoolFile::parse(&node));
}
