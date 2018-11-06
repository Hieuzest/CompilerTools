use super::prelude::*;


#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    msg: String,
    index: usize,
}

#[derive(Debug, Clone)]
pub struct MetaData {
    n: usize,
    err: usize,
    indent: usize,
}

pub trait ParserInner {
    fn parse(&self, src: &[Token], next: &mut usize, grammar: &Grammar, data: &mut MetaData) -> Result<Node, ParseError>;
}

pub trait Parser {
    fn parse(&self, src: &[Token], grammar: &Grammar) -> Result<Node, ParseError>;
}

impl Parser for Production {
    fn parse(&self, src: &[Token], grammar: &Grammar) -> Result<Node, ParseError> {
        let mut next = 0;
        let mut n = self.expr.parse(src, &mut next, grammar, &mut MetaData {
            n: 0,
            err: 0,
            indent: 0,
        })?;
        if let NodeType::InnerNode = n.value {
            n.value = NodeType::NonTerminal(NonTerminal {
                type_: self.name.clone(),
                value_: self.label.clone(),
                rule_: self.clone(),
                ..Default::default()
            })
        } else {
            panic!("Unreached");
        }
        if DEBUG!() { println!("Production Parsed #{:}-{:} {:}", 0, next, self.name); }
        Ok(n)
    }
}

impl ParserInner for Production {
    fn parse(&self, src: &[Token], next: &mut usize, grammar: &Grammar, data: &mut MetaData) -> Result<Node, ParseError> {
        let ori = *next;
        let mut n = self.expr.parse(src, next, grammar, data)?;
        if let NodeType::InnerNode = n.value {
            n.value = NodeType::NonTerminal(NonTerminal {
                type_: self.name.clone(),
                value_: self.label.clone(),
                rule_: self.clone(),
                ..Default::default()
            })
        } else {
            panic!("Unreached");
        }
        if DEBUG!() { print_indent!(data.indent); println!("Production Parsed #{:}-{:} {:}", ori, next, self.name); }
        Ok(n)
    }
}

impl ParserInner for Expression {
    fn parse(&self, src: &[Token], next: &mut usize, grammar: &Grammar, data: &mut MetaData) -> Result<Node, ParseError> {

        let ori = *next;
        let mut set = Vec::new();
        if DEBUG!() { print_indent!(data.indent); println!("#{:} {:?}", ori, self); }
        for term in &self.terms {
            let t = term.parse(src, next, grammar, data)?;
            if DEBUG!() { print_indent!(data.indent); println!("Parsed Term #{:} {:}-{:?}", ori, next, t); }
            if let NodeType::InnerNode = t.value {
                set.extend(t.childs);
            } else if term.is_unwrap() {
                set.extend(t.childs);
            } else {
                set.push(t);
            }
        }
        Ok(Node {
            value: NodeType::InnerNode,
            childs: set,
            index: ori
        })
    }
}

impl ParserInner for Term {
    fn parse(&self, src: &[Token], next: &mut usize, grammar: &Grammar, data: &mut MetaData) -> Result<Node, ParseError> {
        match self {
            Term::NonTerminal { ref name, .. } => {
                let curr = *next;
                for r in &grammar.productions {
                    *next = curr;
                    if &r.name == name {
                        match (r as &ParserInner).parse(src, next, grammar, data) {
                            Ok(ret) => return Ok(ret),
                            Err(ParseError { index, .. }) => {
                                if index > data.err {
                                    data.err = index;
                                }
                            }
                        }
                    }
                }
                Err(ParseError {
                    msg: format!("No rule matched for {:?}", name),
                    index: data.err
                })
            },
            Term::Terminal { type_: ref ty, value: ref _val, .. } => {
                if let Some(token) = src.get(*next) {
                    if self.match_token(token) {
                        *next += 1;
                        Ok(Node { value: NodeType::Terminal(src.get(*next - 1).unwrap().clone()), childs: Vec::default(), index: *next-1 })
                    } else { Err(ParseError {
                        msg: format!("Expected {:?} but found {:?}", ty, src.get(*next)),
                        index: *next
                    }) }
                } else { Err(ParseError {
                        msg: format!("Cannot read: Expected {:?} but found {:?}", ty, src.get(*next)),
                        index: *next
                    })
                }
            },
            Term::Group { ref expr, .. } => {
                let mut ret = Node {
                    value: NodeType::List,
                    childs: Vec::new(),
                    index: *next
                };
                let n = expr.parse(src, next, grammar, data)?;
                if let NodeType::InnerNode = n.value {
                    ret.childs.extend(n.childs);
                } else {
                    ret.childs.push(n);
                }
                Ok(ret)
            },
            Term::Optional { ref expr, .. } => {
                let mut ret = Node {
                    value: NodeType::List,
                    childs: Vec::new(),
                    index: *next
                };
                if let Ok(n) = expr.parse(src, next, grammar, data) {
                    if let NodeType::InnerNode = n.value {
                        ret.childs.extend(n.childs);
                    } else {
                        ret.childs.push(n);
                    }
                }
                Ok(ret)
            },
            Term::Repetition { ref expr, .. } => {
                let mut ret = Node {
                    value: NodeType::List,
                    childs: Vec::new(),
                    index: *next
                };
                let mut curr = *next;
                while let Ok(n) = expr.parse(src, next, grammar, data) {
                    if let NodeType::InnerNode = n.value {
                        ret.childs.extend(n.childs);
                    } else {
                        ret.childs.push(n);
                    }
                    curr = *next;
                }
                *next = curr;
                Ok(ret)
            },
        }
    }
}