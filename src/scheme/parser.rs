use crate::lexer::Token;
use super::beam::*;
use super::symbol::*;
use crate::utils::tree;
use crate::utils::tree::{TreeNode, NodeZipper};

use std::rc::Rc;
use std::cell::RefCell;

pub type Node = tree::Node<Token>;

pub fn parse(src: &[Token]) -> Result<Value, RuntimeError> {
    let mut tree = Node::new(Token {
        type_: "LGroup".to_string(),
        value_: "(".to_string(),
        line_: 0
    }).zipper();
    for token in src {
        // println!("Parsing : {:}", token);
        if tree.node.len() == 1 {
            match tree.node.value.type_.as_str() {
                "Symbolize" | "Template" | "Comma" | "Comma_Splicing" | "Dot" => {
                    tree = tree.parent();
                },
                _ => {}
            }
        }
        match token.type_.as_str() {
            "LGroup" | "VGroup" => {
                tree.node.push(Node::new(token.clone()));
                let pos = tree.node.len() - 1;
                tree = tree.child(pos);
            },
            "Symbolize" | "Template" | "Comma" | "Comma_Splicing" | "Dot" => {
                tree.node.push(Node::new(token.clone()));
                let pos = tree.node.len() - 1;
                tree = tree.child(pos);
            },
            "RGroup" => {
                if tree.check_parent() {
                    tree = tree.parent();
                } else { error!("Unexpected )")? }
            },
            _ => {
                tree.node.push(Node::new(token.clone()));
            }
        }
    }
    let node = tree.finish();
    parse_datum(node)
    // println!("LIST: {:?}", dumplist);
    // match *dumplist.clone().borrow() {
    //     Datum::Pair(ref car, ref cdr) => Ok(car.clone()),
    //     Datum::Nil => Ok(SymbolTable::nil()),
    //     _ => Err(())
    // }
}

pub fn parse_datum(mut src: Node) -> Result<Value, RuntimeError> {
    Ok(match src.value.type_.as_str() {
        // "Eval" => Datum::SpecialForm(SpecialForm::Eval),
        // "Apply" => Datum::SpecialForm(SpecialForm::Apply),
        // "Begin" => Datum::SpecialForm(SpecialForm::Begin),
        // "Define" => Datum::SpecialForm(SpecialForm::Define),
        // "DefineSyntax" => Datum::SpecialForm(SpecialForm::DefineSyntax),
        // "Lambda" => Datum::SpecialForm(SpecialForm::Lambda),
        // "Set" => Datum::SpecialForm(SpecialForm::Set),
        // "Let" => Datum::SpecialForm(SpecialForm::Let),
        // "Letstar" => Datum::SpecialForm(SpecialForm::Letstar),
        // "Letrec" => Datum::SpecialForm(SpecialForm::Letrec),
        // "And" => Datum::SpecialForm(SpecialForm::And),
        // "Or" => Datum::SpecialForm(SpecialForm::Or),
        // "Cond" => Datum::SpecialForm(SpecialForm::Cond),
        // "If" => Datum::SpecialForm(SpecialForm::If),
        // "Quote" => Datum::SpecialForm(SpecialForm::Quote),
        "Number" => SymbolTable::number(src.value.value_.parse().or(error!("Error parsing {:?}", src))?),
        "String" => SymbolTable::string(src.value.value_[1..src.value.value_.len()-1].to_string()),
        "Boolean" => SymbolTable::bool(src.value.value_.as_str() == "#t"),
        "Character" => SymbolTable::character(match src.value.value_.as_str() {
            "#\\newline" => '\n',
            "#\\space" => ' ',
            _ => src.value.value_.chars().skip(2).next().ok_or(error!("Error when parsing char")?)?
        }),
        "LGroup" => if src.len() == 0 { SymbolTable::nil() }
        else if src.len() == 2 && src.childs[1].value.type_.as_str() == "Dot" {
            Datum::Pair(parse_datum(src.childs.remove(0))?, parse_datum(src.childs.remove(0))?).wrap()
        } else {
            Datum::Pair(parse_datum(src.childs.remove(0))?, parse_datum(src)?).wrap()
        },
        "Dot" => parse_datum(src.childs.remove(0))?,
        // "VGroup" => Datum::Vector(src.childs.into_iter().map(|x| parse_datum(x)).collect()),
        "Symbolize" => Datum::Pair(SymbolTable::symbol("quote"), List::one(parse_datum(src.clone().childs.into_iter().next().ok_or(()).or(error!("Error parsing quote : {:?}", src))?)?).into()).wrap(),
        "Template" => Datum::Pair(SymbolTable::symbol("quasiquote"), List::one(parse_datum(src.childs.into_iter().next().ok_or(()).or(error!("Error parsing {:?}", "Quasiquote"))?)?).into()).wrap(),
        "Comma" => Datum::Pair(SymbolTable::symbol("unquote"), List::one(parse_datum(src.childs.into_iter().next().ok_or(()).or(error!("Error parsing {:?}", "Unquote"))?)?).into()).wrap(),
        "Comma_Splicing" => Datum::Pair(SymbolTable::symbol("unquote-splicing"), List::one(parse_datum(src.childs.into_iter().next().ok_or(()).or(error!("Error parsing {:?}", "UnquoteSplicing")?)?)?).into()).wrap(),
        _ => SymbolTable::symbol(src.value.value_)
    })
}