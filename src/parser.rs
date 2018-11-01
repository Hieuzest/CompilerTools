pub use crate::utils;
pub mod llparser;
pub mod lrparser;
pub mod rdparser;
pub mod grammar;

pub mod tree;
pub mod transform;
pub mod functor;

use crate::lexer::Token;
use self::tree::*;
use self::grammar::*;

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Terminal(Token), NonTerminal(NonTerminal), InnerNode, List
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Node {
    pub value: NodeType,
    pub childs: Vec<Node>,
    pub index: usize
}

#[derive(Debug, Clone, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct NonTerminal {
    pub type_: String,
    pub value_: String,
    pub rule_: Production,
}

impl TreeNode for Node {
    type Data = NodeType;

    fn data(&self) -> &Self::Data {
        &self.value
    }

    fn data_mut(&mut self) -> &mut Self::Data {
        &mut self.value
    }

    fn len(&self) -> usize {
        self.childs.len()
    }

    fn get(&self, index: usize) -> Option<&Self> {
        self.childs.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self> {
        self.childs.get_mut(index)
    }

    fn remove(&mut self, index: usize) -> Self {
        self.childs.remove(index)
    }
    
    fn swap_remove(&mut self, index: usize) -> Self {
        self.childs.swap_remove(index)
    }

    fn push(&mut self, rhs: Self) {
        self.childs.push(rhs)
    }

    fn swap(&mut self, a: usize, b: usize) {
        self.childs.swap(a, b)
    }
}
