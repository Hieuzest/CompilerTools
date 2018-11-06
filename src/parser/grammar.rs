use crate::lexer::Token;
use super::transform::Substitution;
use std::cmp::Eq;
use std::fmt;
use std::convert::From;
use std::hash;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Associativity {
    Left, Right
}

#[derive(Debug, Clone, PartialEq, Default, Eq, Hash, Serialize, Deserialize)]
pub struct Grammar {
    pub name: String,
    pub productions: Vec<Production>,
    pub transforms: Vec<Substitution>,
    pub start_symbol: String,
}

#[derive(Debug, Clone, PartialEq, Default, Eq, Hash, Serialize, Deserialize)]
pub struct Production {
    pub name: String,
    pub label: String,
    pub expr: Expression,
    pub precedence: usize,
    pub associativity: Associativity,
}

#[derive(Debug, Clone, PartialEq, Default, Eq, Hash, Serialize, Deserialize)]
pub struct Expression {
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Term {
    NonTerminal {
        name: String,
        unwrap: bool,
    },
    Terminal {
        type_: String,
        value: Option<String>
    },
    Group {
        expr: Box<Expression>,
        unwrap: bool,
    },
    Optional {
        expr: Box<Expression>,
        unwrap: bool,
    },
    Repetition {
        expr: Box<Expression>,
        unwrap: bool,
    },
}

macro_rules! parse_step {
    ($src: expr, $next: expr, $type: expr) => {
        if let Some(&Token{ ref type_, ref value_, .. }) = $src.get(*$next) {
            if type_ == &$type.to_string() {
                *$next += 1;
                Ok(value_.clone())
            } else { Err(ParseError {
                msg: format!("Expected {:?} but found {:?}", $type, $src.get(*$next)),
                index: *$next
            })}
        } else { Err(ParseError {
                msg: format!("Cannot read: Expected {:?} but found {:?}", $type, $src.get(*$next)),
                index: *$next
            })
        }
    };
}


#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    msg: String,
    index: usize,
}

impl Default for Associativity {
    fn default() -> Self {
        Associativity::Left
    }
}

impl Grammar {
    pub fn non_terminals(&self) -> Vec<String> {
        let mut ret = Vec::new();
        for p in &self.productions {
            if !ret.contains(&p.name) {
                ret.push(p.name.clone());
            }
        }
        ret
    }

    pub fn terminals(&self) -> Vec<Term> {
        let mut ret = Vec::new();
        for p in &self.productions {
            for t in &p.expr.terms {
                if let Term::Terminal { .. } = t {
                    if !ret.contains(t) {
                        ret.push(t.clone());
                    }
                }
            }
        }
        ret
    }

    pub fn symbols(&self) -> Vec<Term> {
        self.terminals()
            .into_iter()
            .chain(self.non_terminals()
                .into_iter()
                .map(|t| Term::NonTerminal { name: t, unwrap: false }))
            .collect()
    }

    pub fn get_productions(&self, nt: &String) -> Vec<Production> {
        self.productions.iter().filter(|x| &x.name == nt).map(|x| x.clone()).collect()
    }

    pub fn add_transform(&mut self, tf: Substitution) {
        self.transforms.push(tf);
    }

    pub fn parse(src: &[Token]) -> Result<Self, ParseError> {
        let mut ret = Grammar::default();
        let ref mut next = 0;
        while let Ok(name) = parse_step!(src, next, "ProductionName") {
            let mut i = 0;
            parse_step!(src, next, "Assign")?;
            {
                let mut production = Production::default();
                production.name = name.clone();

                if let Ok(label) = parse_step!(src, next, "SpecialSequence") {
                    production.label = label;            
                }
                if let Ok(t) = parse_step!(src, next, "LeftPrecedence") {
                    production.associativity = Associativity::Left;
                    production.precedence = if t.len() == 2 { 0 } else { t[1..t.len()-1].parse().unwrap() };
                } else if let Ok(t) = parse_step!(src, next, "RightPrecedence") {
                    production.associativity = Associativity::Right;
                    production.precedence = if t.len() == 2 { 0 } else { t[1..t.len()-1].parse().unwrap() };
                }
                if let Ok(label) = parse_step!(src, next, "SpecialSequence") {
                    production.label = label;            
                }

                if production.label.is_empty() { production.label = "#".to_string() + &i.to_string(); }

                production.expr = if let Ok(expr) = Expression::parse_inner(src, next) {
                    expr
                } else {
                    Expression::epsilon()
                };
                ret.productions.push(production);
            }
            while let Ok(_) = parse_step!(src, next, "Alternation") {
                i += 1;
                let mut production = Production::default();
                production.name = name.clone();
                if let Ok(label) = parse_step!(src, next, "SpecialSequence") {
                    production.label = label;            
                }
                if let Ok(t) = parse_step!(src, next, "LeftPrecedence") {
                    production.associativity = Associativity::Left;
                    production.precedence = if t.len() == 2 { 0 } else { t[1..t.len()-1].parse().unwrap() };
                } else if let Ok(t) = parse_step!(src, next, "RightPrecedence") {
                    production.associativity = Associativity::Right;
                    production.precedence = if t.len() == 2 { 0 } else { t[1..t.len()-1].parse().unwrap() };
                }
                if let Ok(label) = parse_step!(src, next, "SpecialSequence") {
                    production.label = label;            
                }

                if production.label.is_empty() { production.label = "#".to_string() + &i.to_string(); }

                production.expr = if let Ok(expr) = Expression::parse_inner(src, next) {
                    expr
                } else {
                    Expression::epsilon()
                };
                ret.productions.push(production);
            }
            parse_step!(src, next, "Terminator")?;
        }
        ret.start_symbol = ret.productions[0].name.clone();
        Ok(ret)
    }
}

