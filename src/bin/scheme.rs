use argparse::{ArgumentParser, Store, StoreTrue};

use coolc::scheme;
use coolc::lexer::Token;
use coolc::lexer;
use coolc::utils::*;


fn main() {


    let mut debug = false;
    let mut verbose = false;
    let mut test = false;
    let mut lexer_input_model = get_env_var("LEXER_MODEL", "");
    let mut lexer_input_tokens = get_env_var("PARSER_TOKENS", "");
    let mut input_file = String::new();
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
        ap.refer(&mut lexer_input_model)
            .add_option(&["--loadlexer"], Store, "Input lexer model file");
        ap.refer(&mut lexer_input_tokens)
            .add_option(&["--loadtokens"], Store, "Input tokens from lexer");
        ap.refer(&mut output_file)
            .add_option(&["-o", "--output"], Store, "Output file");
        ap.refer(&mut input_file)
            .add_argument("input file", Store, "Source file to process");
        ap.parse_args_or_exit();
    }



    let tokens: Vec<Token> = if lexer_input_tokens.is_empty() {
        let rules: Vec<lexer::RegularRule> = serde_yaml::from_str(&read_file(lexer_input_model.as_str()).expect(&format!("Cannot open file: {:} as LEXER_MODEL", lexer_input_model))).expect("Deserialize error");
        lexer::tokenize(read_file(input_file.as_str()).expect("Cannot open source file").as_str(), &rules).unwrap()
    } else {
        serde_yaml::from_str(&read_file(lexer_input_tokens.as_str()).expect(&format!("Cannot open file: {:} as PARSER_TOKENS", lexer_input_tokens))).expect("Deserialize error")
    };

    let program = self::scheme::parser::parse(&tokens).unwrap();
    println!("{:?}", program);

    let ret = self::scheme::engine::eval(&program, &mut self::scheme::env::Enviroment::new());
    println!("Ret: {:?}", ret);
}