pub use crate::utils;
pub mod re;
pub mod nfa;
pub mod dfa;
pub mod graph;


use std::collections::HashMap;
use self::re::*;
use std::fmt;
use std::io;
use crate::utils::*;
use crate::parser::functor::REParser;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    pub type_: String,
    pub value_: String,
    pub line_: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "#{:} {:} {:}", self.line_, self.type_, self.value_)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegularRule {
    pub name: String,
    pub expr: StateTransferGraph,
    pub greedy: bool,
    pub ignore: bool,
}


pub fn tokenize(src: &str, rules: &Vec<RegularRule>) -> Result<Vec<Token>, ()> {
    
    #[derive(Debug)]
    struct MatchProcess {
        rule: usize,
        pos: Option<usize>,
        valid: bool,
    }

    let src = src.to_string() + "\0";

    let mut line = 1;
    let mut tokens = Vec::new();
    let mut buffer = String::new();

    let mut finish = true;

    let reset = || -> Vec<MatchProcess> { (0..rules.len()).map(|i| MatchProcess {
        rule: i,
        pos: None,
        valid: true
    }).collect()};
    let mut last_matched: Vec<MatchProcess> = reset();

    let mut i = 0;
    while i < src.len() {
        let c = src.get(i..i+1).unwrap();

        buffer.push_str(c);
        // println!("{:?}", buffer);
        let matched: Vec<MatchProcess> = last_matched.iter().map(|MatchProcess {rule, pos, valid}| {
            if !valid {
                return MatchProcess {
                    rule: *rule,
                    pos: *pos,
                    valid: *valid
                };
            }
            if !rules[*rule].greedy && !(pos == &None) {
                return MatchProcess {
                    rule: *rule,
                    pos: *pos,
                    valid: false
                };
            }
            let ret = dfa::match_dfa(&rules[*rule].expr, buffer.chars());
            match ret {
                dfa::MatchResult::Err => MatchProcess {
                    rule: *rule,
                    pos: *pos,
                    valid: false
                },
                dfa::MatchResult::Unfinished => MatchProcess {
                    rule: *rule,
                    pos: *pos,
                    valid: *valid
                },
                dfa::MatchResult::Ok => MatchProcess {
                    rule: *rule,
                    pos: Some(i),
                    valid: *valid
                }
            }
        }).collect();
        // println!("cur: {:?}", matched);
        // println!("last: {:?}", last_matched);

        finish =
        if let Some(_) = matched.iter().find(|MatchProcess {rule, pos, valid}| *valid) {
            if c == "\n" { line += 1; }
            i += 1;
            false
        } else if let Some(MatchProcess {rule, pos, ..}) = last_matched.iter().filter(|MatchProcess {pos, ..}|  !(pos == &None)).max_by(|MatchProcess {rule: rx, pos: px, ..}, MatchProcess {rule: ry, pos: py, ..}| px.unwrap().cmp(&py.unwrap()).then(ry.cmp(rx))) {
            // buffer.pop();
            // println!("{:?} {:?} {:?}", buffer.len(), i, pos);
            let token = Token {
                type_: rules[*rule].name.clone(),
                value_: buffer[0..(buffer.len() + pos.unwrap() - i)].to_string(),
                line_: line,
            };
            if !rules[*rule].ignore { tokens.push(token); }
            buffer.clear();
            i = pos.unwrap() + 1;
            true
        } else {
            if c == "\n" { line += 1; }
            if finish {
                i += 1;
            } else {
                buffer.pop();
            }
            if buffer != "\0" {
                // println!("{:?}", last_matched);
                // println!("len: {:}  [{:}]", buffer.len(), buffer);
                let token = Token {
                    type_: String::from("ERROR"),
                    value_: buffer.clone(),
                    line_: line,
                };
                tokens.push(token);
            }
            buffer.clear();
            true
        };
        if finish { last_matched = reset(); }
        else { last_matched = matched; }
    }

    Ok(tokens)
}