impl Production {
    pub fn dump(&self) -> String {
        format!("{:}\t\t=\t?{:}?\t{:}\t{:} .", self.name, self.label, match self.associativity {
            Associativity::Left => format!("|{:} >", self.precedence),
            Associativity::Right => format!("|{:} <", self.precedence),
        },
        self.expr.dump())
    }
    
    // fn parse_inner(src: &[Token], next: &mut usize) -> Result<Self, ParseError> {
    //     let mut ret = Production::default();
    //     ret.name = parse_step!(src, next, "ProductionName")?;
    //     parse_step!(src, next, "Assign")?;
    //     if let Ok(label) = parse_step!(src, next, "SpecialSequence") {
    //         ret.label = label;            
    //     }
    //     ret.expr = if let Ok(expr) = Expression::parse_inner(src, next) {
    //         expr
    //     } else {
    //         Expression::epsilon()
    //     };
    //     parse_step!(src, next, "Terminator")?;
    //     Ok(ret)
    // }

}

impl Expression {

    fn epsilon() -> Self {
        Expression::default()
    }

    fn dump(&self) -> String {
        format!("{:}", self.terms.iter().map(|x| x.dump()).collect::<Vec<String>>().join(" "))
    }

    fn parse_inner(src: &[Token], next: &mut usize) -> Result<Self, ParseError> {
        let mut ret = Expression::default();
        ret.terms.push(Term::parse_inner(src, next)?);
        let mut curr = *next;
        while let Ok(term) = Term::parse_inner(src, next) {
            curr = *next;
            ret.terms.push(term);
        }
        *next = curr;
        Ok(ret)
    }
}

impl Term {

    fn dump(&self) -> String {
        match self {
            Term::NonTerminal { ref name, ref unwrap } => {
                if *unwrap { format!("< {:} >", name) } else { name.clone() }
            },
            Term::Terminal { ref type_, ref value } => {
                if let Some(ref val) = value {
                    format!("\"{:}\" <- \"{:}\"", type_, val)
                } else { format!("\"{:}\"", type_) }
            },
            Term::Group { ref expr, ref unwrap } => {
                let mut lstr = String::from("(");
                let mut rstr = String::from(")");
                if *unwrap { 
                    lstr.insert(0, '<');
                    rstr.push('>');
                }
                format!("{:} {:} {:}", lstr, expr.dump(), rstr)
            },
            Term::Optional { ref expr, ref unwrap } => {
                let mut lstr = String::from("[");
                let mut rstr = String::from("]");
                if *unwrap { 
                    lstr.insert(0, '<');
                    rstr.push('>');
                }
                format!("{:} {:} {:}", lstr, expr.dump(), rstr)
            },
            Term::Repetition { ref expr, ref unwrap } => {
                let mut lstr = String::from("{");
                let mut rstr = String::from("}");
                if *unwrap { 
                    lstr.insert(0, '<');
                    rstr.push('>');
                }
                format!("{:} {:} {:}", lstr, expr.dump(), rstr)
            },
        }
    }

