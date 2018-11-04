use super::*;
use crate::lexer::Token;
use crate::lexer;
use super::utils::*;
use crate::lexer::re::{SingleToken, RegularExpression};
use super::lrparser;
use super::transform;
use std::env;

pub struct REParser {

}

impl REParser {

    fn parse_char(s: &String) -> SingleToken {
        let mut cs = s.chars();
        let t1 = cs.next().unwrap();
        if t1 == '\\' {
            let t2 = cs.next().unwrap();
            match t2 {
			    't' => '\t',
			    'n' => '\n',
			    'r' => '\r',
                '0' => '\0',
                _ => t2
            }
        } else { t1 }
    }

    pub fn parse_from_str(s: &str) -> Result<RegularExpression, ()> {
        let lexer_input_model = get_env_var("RE_LEXMODEL", "examples/re/re.lexmodel");
        let input_model = get_env_var("RE_LRTABLE", "examples/re/re.lrtable");

        let input_tokens: Vec<Token> = {
            let input_lexer_rules = serde_yaml::from_str(&read_file(lexer_input_model.as_str()).expect(&format!("Cannot open file: {:} as RE_LEXMODEL", lexer_input_model))).expect("Deserialize error");
            lexer::tokenize(s, &input_lexer_rules).unwrap()
        };


        let table = serde_yaml::from_str(&read_file(input_model.as_str()).expect(&format!("Cannot open file: {:} as RE_LRTABLE", input_model))).expect("Deserialize error");
        let n = lrparser::parse_with_table(&input_tokens, &table).unwrap();
        let n = transform::retrieve_unwrap(n);
        REParser::parse(&n)
    }

    pub fn parse(node: &Node) -> Result<RegularExpression, ()> {
        match &node.value {
            NodeType::Terminal(Token { type_, value_, ..}) => {
                let c = REParser::parse_char(&value_);
                if c == '\0' {
                    return Ok(RegularExpression::Epsilon)
                } else { return Ok(RegularExpression::Atomic { id: c }) }
            },
            NodeType::NonTerminal(NonTerminal { type_, value_, .. }) => {
                match type_.as_str() {
                    "RegularExpression" => {
                        let ret = RegularExpression::Union {
                            operands: node.childs
                                .iter()
                                .filter_map(|x| if let NodeType::NonTerminal(_) = x.value { Some(x) } else { None })
                                .map(|x| REParser::parse(x).unwrap())
                                .collect()
                        };
                        return Ok(ret);
                    },
                    "Alternative" => {
                        let ret = RegularExpression::Concatenation {
                            operands: node.childs
                                .iter()
                                .filter_map(|x| if let NodeType::NonTerminal(_) = x.value { Some(x) } else { None })
                                .map(|x| REParser::parse(x).unwrap())
                                .collect()
                        };
                        return Ok(ret);
                    },
                    "Kleen" => {
                        let ret = if node.childs.len() > 1 {
                            RegularExpression::Iteration {
                                operand: Box::new(REParser::parse(&node.childs[0]).unwrap())
                            }
                        } else {
                            REParser::parse(&node.childs[0])?
                        };
                        return Ok(ret);
                    },
                    "Term" => match value_.as_str() {
                        "match" => {
                            let ret = RegularExpression::Match {
                                operand: Box::new(REParser::parse(&node.childs[1]).unwrap())
                            };
                            return Ok(ret);
                        },
                        "group" => {
                            return REParser::parse(&node.childs[1]);
                        },
                        "alias" => {
                            let ret = RegularExpression::Alias {
                                id: if let NodeType::Terminal(Token { value_, .. }) = &node.childs[0].value { value_[1..value_.len()-1].to_string() } else { return Err(()) }
                            };
                            return Ok(ret);                            
                        },
                        "char" | "charn" | "charr" => {
                            return REParser::parse(&node.childs[0]);
                        },
                        "chargroup" => {
                            let mut set: Vec<SingleToken> = Vec::new();
                            let mut flag_neg = false;
                            for n in &node.childs {
                                if let NodeType::NonTerminal(NonTerminal { value_, .. }) = &n.value {
                                    if flag_neg {
                                        if value_ == "char" {
                                            if let NodeType::Terminal(Token { value_, .. }) = &n.childs[0].value {
                                                if let Some(p) = set.iter().position(|x| x == &REParser::parse_char(&value_)) {
                                                    set.remove(p);
                                                }
                                            }
                                        } else if value_ == "charset" {
                                            let mut neg_set = Vec::new();
                                            let mut curr = if let NodeType::Terminal(Token { value_, .. }) = &n.childs[0].value { REParser::parse_char(&value_) } else { return Err(()) };
                                            let end = if let NodeType::Terminal(Token { value_, .. }) = &n.childs[2].value { REParser::parse_char(&value_) } else { return Err(()) };
                                            while curr <= end {
                                                neg_set.push(curr);
                                                curr = (curr as u8 + 1) as char;
                                            }
                                            set = set.into_iter().filter(|x| !neg_set.contains(x)).collect();
                                        }
                                    } else {
                                        if value_ == "char" {
                                            if let NodeType::Terminal(Token { value_, .. }) = &n.childs[0].value {
                                                set.push(REParser::parse_char(&value_));
                                            }
                                        } else if value_ == "charset" {
                                            let mut curr = if let NodeType::Terminal(Token { value_, .. }) = &n.childs[0].value { REParser::parse_char(&value_) } else { return Err(()) };
                                            let end = if let NodeType::Terminal(Token { value_, .. }) = &n.childs[2].value { REParser::parse_char(&value_) } else { return Err(()) };
                                            while curr <= end {
                                                set.push(curr);
                                                curr = (curr as u8 + 1) as char;
                                            }
                                        }
                                    }
                                } else if let NodeType::Terminal(Token { type_, .. }) = &n.value {
                                    if type_ == "CharNeg" {
                                        flag_neg = true;
                                    }
                                }
                            }
                            let ret = RegularExpression::Union {
                                operands: set.into_iter().map(|x| RegularExpression::Atomic { id: x}).collect()
                            };
                            return Ok(ret);
                        },
                        _ => {}
                    },
                    _ => {}
                }
            },
            _ => ()
        }
        Err(())
    }
}
