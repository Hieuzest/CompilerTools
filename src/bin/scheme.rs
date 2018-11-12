use argparse::{ArgumentParser, Store, StoreTrue};
use coolc::utils::*;
use coolc::scheme;
use coolc::lexer::Token;
use coolc::lexer;

use std::io::*;

fn repl(rules: &Vec<lexer::RegularRule>, env: self::scheme::env::Env, test: bool) {
    // if !test {
    // let code = vec![
    //     "(define (exit))",
    //     "(define (cmds cmd arg . l) (if (null? l) (cmd arg) (begin (cmd arg) (cmds cmd . l))))",
    //     "(begin (define dd display) (define (displays . l) (cmds dd . l)) (set! display displays))",
    //     "(define (error x . l) (define (error-1 x) (begin (display 'err:) (display #\\space) (display x) (display #\\newline) )) (if (null? l) (begin (error-1 x) (exit)) (begin (error-1 x) (error . l))))",
    //     "(define (> x y) (< y x))",
    //     "(define (>= x y) (<= y x))",
    //     "(define (square x) (* x x)",
    //     "(define (ops op intial . l) (define (op-l-i sum x . l) (if (null? l) (op sum x) (op-l-i (op sum x) . l))) (op-l-i intial . l))",
    //     "(begin (define oldop +) (define (op . l) (ops oldop 0 . l)) (set! + op))",
    //     "(begin (define oldop -) (define (op x . l) (ops oldop x . l)) (set! - op))",
    //     "(begin (define oldop *) (define (op . l) (ops oldop 1 . l)) (set! * op))",
    //     "(begin (define oldop /) (define (op x . l) (ops oldop x . l)) (set! / op))",
    // ];


    // for input in code {
    //     let tokens: Vec<Token> = lexer::tokenize(input, &rules).unwrap();
    //     let program = self::scheme::parser::parse(&tokens).unwrap();
    //     self::scheme::engine::eval_begin(program, env.clone()).expect("inner install fail");
    // }
    // }

    loop {
        let mut input = String::new();

        stdout().flush().unwrap();
        print!("> ");
        stdout().flush().unwrap();
        match stdin().read_line(&mut input) {
            Ok(n) if n > 0 => (),
            _ => break
        }
        if input.trim().is_empty() { continue; }

        let tokens: Vec<Token> = lexer::tokenize(input.as_str(), &rules).unwrap();

        // println!("\t{:?}", tokens);

        let program = if let Ok(p) = self::scheme::parser::parse(&tokens) { p }
        else {
            println!("Error: parsing");
            continue;
        };

        // println!("\t{:?}", program.borrow());

        let answer = self::scheme::engine::eval(program.borrow().car().unwrap(), env.clone());

        match answer {
            Ok(value) => println!("=> {:?}", value.borrow()),
            Err(e) => println!("Error: {:?}", e)
        }
        
    }
}


fn main() {
    let mut debug = false;
    let mut verbose = false;
    let mut repl = false;
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
        ap.refer(&mut repl)
            .add_option(&["-r", "--repl"], StoreTrue, "Show more info");
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

    unsafe {
        DEBUG = debug;
        VERBOSE = verbose;
    }

    let env = self::scheme::env::Environment::new();

    let rules: Vec<lexer::RegularRule> = serde_yaml::from_str(&read_file(lexer_input_model.as_str()).expect(&format!("Cannot open file: {:} as LEXER_MODEL", lexer_input_model))).expect("Deserialize error");


    if !input_file.is_empty() {
        let tokens: Vec<Token> = lexer::tokenize(read_file(input_file.as_str()).expect("Cannot open source file").as_str(), &rules).unwrap();

        let program = self::scheme::parser::parse(&tokens).unwrap();
        if debug { println!("{:?}", program); }

        let ret = self::scheme::engine::eval(program.borrow().car().unwrap(), env.clone());
        println!("\nAnswer: {:?}", ret.map(|x| x.borrow().clone()));
    }
    if repl {
        self::repl(&rules, env.clone(), test);
    }
}