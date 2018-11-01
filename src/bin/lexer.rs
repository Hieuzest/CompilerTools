
pub static mut DEBUG: bool = false;
pub static mut VERBOSE: bool = false;

use coolc::utils::*;
use coolc::lexer;

use argparse::{ArgumentParser, Store, StoreTrue};

fn main() {
    // let restr = "[\\ \\r\\n\\t]*";
    // let charmap: Vec<char> = " |\r|\n|\t|;|0|1|2|3|4|5|6|7|8|9|a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|_".split('|').map(|x| x.chars().next().unwrap()).collect();

    // let rexpr = re::RegularExpression::parse(restr, None).unwrap();
    // println!("{:?}\n", rexpr);
    // let dfa = dfa::construct_dfa(&rexpr, &charmap);
    // println!("{:?}", dfa);
    // let nfa = nfa::construct_nfa(&rexpr);
    // println!("{:?}", nfa);
    // println!("{:?}", dfa::nfa_to_dfa(&nfa, &charmap));
    // println!("{:?}", dfa::minimize_dfa(&dfa::nfa_to_dfa(&nfa, &charmap), &charmap));
    // let token = "\r\r\n";
    // println!("{:?}", dfa::match_dfa(&dfa, token));

    let mut debug = false;
    let mut verbose = false;
    let mut config = "cool.lex".to_string();
    let mut input_model = String::new();
    let mut output_model = String::new();
    let mut source = String::new();
    let mut output_file = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("General Lexical Analyzer");
        ap.refer(&mut debug)
            .add_option(&["-d", "--debug"], StoreTrue, "Show debug messages");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Show more info");
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Store, "Lexer specfication file");
        ap.refer(&mut input_model)
            .add_option(&["-l", "--load"], Store, "Input graph model file");
        ap.refer(&mut output_model)
            .add_option(&["-s", "--save"], Store, "Output graph model file");
        ap.refer(&mut output_file)
            .add_option(&["-o", "--output"], Store, "Output file");
        ap.refer(&mut source)
            .add_argument("source", Store, "Source file to process");
        ap.parse_args_or_exit();
    }


    let rules = if input_model.is_empty() { lexer::read_config(config.as_str()) } else { serde_yaml::from_str(&read_file(input_model.as_str()).unwrap()).expect("Deserialize error") };
    if !output_model.is_empty() { write_file(output_model.as_str(), serde_yaml::to_string(&rules).expect("Serialize error")).unwrap(); }
    let tokens = lexer::tokenize(read_file(source.as_str()).expect("Cannot open target file").as_str(), &rules);

    println!("#name \"{:}\"", source);

    if let Ok(tokens) = tokens {
        // println!("{:?}", tokens);
        if !output_file.is_empty() {
            write_file(output_file.as_str(), serde_yaml::to_string(&tokens).unwrap()).unwrap();
        }

        for token in &tokens {
            if verbose {
                println!("{:}", token);
                continue;
            }
            match token.type_.as_str() {
                "OBJECTID" | "TYPEID" | "INT_CONST" | "STR_CONST" | "BOOL_CONST" => println!("#{:} {:} {:}", token.line_, token.type_, token.value_),
                "DARROW" | "ASSIGN" | "CLASS" | "ELSE" | "FI" | "IF" | "IN" | "INHERITS" | "ISVOID" | "LET" | "LOOP" | "POOL" | "THEN" | "WHILE" | "CASE" | "ESAC" | "NEW" | "OF" | "NOT" => println!("#{:} {:}", token.line_, token.type_),
                "OP_LE" => println!("#{:} {:}", token.line_, "LE"),
                "ERROR" if token.value_.starts_with("(*") => println!("#{:} {:} \"{:}\"", token.line_, token.type_, "EOF in comment"),
                "ERROR" => println!("#{:} {:} \"{:}\"", token.line_, token.type_, token.value_),
                // "COMMENTBLOCK" | "WHITESPACE" => (),
                _ => println!("#{:} '{:}'", token.line_, token.value_)
            }
            
        }
    }
//     let ddfa = dfa::construct_dfa(&rexpr, &charmap);
//     println!("{:?}", ddfa);

// // indirect





}