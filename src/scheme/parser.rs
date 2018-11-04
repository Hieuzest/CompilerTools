use crate::lexer::Token;
use super::beam::*;
use crate::utils::tree;
use crate::utils::tree::{TreeNode, NodeZipper};

pub type Node = tree::Node<Token>;

pub fn parse(src: &[Token]) -> Result<Datum, ()> {
    let mut tree = Node::new(src[0].clone()).zipper();
    for token in src {
        match token.type_.as_str() {
            "LGroup" | "VGroup" => {
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
        "Begin" => Datum::SpecialForm(SpecialForm::Begin),
        "Define" => Datum::SpecialForm(SpecialForm::Define),
        "Lambda" => Datum::SpecialForm(SpecialForm::Lambda),
        "Set" => Datum::SpecialForm(SpecialForm::Set),
        "Let" => Datum::SpecialForm(SpecialForm::Let),
        "And" => Datum::SpecialForm(SpecialForm::And),
        "Or" => Datum::SpecialForm(SpecialForm::Or),
        "Cond" => Datum::SpecialForm(SpecialForm::Cond),
        "If" => Datum::SpecialForm(SpecialForm::If),
        "Number" => Datum::Number(src.value.value_.parse().unwrap()),
        "String" => Datum::String(src.value.value_),
        "Boolean" => Datum::Boolean(src.value.value_.as_str() == "#t"),
        "Character" => Datum::Character(src.value.value_.chars().next().unwrap()),
        "LGroup" => Datum::List(src.childs.into_iter().map(|x| parse_datum(x)).collect()),
        "VGroup" => Datum::Vector(src.childs.into_iter().map(|x| parse_datum(x)).collect()),
        _ => Datum::Identifier(src.value.value_)
    }
    
}