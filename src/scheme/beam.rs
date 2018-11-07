use super::env::*;
use std::iter;
use std::str::FromStr;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

pub type Value = Rc<RefCell<Datum>>;


#[derive(Debug)]
pub struct RuntimeError {
    msg: String
}

impl RuntimeError {
    pub fn new(s: impl Into<String>) -> Self {
        RuntimeError {
            msg: s.into()
        }
    }


}


#[derive(Debug, Copy, Clone)]
pub enum AbbrevPrefix {
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
}

#[derive(Clone)]
pub enum Datum {
    // Atomic
    Boolean(bool),
    Number(f64),
    Character(char),
    String(String),
    Symbol(String),
    Nil,
    Unspecified,

    Pair(Value, Value),
    // Non-s-expr
    Abbreviation(AbbrevPrefix, Value),

    // Vector(Vec<Datum>),


    // Evaluated Value
    SpecialForm(SpecialForm),
    Builtin(Box<fn(Value) -> Result<Value, RuntimeError>>),
    Lambda(LambdaExpression),
    // Syntax(SyntaxRule),
}

impl fmt::Debug for Datum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Datum::Boolean(true) => write!(f, "#t"),
            Datum::Boolean(false) => write!(f, "#f"),
            Datum::Number(ref n) => write!(f, "{:}", n),
            Datum::Character('\n') => write!(f, "#\\newline"),
            Datum::Character(' ') => write!(f, "#\\space"),
            Datum::Character(ref c) => write!(f, "#\\{:}", c),
            Datum::String(ref s) => write!(f, "\"{:}\"", s),
            Datum::Symbol(ref id) => write!(f, "{:}", id),
            Datum::Nil => write!(f, "()"),
            Datum::Unspecified => write!(f, "<Unspecified>"),
            Datum::Pair(ref a, ref b) => write!(f, "({:?}{:})", a.borrow(), List::from(b.clone()).fold(String::new(), |s, x| format!("{:} {:}", s, match x {
                ListItem::Item(x) => format!("{:?}", x.borrow()),
                ListItem::Ellipsis(x) => format!(". {:?}", x.borrow())
            }))),
            // Datum::Pair(ref a, ref d) => write!(f, "({:?} . {:?})", *a.borrow(), *d.borrow()),
            Datum::Builtin(ref func) => write!(f, "<Builtin {:?}>", func),
            Datum::Lambda(ref lambda) => write!(f, "{:?}", lambda),
            Datum::Abbreviation(AbbrevPrefix::Quote, ref val) => write!(f, "'{:?}", val.borrow()),
            Datum::Abbreviation(AbbrevPrefix::Quasiquote, ref val) => write!(f, "`{:?}", val.borrow()),
            Datum::Abbreviation(AbbrevPrefix::Unquote, ref val) => write!(f, ",{:?}", val.borrow()),
            Datum::Abbreviation(AbbrevPrefix::UnquoteSplicing, ref val) => write!(f, ",@{:?}", val.borrow()),
            Datum::SpecialForm(ref sf) => write!(f, "<SpecialForm {:?}>", sf),
        }
    }
}

impl fmt::Display for Datum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Datum::Boolean(true) => write!(f, "#t"),
            Datum::Boolean(false) => write!(f, "#f"),
            Datum::Number(ref n) => write!(f, "{:}", n),
            Datum::Character(ref c) => write!(f, "{:}", c),
            Datum::String(ref s) => write!(f, "\"{:}\"", s),
            Datum::Symbol(ref id) => write!(f, "{:}", id),
            Datum::Nil => write!(f, "()"),
            Datum::Unspecified => write!(f, "<Unspecified>"),
            Datum::Pair(ref a, ref b) => write!(f, "({:}{:})", a.borrow(), List::from(b.clone()).fold(String::new(), |s, x| format!("{:} {:}", s, match x {
                ListItem::Item(x) => format!("{:}", x.borrow()),
                ListItem::Ellipsis(x) => format!(". {:}", x.borrow())
            }))),
            // Datum::Pair(ref a, ref d) => write!(f, "({:?} . {:?})", *a.borrow(), *d.borrow()),
            Datum::Builtin(ref func) => write!(f, "{:?}", func),
            Datum::Lambda(ref lambda) => write!(f, "{:}", lambda),
            Datum::Abbreviation(AbbrevPrefix::Quote, ref val) => write!(f, "'{:}", val.borrow()),
            Datum::Abbreviation(AbbrevPrefix::Quasiquote, ref val) => write!(f, "`{:}", val.borrow()),
            Datum::Abbreviation(AbbrevPrefix::Unquote, ref val) => write!(f, ",{:}", val.borrow()),
            Datum::Abbreviation(AbbrevPrefix::UnquoteSplicing, ref val) => write!(f, ",@{:}", val.borrow()),
            Datum::SpecialForm(ref sf) => write!(f, "<SpecialForm {:?}>", sf),
        }
    }
}

