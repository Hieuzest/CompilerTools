use crate::lexer::Token;
use super::beam::*;
use super::symbol::SymbolTable;
use crate::utils::tree;
use crate::utils::tree::{TreeNode, NodeZipper};

use std::rc::Rc;
use std::cell::RefCell;

pub type Node = tree::Node<Token>;

pub fn parse(src: &[Token], symtable: &mut SymbolTable) -> Result<Value, ()> {
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
                } else { return Err(()) }
            },
            _ => {
                tree.node.push(Node::new(token.clone()));
            }
        }
    }
    let node = tree.finish();
    let dumplist = parse_datum(node, symtable);
    match Rc::try_unwrap(dumplist).unwrap().into_inner() {
        Datum::Pair(ref car, ref cdr) => Ok(car.clone()),
        Datum::Nil => Ok(Datum::Nil.wrap()),
        _ => Err(())
    }
}

pub fn parse_datum(mut src: Node, symtable: &mut SymbolTable) -> Value {
    match src.value.type_.as_str() {
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
        "Number" => Datum::Number(src.value.value_.parse().expect(&format!("Error parsing {:?}", src))).wrap(),
        "String" => Datum::String(src.value.value_).wrap(),
        "Boolean" => Datum::Boolean(src.value.value_.as_str() == "#t").wrap(),
        "Character" => Datum::Character(match src.value.value_.as_str() {
            "#\\newline" => '\n',
            "#\\space" => ' ',
            _ => src.value.value_.chars().skip(2).next().expect("Error when parsing char")
        }).wrap(),
        "LGroup" => if src.len() == 0 { Datum::Nil.wrap() }
        else if src.len() == 2 && src.childs[1].value.type_.as_str() == "Dot" {
            Datum::Pair(parse_datum(src.childs.remove(0), symtable), parse_datum(src.childs.remove(0), symtable)).wrap()
        } else {
            Datum::Pair(parse_datum(src.childs.remove(0), symtable), parse_datum(src, symtable)).wrap()
        },
        "Dot" => parse_datum(src.childs.remove(0), symtable),
        // "VGroup" => Datum::Vector(src.childs.into_iter().map(|x| parse_datum(x)).collect()),
        "Symbolize" => Datum::Abbreviation(AbbrevPrefix::Quote, parse_datum(src.childs.into_iter().next().expect(&format!("Error parsing {:?}", "Quote")), symtable)).wrap(),
        "Template" => Datum::Abbreviation(AbbrevPrefix::Quasiquote, parse_datum(src.childs.into_iter().next().expect(&format!("Error parsing {:?}", "Template")), symtable)).wrap(),
        "Comma" => Datum::Abbreviation(AbbrevPrefix::Unquote, parse_datum(src.childs.into_iter().next().expect(&format!("Error parsing {:?}", "Comma")), symtable)).wrap(),
        "Comma_Splicing" => Datum::Abbreviation(AbbrevPrefix::UnquoteSplicing, parse_datum(src.childs.into_iter().next().expect(&format!("Error parsing {:?}", "CommaS")), symtable)).wrap(),
        _ => symtable.get(src.value.value_)
    }
}