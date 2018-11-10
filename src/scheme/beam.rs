use super::env::*;
use super::symbol::*;
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
    TransformerSpec {
        pattern: Value,
        template: Value
    },
    // literals, transformerspecs
    Syntax {
        literals: Value,
        rules: Value
    },
    Continuation {
        expr: Value,
        env: Env,
        level: usize
    },

    Holder,
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
            Datum::Holder => write!(f, "_"),
            Datum::Continuation { ref expr, ref env, ref level } => write!(f, "{:?}", expr.borrow()),
            Datum::TransformerSpec { ref pattern, ref template } => write!(f, "<SyntaxRule {:?} -> {:?}>", pattern.borrow(), template.borrow()),
            Datum::Syntax { ref literals, ref rules } => write!(f, "<Syntax {:?} {:?}>", literals.borrow(), rules.borrow()),
            
            _ => write!(f, "<Unknown>")
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
            Datum::String(ref s) => write!(f, "{:}", s),
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
            Datum::Holder => write!(f, "_"),
            Datum::Continuation { ref expr, ref env, ref level } => write!(f, "{:}", expr.borrow()),
            Datum::TransformerSpec { ref pattern, ref template } => write!(f, "<SyntaxRule {:?} -> {:?}>", pattern.borrow(), template.borrow()),
            Datum::Syntax { ref literals, ref rules } => write!(f, "<Syntax {:?} {:?}>", literals.borrow(), rules.borrow()),
         
            _ => write!(f, "<Unknown>")
        }
    }
}


impl Datum {

    pub fn wrap(self) -> Value {
        Rc::new(RefCell::new(self))
    }

    pub fn is_nil(&self) -> bool {
        if let Datum::Nil = self { true } else { false }
    }

    pub fn is_specified(&self) -> bool {
        if let Datum::Unspecified = self { false } else { true }
    }
    
    pub fn is_holder(&self) -> bool {
        if let Datum::Holder = self { true } else { false }
    }

    pub fn is_pair(&self) -> bool {
        if let Datum::Pair(_, _) = self { true } else { false }
    }

    pub fn is_false(&self) -> bool {
        if let Datum::Boolean(false) = self { true } else { false }
    }

    pub fn is_true(&self) -> bool {
        !self.is_false()
    }

    pub fn is_string(&self) -> bool {
        if let Datum::String(_) = self { true } else { false }
    }

    pub fn is_number(&self) -> bool {
        if let Datum::Number(_) = self { true } else { false }
    }

    pub fn is_symbol(&self) -> bool {
        if let Datum::Symbol(_) = self { true } else { false }
    }

    pub fn car(&self) -> Result<Value, RuntimeError> {
        if let Datum::Pair(a, d) = self {
            Ok(a.clone())
        } else {
            Err(RuntimeError::new(format!("ice: car on non-list : {:?}", self)))
        }
    }

    pub fn set_car(&mut self, rhs: Value) -> Result<Value, RuntimeError> {
        if let Datum::Pair(ref mut a, _) = self {
            *a = rhs.clone();
            Ok(SymbolTable::unspecified())
        } else {
            Err(RuntimeError::new("ice: set_car on non-list"))
        }
    }

    pub fn set_cdr(&mut self, rhs: Value) -> Result<Value, RuntimeError> {
        if let Datum::Pair(_, ref mut d) = self {
            *d = rhs.clone();
            Ok(SymbolTable::unspecified())
        } else {
            Err(RuntimeError::new("ice: set_car on non-list"))
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

    pub fn len(&self) -> usize {
        if let Datum::Pair(ref a, ref d) = self {
            1 + List::from(d.clone()).collect::<Vec<_>>().len()
        } else {
            0
        }
        
    }
}


#[derive(Debug, Clone)]
pub struct List {
    item: Option<Value>
}

impl List {
    pub fn new() -> Self {
        List {
            item: None
        }
    }

    fn is_nil(&self) -> bool {
        self.item.as_ref().unwrap().borrow().is_nil()
    }

    pub fn clone(l: Value) -> Value {
        List::from(l).collect::<List>().into()
    }

    pub fn clone_deep(l: Value) -> Value {
        if l.borrow().is_pair() {
            List::from(l).map(|x| x.map(|y| List::clone_deep(y))).collect::<List>().into()
        } else {
            l
        }
    }
}

impl Into<Value> for List {
    fn into(self) -> Value {
        if let Some(val) = self.item {
            val
        } else {
            SymbolTable::nil()
        }
    }
}

impl iter::FromIterator<ListItem> for List {
    /* If a list contains multiply ellipsis,
     * only the last will count, others will be skipped
     */
    fn from_iter<I: IntoIterator<Item = ListItem>>(list: I) -> List {
        let mut ret = SymbolTable::nil();
        let mut last = SymbolTable::nil();
        let mut list = list.into_iter();
        while let Some(next) = list.next() {
            if let ListItem::Item(x) = next {
                if ret.borrow().is_nil() {
                    ret = Datum::Pair(x, SymbolTable::nil()).wrap();
                    last = ret.clone();
                } else {
                    let d = Datum::Pair(x, SymbolTable::nil()).wrap();
                    last.borrow_mut().set_cdr(d.clone());
                    last = d;
                }
            } else if let ListItem::Ellipsis(x) = next {
                if ret.borrow().is_nil() {
                    ret = x;
                } else {
                    last.borrow_mut().set_cdr(x);
                }
            }
        }
        List::from(ret)
    }
}

// impl Extend<ListItem> for List {
//     fn extend<I: IntoIterator<Item = ListItem>>(&mut self, list: I) {
//         let mut l = self.collect::<Vec<ListItem>>();
//         l.extend(list.into_iter().collect::<Vec<ListItem>>());
//         *self = l.into_iter().collect();
//     }
// }

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

impl ListItem {
    fn map<F: FnOnce(Value) -> Value>(self, f: F) -> Self {
        match self {
            ListItem::Item(val) => ListItem::Item(f(val)),
            ListItem::Ellipsis(val) => ListItem::Ellipsis(f(val)),
        }
    }

    // fn try_map<F: FnOnce(Value) -> Result<Value, RuntimeError>>(self, f: F) -> Result<ListItem, RuntimeError> {
    //     match self {
    //         ListItem::Item(val) => Ok(ListItem::Item(f(val)?)),
    //         ListItem::Ellipsis(val) => Ok(ListItem::Ellipsis(f(val)?)),
    //     }
    // }
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
        write!(f, "<lambda {:?} -> {:?}>", self.formals.borrow(), self.expr.borrow())
    }
}

impl fmt::Debug for LambdaExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<lambda {:?} -> {:?} {:?}>", self.formals.borrow(), self.expr.borrow(), self.env.borrow())
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxRule {
    pub pattern: Value,
    pub template: Value
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
    DefineSyntax,
    SyntaxRules
}