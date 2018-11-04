#[macro_use]
extern crate coolc;

use coolc::utils::*;
use coolc::lexer;

use coolc::parser::*;
use coolc::parser::grammar::*;
use coolc::lexer::Token;
use argparse::{ArgumentParser, Store, StoreTrue, StoreConst};

use std::fs::File;
use std::io::{BufWriter, Write};


macro_rules! DEBUG {
    () => { unsafe{ DEBUG } };
}

macro_rules! VERBOSE {
    () => { unsafe{ VERBOSE } };
}


#[derive(Debug, Clone, PartialEq, Copy)]
enum SupportedParsers {
    LL, LR, RD, LALR, GLR
}

fn main() {

    let mut use_parser: Option<SupportedParsers> = Some(SupportedParsers::RD);
    let mut debug = false;
    let mut verbose = false;
    let mut test = false;
    let mut do_lexer = false;
    let mut lexer_grammar_config = get_env_var("PARSER_GRAMMAR_LEXER_CONFIG", "ebnf.lex");
    let mut lexer_input_config = get_env_var("PARSER_LEXER_CONFIG", "examples/cool/cool.lex");
    let mut parser_config = get_env_var("PARSER_CONFIG", "examples/cool/cool.ebnf");
    let mut input_file = String::new();
    let mut lexer_input_model = get_env_var("PARSER_LEXMODEL", "");
    let mut lexer_output_model = String::new();
    let mut lexer_input_tokens = get_env_var("PARSER_TOKENS", "");
    let mut input_model = get_env_var("PARSER_LRTABLE", "");
    let mut output_model = String::new();
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
        ap.refer(&mut do_lexer)
            .add_option(&["--lexer"], StoreTrue, "Output lexical result");
        ap.refer(&mut use_parser)
            .add_option(&["--rd"], StoreConst(Some(SupportedParsers::RD)), "Using RD Parser")
            .add_option(&["--ll"], StoreConst(Some(SupportedParsers::LL)), "Using LL(1) Parser")
            .add_option(&["--lr"], StoreConst(Some(SupportedParsers::LALR)), "Using LALR(1) Parser")
            .add_option(&["--lr0"], StoreConst(Some(SupportedParsers::LR)), "Using LR(0) Parser")
            .add_option(&["--glr"], StoreConst(Some(SupportedParsers::GLR)), "Using GLR(1) Parser");
        ap.refer(&mut lexer_grammar_config)
            .add_option(&["-e", "--ebnfconfig"], Store, "EBNF lexer specfication file");
        ap.refer(&mut lexer_input_config)
            .add_option(&["-x", "--lexerconfig"], Store, "Source lexer specfication file");    
        ap.refer(&mut parser_config)
            .add_option(&["-c", "--config"], Store, "Syntax specfication file");
        ap.refer(&mut input_model)
            .add_option(&["-l", "--load"], Store, "Input graph model file");
        ap.refer(&mut output_model)
            .add_option(&["-s", "--save"], Store, "Output graph model file");
        ap.refer(&mut lexer_input_model)
            .add_option(&["--loadlexer"], Store, "Input lexer model file");
        ap.refer(&mut lexer_output_model)
            .add_option(&["--savelexer"], Store, "Output lexer model file");
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

    /* Initilize Parser */
    let rules = lexer::read_config(&lexer_grammar_config).expect(&format!("Cannot open file: {:} as PARSER_GRAMMAR_LEXER_CONFIG", lexer_grammar_config));
    let tokens: Vec<Token> = lexer::tokenize(read_file(&parser_config).expect(&format!("Cannot open file: {:} as PARSER_CONFIG", parser_config)).as_str(), &rules).unwrap()
        .into_iter()
        .map(|mut t| {if t.type_.as_str()=="Token" { t.value_ = t.value_[1..t.value_.len()-1].to_string() } t})
        .map(|mut t| {if t.type_.as_str()=="SpecialSequence" { t.value_ = t.value_[1..t.value_.len()-1].to_string() } t})
        .collect();

    let mut grammar = Grammar::parse(&tokens).unwrap();

    /********************
     * User Custom Code *
     ********************/

    /* Initilize Lexer */
    /* Get Tokens */

    let input_tokens: Vec<Token> = if lexer_input_tokens.is_empty() {
        let input_lexer_rules = if lexer_input_model.is_empty() { lexer::read_config(lexer_input_config.as_str()).expect(&format!("Cannot open file: {:} as PARSER_LEXER_CONFIG", lexer_input_config)) } 
            else { serde_yaml::from_str(&read_file(lexer_input_model.as_str()).expect(&format!("Cannot open file: {:} as PARSER_LEXER_CONFIG", lexer_input_model))).expect("Deserialize error") };
        if !lexer_output_model.is_empty() { write_file(lexer_output_model.as_str(), serde_yaml::to_string(&input_lexer_rules).expect("Serialize error")).unwrap(); }
        lexer::tokenize(read_file(&input_file).expect("Cannot open source file").as_str(), &input_lexer_rules).unwrap()
    } else {
        serde_yaml::from_str(&read_file(lexer_input_tokens.as_str()).expect(&format!("Cannot open file: {:} as PARSER_TOKENS", lexer_input_tokens))).expect("Deserialize error")
    };

    if do_lexer {
        for (i, token) in input_tokens.iter().enumerate() {
            println!("${:} {:}", i, token);
        }
    }

    if DEBUG!() { println!("\n{:}", grammar.productions.iter().map(|x| x.dump()).collect::<Vec<String>>().join("\n")); }
    grammar = transform::convert_to_formal_grammar(grammar);
    if DEBUG!() { println!("\n{:}", grammar.productions.iter().map(|x| x.dump()).collect::<Vec<String>>().join("\n")); }
    // grammar = transform::elimate_undirect_left_recursion(grammar);
    // if DEBUG!() { println!("\n{:}", grammar.productions.iter().map(|x| x.dump()).collect::<Vec<String>>().join("\n")); }
    // grammar = transform::elimate_left_recursion(grammar);
    // if DEBUG!() { println!("\n{:}", grammar.productions.iter().map(|x| x.dump()).collect::<Vec<String>>().join("\n")); }

    /* Parsing */
    let mut ret = match use_parser {
        Some(SupportedParsers::LL) => {
            grammar = transform::elimate_left_recursion(grammar);
            if DEBUG!() { println!("\n{:}", grammar.productions.iter().map(|x| x.dump()).collect::<Vec<String>>().join("\n")); }
            grammar = transform::left_factor(grammar);
            if DEBUG!() { println!("\n{:}", grammar.productions.iter().map(|x| x.dump()).collect::<Vec<String>>().join("\n")); }
            let n = match llparser::parse(&input_tokens, &grammar) {
                Ok(n) => n,
                Err(errs) => {
                    for err in errs {
                        println!("Line {:} : {:}", input_tokens[err.index].line_, err.msg);
                    }
                    println!("Compiling aborted");
                    return;
                },
            };
            transform::retrieve_unwrap(n)
        },
        Some(SupportedParsers::RD) => {
            grammar = transform::elimate_left_recursion(grammar);
            if DEBUG!() { println!("\n{:}", grammar.productions.iter().map(|x| x.dump()).collect::<Vec<String>>().join("\n")); }
            (&grammar.productions[0] as &rdparser::Parser).parse(&input_tokens, &grammar).unwrap()
        },
        Some(SupportedParsers::LALR) => {
            // let graph = if input_model.is_empty() { lrparser::construct_lalr_1(&grammar) } else { serde_yaml::from_str(&read_file(input_model.as_str()).unwrap()).expect("Deserialize error") };
            let table = if input_model.is_empty() { lrparser::construct_table(&lrparser::construct_lalr_1(&grammar)).unwrap() } else { serde_yaml::from_str(&read_file(input_model.as_str()).expect(&format!("Cannot open file: {:} as PARSER_LRTABLE", input_model))).expect("Deserialize error") };
            if !output_model.is_empty() { write_file(output_model.as_str(), serde_yaml::to_string(&table).expect("Serialize error")).unwrap(); }
            let n = lrparser::parse_with_table(&input_tokens, &table).unwrap();
            transform::retrieve_unwrap(n)
        },
        Some(SupportedParsers::GLR) => {
            let graph = if input_model.is_empty() { lrparser::construct_lalr_1(&grammar) } else { serde_yaml::from_str(&read_file(input_model.as_str()).expect(&format!("Cannot open file: {:} as PARSER_LRTABLE", input_model))).expect("Deserialize error") };
            // let table = if input_model.is_empty() { lrparser::construct_table(&lrparser::construct_lr_0(&grammar)).unwrap() } else { serde_yaml::from_str(&read_file(input_model.as_str()).unwrap()).expect("Deserialize error") };
            if !output_model.is_empty() { write_file(output_model.as_str(), serde_yaml::to_string(&graph).expect("Serialize error")).unwrap(); }
            let n = glrparser::parse_with_graph(&input_tokens, &graph).unwrap();
            transform::retrieve_unwrap(n)
        },
        Some(SupportedParsers::LR) => {
            let graph = if input_model.is_empty() { lrparser::construct_lr_0(&grammar) } else { serde_yaml::from_str(&read_file(input_model.as_str()).expect(&format!("Cannot open file: {:} as PARSER_LRTABLE", input_model))).expect("Deserialize error") };
            // let table = if input_model.is_empty() { lrparser::construct_table(&lrparser::construct_lr_0(&grammar)).unwrap() } else { serde_yaml::from_str(&read_file(input_model.as_str()).unwrap()).expect("Deserialize error") };
            if !output_model.is_empty() { write_file(output_model.as_str(), serde_yaml::to_string(&graph).expect("Serialize error")).unwrap(); }
            let n = lrparser::parse_with_graph(&input_tokens, &graph).unwrap();
            transform::retrieve_unwrap(n)
        },
        _ => panic!("Parser not specfied")
    };



    // ret = transform::retrieve_undirect_left_recursion(ret, &grammar);
    ret = transform::retrieve_left_recursion(ret);

    if output_file.is_empty() {
        print_syntax_tree(&ret, &mut 0, &input_tokens);
    } else if DEBUG!() {
        let write_file = File::create(&output_file).unwrap();
        let mut writer = BufWriter::new(&write_file);
        write_syntax_tree(&ret, &mut 0, &input_tokens, &mut writer);
    } else {
        write_file(output_file.as_str(), serde_yaml::to_string(&ret).unwrap()).unwrap();
    }

    if test {
        let ret = functor::REParser::parse(&ret);
        write_file(output_file.as_str(), serde_yaml::to_string(&ret).unwrap()).unwrap();

    }
    // println!("{:?}", beam::CoolFile::parse(&ret));

}