impl Default for Datum {
    fn default() -> Self {
        Datum::Unspecified
    }
}

impl Datum {
    pub fn new() -> Self {
        Datum::Nil
    }

    pub fn wrap(self) -> Value {
        Rc::new(RefCell::new(self))
    }

    pub fn is_nil(&self) -> bool {
        if let Datum::Nil = self { true } else { false }
    }

    pub fn is_specified(&self) -> bool {
        if let Datum::Unspecified = self { false } else { true }
    }

    pub fn is_pair(&self) -> bool {
        if let Datum::Pair(_, _) = self { true } else { false }
    }

    pub fn car(&self) -> Result<Value, RuntimeError> {
        if let Datum::Pair(a, d) = self {
            Ok(a.clone())
        } else {
            Err(RuntimeError::new(format!("ice: car on non-list : {:?}", self)))
        }
    }

    pub fn cdr(&self) -> Result<Value, RuntimeError> {
        if let Datum::Pair(a, d) = self {
            Ok(d.clone())
        } else {
            Err(RuntimeError::new(format!("ice: cdr on non-list : {:?}", self)))
        }
    }

    pub fn cadr(&self) -> Result<Value, RuntimeError> {
        self.cdr()?.borrow().car()
    }
}

#[derive(Debug, Clone)]
pub struct List {
    item: Option<Value>
}

impl List {
    fn is_nil(&self) -> bool {
        self.item.as_ref().unwrap().borrow().is_nil()
    }

    // pub fn car(&self) -> Result<Value, RuntimeError> {
    //     self.item.borrow().car()
    // }

    // pub fn cdr(&self) -> Result<Value, RuntimeError> {
    //     self.item.borrow().cdr()
    // }
}

impl From<Value> for List {
    fn from(value: Value) -> List {
        List {
            item: Some(value)
        }
    }
}

impl From<&Value> for List {
    fn from(value: &Value) -> List {
        List {
            item: Some(value.clone())
        }
    }
}

pub enum ListItem {
    Item(Value), Ellipsis(Value)
}

impl Iterator for List {
    type Item = ListItem;

    fn next(&mut self) -> Option<ListItem> {
        if let None = self.item {
            None
        } else if let Ok(a) = self.item.as_ref().unwrap().clone().borrow().car() {
            self.item = Some(self.item.as_ref().unwrap().clone().borrow().cdr().unwrap());
            Some(ListItem::Item(a))
        } else if self.is_nil() {
            self.item = None;
            None
        } else {
            let ret = ListItem::Ellipsis(self.item.as_ref().unwrap().clone());
            self.item = None;
            Some(ret)
        }
    }
}

#[derive(Clone)]
pub struct LambdaExpression {
    pub formals: Value,
    pub expr: Value,
    pub env: Env
}

impl fmt::Display for LambdaExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<lambda {:?} -> {:?}", self.formals.borrow(), self.expr.borrow())
    }
}

impl fmt::Debug for LambdaExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<lambda {:?} -> {:?} {:?}", self.formals.borrow(), self.expr.borrow(), self.env.borrow())
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxRule {
    pub formals: Vec<String>,
    pub expr: Box<Datum>
}

#[derive(Debug, Copy, Clone)]
pub enum SpecialForm {
    Eval,
    Apply,
    Begin,
    Define,
    Lambda,
    Set,
    SetCar,
    SetCdr,
    Let,
    Letstar,
    Letrec,
    And,
    Or,
    Cond,
    If,
    Else,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    DefineSyntax
}