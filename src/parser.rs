pub use crate::utils;
mod prelude;
pub mod llparser;
pub mod lrparser;
pub mod glrparser;
pub mod rdparser;
pub mod grammar;
mod derivations;

pub mod transform;
pub mod functor;

use crate::lexer::Token;
use crate::utils::tree;
use self::grammar::*;


pub type Node = tree::Node<NodeType>;

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Terminal(Token), NonTerminal(NonTerminal), InnerNode, List
}

#[derive(Debug, Clone, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct NonTerminal {
    pub type_: String,
    pub value_: String,
    pub rule_: Production,
}

impl<'a> From<&'a Production> for NonTerminal {
    fn from(rule: &Production) -> Self {
        NonTerminal {
            type_: rule.name.clone(),
            value_: rule.label.clone(),
            rule_: rule.clone()
        }
    }
}