fn print_syntax_tree(node: &Node, indent: &mut usize, tokens: &[Token]) {
    match &node.value {
        NodeType::Terminal(t) => {
            for _ in 0..*indent { print!(" "); }
            println!("{:?}", t);

            *indent += 2;
            for n in &node.childs {
                print_syntax_tree(n, indent, tokens);
            }
            *indent -= 2;
        },
        NodeType::NonTerminal(t) => {
            for _ in 0..*indent { print!(" "); }
            println!("{:?}", t);

            *indent += 2;
            for n in &node.childs {
                print_syntax_tree(n, indent, tokens);
            }
            *indent -= 2;
        },
        NodeType::List => {
            for _ in 0..*indent { print!(" "); }
            println!("{{");

            *indent += 2;
            for n in &node.childs {
                print_syntax_tree(n, indent, tokens);
            }
            *indent -= 2;

            for _ in 0..*indent { print!(" "); }
            println!("}}");
        },
        NodeType::InnerNode => {
            for _ in 0..*indent { print!(" "); }
            println!("(");

            for n in &node.childs {
                print_syntax_tree(n, indent, tokens);
            }

            for _ in 0..*indent { print!(" "); }
            println!(")");
        },
    }
}


fn write_syntax_tree(node: &Node, indent: &mut usize, tokens: &[Token], f: &mut Write) {
    match &node.value {
        NodeType::Terminal(t) => {
            for _ in 0..*indent { write!(f, " "); }
            writeln!(f, "{:?}", t);

            *indent += 2;
            for n in &node.childs {
                write_syntax_tree(n, indent, tokens, f);
            }
            *indent -= 2;
        },
        NodeType::NonTerminal(t) => {
            for _ in 0..*indent { write!(f, " "); }
            writeln!(f, "{:?}", t);

            *indent += 2;
            for n in &node.childs {
                write_syntax_tree(n, indent, tokens, f);
            }
            *indent -= 2;
        },
        NodeType::List => {
            for _ in 0..*indent { write!(f, " "); }
            writeln!(f, "{{");

            *indent += 2;
            for n in &node.childs {
                write_syntax_tree(n, indent, tokens, f);
            }
            *indent -= 2;

            for _ in 0..*indent { write!(f, " "); }
            writeln!(f, "}}");
        },
        NodeType::InnerNode => {
            for _ in 0..*indent { write!(f, " "); }
            writeln!(f, "(");

            for n in &node.childs {
                write_syntax_tree(n, indent, tokens, f);
            }

            for _ in 0..*indent { write!(f, " "); }
            writeln!(f, ")");
        },
    }
}