pub fn read_config(path: &str) -> Result<Vec<RegularRule>, io::Error> {
    let string = read_file(path)?;
    let mut rules = Vec::new();
    let configs: Vec<&str> = string.split('\n').into_iter().collect();

    let charmap: Vec<char> = (0..255).map(|x: u8| x as char).collect();
    // let charmap: Vec<char> = "'| |,|\r|\n|\t|{|}|*|+|/|=|(|)|@|:|.|-|<|>|;|\\|\"|0|1|2|3|4|5|6|7|8|9|a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|_".split('|').map(|x| x.chars().next().unwrap()).collect();
    let mut alias: HashMap<String, RegularExpression> = HashMap::new();
    let mut flag = false;

    for config in configs {
        // println!("Parsing: {:?}", config);
        if config.trim().is_empty() || config.starts_with('#') { continue; }
        if config.starts_with('%') {
            flag = true;
            continue;
        }
        let mut ty = config.split_whitespace().next().unwrap().to_string();
        
        let greedy = if ty.ends_with('?') {
            ty = ty[0..ty.len()-1].to_string();
            false } else { true };

        let ignore = if ty.starts_with('-') {
            ty = ty[1..ty.len()].to_string();
            true } else { false };

        let re = config.split_whitespace().skip(1).collect::<Vec<&str>>().join(" ");
        let re = RegularExpression::parse(&re, &alias).expect(&format!("Unable to resolve regular rule [{:}]", ty));

        // let mut re = REParser::parse_from_str(re.as_str()).expect(&format!("Unable to resolve regular rule [{:}]", ty));
        // re.apply_alias(&alias);


        // println!("{:?}", re);
        if flag {
            // println!("{:?}", dfa::construct_dfa(&re, &charmap));
            rules.push(RegularRule {
                name: ty,
                expr: dfa::construct_dfa(&re, &charmap),
                greedy: greedy,
                ignore: ignore,
            });
        } else {
            alias.insert(ty, re);
        }
    }
    Ok(rules)
}


pub fn read_config_external(path: &str) -> Result<Vec<RegularRule>, io::Error> {
    let string = read_file(path)?;
    let mut rules = Vec::new();
    let configs: Vec<&str> = string.split('\n').into_iter().collect();

    let charmap: Vec<char> = (0..255).map(|x: u8| x as char).collect();
    // let charmap: Vec<char> = "'| |,|\r|\n|\t|{|}|*|+|/|=|(|)|@|:|.|-|<|>|;|\\|\"|0|1|2|3|4|5|6|7|8|9|a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|_".split('|').map(|x| x.chars().next().unwrap()).collect();
    let mut alias: HashMap<String, RegularExpression> = HashMap::new();
    let mut flag = false;

    for config in configs {
        // println!("Parsing: {:?}", config);
        if config.trim().is_empty() || config.starts_with('#') { continue; }
        if config.starts_with('%') {
            flag = true;
            continue;
        }
        let mut ty = config.split_whitespace().next().unwrap().to_string();
        
        let greedy = if ty.ends_with('?') {
            ty = ty[0..ty.len()-1].to_string();
            false } else { true };

        let ignore = if ty.starts_with('-') {
            ty = ty[1..ty.len()].to_string();
            true } else { false };

        let re = config.split_whitespace().skip(1).collect::<Vec<&str>>().join(" ");
        // let re = RegularExpression::parse(&re, &alias).expect(&format!("Unable to resolve regular rule [{:}]", ty));

        let mut re = REParser::parse_from_str(re.as_str()).expect(&format!("Unable to resolve regular rule [{:}]", ty));
        re.apply_alias(&alias);


        // println!("{:?}", re);
        if flag {
            // println!("{:?}", dfa::construct_dfa(&re, &charmap));
            rules.push(RegularRule {
                name: ty,
                expr: dfa::construct_dfa(&re, &charmap),
                greedy: greedy,
                ignore: ignore,
            });
        } else {
            alias.insert(ty, re);
        }
    }
    Ok(rules)
}

