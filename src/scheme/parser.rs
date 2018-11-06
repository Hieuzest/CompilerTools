use crate::lexer::Token;
use super::beam::*;
use crate::utils::tree;
use crate::utils::tree::{TreeNode, NodeZipper};

pub type Node = tree::Node<Token>;

pub fn parse(src: &[Token]) -> Result<Datum, ()> {
    let mut tree = Node::new(src[0].clone()).zipper();
    for token in src {
        // println!("Parsing : {:}", token);
        if tree.node.len() == 1 {
            match tree.node.value.type_.as_str() {
                "Symbolize" | "Template" | "Comma" | "Comma_Splicing" => {
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
            "Symbolize" | "Template" | "Comma" | "Comma_Splicing" => {
                tree.node.push(Node::new(token.clone()));
                let pos = tree.node.len() - 1;
                tree = tree.child(pos);
            },
            "RGroup" => {
                tree = tree.parent();
            },
            _ => {
                tree.node.push(Node::new(token.clone()));
            }
        }
    }
    let node = tree.finish();
    let dumplist = parse_datum(node);
    if let Datum::List(mut l) = dumplist { Ok(l.remove(0)) } else { Err(()) }
}

pub fn parse_datum(src: Node) -> Datum {
    match src.value.type_.as_str() {
        "Eval" => Datum::SpecialForm(SpecialForm::Eval),
        "Apply" => Datum::SpecialForm(SpecialForm::Apply),
        "Begin" => Datum::SpecialForm(SpecialForm::Begin),
        "Define" => Datum::SpecialForm(SpecialForm::Define),
        "DefineSyntax" => Datum::SpecialForm(SpecialForm::DefineSyntax),
        "Lambda" => Datum::SpecialForm(SpecialForm::Lambda),
        "Set" => Datum::SpecialForm(SpecialForm::Set),
        "Let" => Datum::SpecialForm(SpecialForm::Let),
        "Letstar" => Datum::SpecialForm(SpecialForm::Letstar),
        "Letrec" => Datum::SpecialForm(SpecialForm::Letrec),
        "And" => Datum::SpecialForm(SpecialForm::And),
        "Or" => Datum::SpecialForm(SpecialForm::Or),
        "Cond" => Datum::SpecialForm(SpecialForm::Cond),
        "If" => Datum::SpecialForm(SpecialForm::If),
        "Quote" => Datum::SpecialForm(SpecialForm::Quote),
        "Number" => Datum::Number(src.value.value_.parse().expect(&format!("Error parsing {:?}", src))),
        "String" => Datum::String(src.value.value_),
        "Boolean" => Datum::Boolean(src.value.value_.as_str() == "#t"),
        "Character" => Datum::Character(src.value.value_.chars().next().expect(&format!("Error parsing {:?}", src))),
        "LGroup" => Datum::List(src.childs.into_iter().map(|x| parse_datum(x)).collect()),
        "VGroup" => Datum::Vector(src.childs.into_iter().map(|x| parse_datum(x)).collect()),
        "Symbolize" => Datum::Abbreviation(AbbrevPrefix::Quote, Box::new(parse_datum(src.childs.into_iter().next().expect(&format!("Error parsing {:?}", "Quote"))))),
        "Template" => Datum::Abbreviation(AbbrevPrefix::Template, Box::new(parse_datum(src.childs.into_iter().next().expect(&format!("Error parsing {:?}", "Template"))))),
        "Comma" => Datum::Abbreviation(AbbrevPrefix::Comma, Box::new(parse_datum(src.childs.into_iter().next().expect(&format!("Error parsing {:?}", "Comma"))))),
        "Comma_Splicing" => Datum::Abbreviation(AbbrevPrefix::CommaSplicing, Box::new(parse_datum(src.childs.into_iter().next().expect(&format!("Error parsing {:?}", "CommaS"))))),
        _ => Datum::Symbol(src.value.value_)
    }
}