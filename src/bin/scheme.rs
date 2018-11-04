use argparse::{ArgumentParser, Store, StoreTrue};

use coolc::scheme;
use coolc::lexer::Token;
use coolc::utils::*;


fn main() {


    let mut debug = false;
    let mut verbose = false;
    let mut test = false;
    let mut lexer_input_tokens = get_env_var("PARSER_TOKENS", "");
    // let mut input_file = String::new();
    let mut output_file = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("General Syntax Analyzer");
        ap.refer(&mut debug)
            .add_option(&["-d", "--debug"], StoreTrue, "Show debug messages");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Show more info");
        ap.refer(&mut test)
            .add_option(&["--test"], StoreTrue, "Test");
        ap.refer(&mut lexer_input_tokens)
            .add_option(&["--loadtokens"], Store, "Input tokens from lexer");
        ap.refer(&mut output_file)
            .add_option(&["-o", "--output"], Store, "Output file");
        // ap.refer(&mut input_file)
            // .add_argument("input file", Store, "Source file to process");
        ap.parse_args_or_exit();
    }



    let tokens: Vec<Token> = serde_yaml::from_str(&read_file(lexer_input_tokens.as_str()).expect(&format!("Cannot open file: {:} as PARSER_TOKENS", lexer_input_tokens))).expect("Deserialize error");

    let program = self::scheme::parser::parse(&tokens).unwrap();
    println!("{:?}", program);

    let ret = self::scheme::engine::eval(&program, &mut self::scheme::env::Enviroment::new());
    println!("Ret: {:?}", ret);
}