    fn parse_inner(src: &[Token], next: &mut usize) -> Result<Self, ParseError> {
        let unwrap = parse_step!(src, next, "LeftUnwrap").is_ok();
        if let Ok(pn) = parse_step!(src, next, "ProductionName") {
            if unwrap { parse_step!(src, next, "RightUnwrap")?; }
            Ok(Term::NonTerminal{ name: pn, unwrap: unwrap })
        } else if let Ok(token) = parse_step!(src, next, "Token") {
            let value = if let Ok(_) = parse_step!(src, next, "TokenValue") {
                let val = parse_step!(src, next, "Token")?;
                Some(val)
            } else { None };
            if unwrap {
                Err(ParseError {
                    msg: String::from("Token cannot be unwrap!"),
                    index: *next
                })
            } else {
                Ok(Term::Terminal {
                    type_: token,
                    value: value,
                })
            }
        } else if let Ok(_) = parse_step!(src, next, "LeftGroup") {
            let expr = Expression::parse_inner(src, next)?;
            parse_step!(src, next, "RightGroup")?;
            if unwrap { parse_step!(src, next, "RightUnwrap")?; }
            Ok(Term::Group {
                expr: Box::new(expr),
                unwrap: !unwrap,
            })
        } else if let Ok(_) = parse_step!(src, next, "LeftOptional") {
            let expr = Expression::parse_inner(src, next)?;
            parse_step!(src, next, "RightOptional")?;
            if unwrap { parse_step!(src, next, "RightUnwrap")?; }
            Ok(Term::Optional {
                expr: Box::new(expr),
                unwrap: !unwrap,
            })
        } else if let Ok(_) = parse_step!(src, next, "LeftRepetition") {
            let expr = Expression::parse_inner(src, next)?;
            parse_step!(src, next, "RightRepetition")?;
            if unwrap { parse_step!(src, next, "RightUnwrap")?; }
            Ok(Term::Repetition {
                expr: Box::new(expr),
                unwrap: !unwrap,
            })
        } else {
            Err(ParseError {
                msg: String::from("Unknown type of Term!"),
                index: *next
            })
        }
    }

    pub fn is_unwrap(&self) -> bool {
        match self {
            Term::NonTerminal { ref unwrap, .. } => {
                *unwrap
            },
            Term::Terminal { .. } => {
                false
            },
            Term::Group { ref unwrap, .. } => {
                *unwrap
            },
            Term::Optional { ref unwrap, .. } => {
                *unwrap
            },
            Term::Repetition { ref unwrap, .. } => {
                *unwrap
            },
        }
    }

    pub fn nonterminal<S: Into<String>>(name: S) -> Self {
        Term::NonTerminal {
            name : name.into(),
            unwrap: false,
        }
    }

    pub fn terminal<S: Into<String>>(name: S) -> Self {
        Term::Terminal {
            type_ : name.into(),
            value: None,
        }
    }


    /*
     * There is two case that match token to terminal
     * 1 - Token with type_ match to terminal with type_
     * 2 - Token with type_ and value_ match to terminal with type_ and value_
     */

    pub fn match_token(&self, rhs: &Token) -> bool {
        if let Term::Terminal { type_, value, .. } = self {
            type_ == &rhs.type_ && (value == &None || value.as_ref().unwrap() == &rhs.value_)
        } else { panic!("cannot match non-terminal with token") }
    }
}


/* 
 * We don't care the metadata (unwrap etc.)
 */

impl PartialEq for Term {
    fn eq(&self, other: &Term) -> bool {
        match self {
            Term::NonTerminal { name, .. } => {
                if let Term::NonTerminal { name: rhs, .. } = other {
                    name == rhs
                } else { false }
            },
            Term::Terminal { type_, value, .. } => {
                if let Term::Terminal { type_: rhs, value: rhsvalue } = other {
                    type_ == rhs && value == rhsvalue
                } else { false }
            },    
            _ => panic!("Cannot compare two informal Term !")        
        }
    }
}

impl Eq for Term {
}

impl hash::Hash for Term {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Term::NonTerminal { name, .. } => {
                name.hash(state);
                "NT".hash(state);
            },
            Term::Terminal { type_, .. } => {
                type_.hash(state);
                "T".hash(state);
            },    
            _ => panic!("Cannot hash informal Term !") 
        }
    }
}

impl From<Token> for Term {
    fn from(token: Token) -> Self {
        Term::Terminal {
            type_: token.type_,
            value: Some(token.value_),
        }
    }
}

impl<'a> From<&'a Token> for Term {
    fn from(token: &Token) -> Self {
        Term::Terminal {
            type_: token.type_.clone(),
            value: Some(token.value_.clone()),
        }
    }
}

impl fmt::Display for Production {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:}", self.dump())
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:}", self.dump())
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:}", self.dump())
    }
}
