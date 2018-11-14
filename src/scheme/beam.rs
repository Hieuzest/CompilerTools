use super::env::*;
use super::symbol::*;
use super::number::*;
use std::iter;
use std::str::FromStr;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::fmt;
use std::fs::File;
use std::io::{Write, Read};

pub type Value = Rc<RefCell<Datum>>;
pub type ValueRef = Weak<RefCell<Datum>>;

#[derive(Debug)]
pub struct RuntimeError {
    msg: String
}

macro_rules! error {
    ($($arg:tt)*) => (
        Err(RuntimeError::new(format!($($arg)*)))
    )
}

impl RuntimeError {
    pub fn new(s: impl Into<String>) -> Self {
        RuntimeError {
            msg: s.into()
        }
    }
}



#[derive(Clone)]
pub enum Datum {
    // Atomic
    Boolean(bool),
    Number(Number),
    Character(char),
    String(String),
    Symbol(String),
    Nil,
    Unspecified,

    Pair(Value, Value),

    Vector(Vec<Value>),

    Port(Port),

    // Evaluated Value
     
    // Native syntax
    SpecialForm(SpecialForm),

    Syntax(SyntaxRules),

    //Native procedure
    Builtin(Box<fn(Value) -> Result<Value, RuntimeError>>),

    Lambda(LambdaExpression),

    
    Continuation(Continuation),

}

// impl Drop for Datum {
//     fn drop(&mut self) {
//         println!("Droping {:?}", self);
//     }
// }

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
            Datum::Vector(ref v) => write!(f, "#({})", v.iter().fold(String::new(), |s, x| format!("{:}{:?} ", s, x.borrow()))),
            Datum::Pair(ref a, ref b) => write!(f, "({:?}{:})", a.borrow(), List::from(b.clone()).fold(String::new(), |s, x| format!("{:} {:}", s, match x {
                ListItem::Item(x) => format!("{:?}", x.borrow()),
                ListItem::Ellipsis(x) => format!(". {:?}", x.borrow())
            }))),
            Datum::Builtin(ref func) => write!(f, "<Builtin {:?}>", func),
            Datum::Lambda(ref lambda) => write!(f, "{:?}", lambda),
            Datum::SpecialForm(ref sf) => write!(f, "<SpecialForm {:?}>", sf),
            Datum::Continuation(ref cont) => write!(f, "{:?}", cont),
            Datum::Syntax(ref syntax) => write!(f, "{:?}", syntax),
            Datum::Port(ref port) => write!(f, "{:?}", port),
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
            Datum::Vector(ref v) => write!(f, "#({})", v.iter().fold(String::new(), |s, x| format!("{:}{:?} ", s, x.borrow()))),
            Datum::Pair(ref a, ref b) => write!(f, "({:}{:})", a.borrow(), List::from(b.clone()).fold(String::new(), |s, x| format!("{:} {:}", s, match x {
                ListItem::Item(x) => format!("{:}", x.borrow()),
                ListItem::Ellipsis(x) => format!(". {:}", x.borrow())
            }))),
            Datum::Builtin(ref func) => write!(f, "{:?}", func),
            Datum::Lambda(ref lambda) => write!(f, "{:}", lambda),
            Datum::SpecialForm(ref sf) => write!(f, "<SpecialForm {:?}>", sf),
            Datum::Continuation(ref cont) => write!(f, "{:?}", cont),
            Datum::Syntax(ref syntax) => write!(f, "{:?}", syntax),
            Datum::Port(ref port) => write!(f, "{:?}", port),
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

    pub fn is_false(&self) -> bool {
        if let Datum::Boolean(false) = self { true } else { false }
    }

    pub fn is_true(&self) -> bool {
        !self.is_false()
    }

    pub fn is_boolean(&self) -> bool {
        if let Datum::Boolean(_) = self { true } else { false }
    }

    pub fn as_boolean(&self) -> Result<bool, RuntimeError> {
        if let Datum::Boolean(ref b) = self { Ok(*b) } else { Err(RuntimeError::new(format!("Expected boolean: {:?}", self))) }
    }

    pub fn is_character(&self) -> bool {
        if let Datum::Character(_) = self { true } else { false }
    }

    pub fn as_character(&self) -> Result<char, RuntimeError> {
        if let Datum::Character(ref c) = self { Ok(*c) } else { Err(RuntimeError::new(format!("Expected number: {:?}", self))) }
    }

    pub fn is_port(&self) -> bool {
        if let Datum::Port(_) = self { true } else { false }
    }

    pub fn as_port(&self) -> Result<Port, RuntimeError> {
        if let Datum::Port(ref p) = self { Ok(p.clone()) } else { Err(RuntimeError::new(format!("Expected port: {:?}", self))) }
    }

    pub fn is_pair(&self) -> bool {
        if let Datum::Pair(_, _) = self { true } else { false }
    }

    pub fn as_pair(&self) -> Result<(Value, Value), RuntimeError> {
        if let Datum::Pair(ref car, ref cdr) = self { 
            Ok((car.clone(), cdr.clone())) } else { Err(RuntimeError::new(format!("Expected pair: {:?}", self))) }
    }

    pub fn into_pair(self) -> Result<(Value, Value), RuntimeError> {
        if let Datum::Pair(car, cdr) = self { 
            Ok((car, cdr)) } else { Err(RuntimeError::new(format!("Expected pair: {:?}", self))) }
    }

    pub fn is_list(&self) -> bool {
        if let Datum::Pair(ref a, ref d) = self {
            if d.borrow().is_nil() { true } 
            else {
                if let ListItem::Ellipsis(_) = List::from(d.clone()).last().unwrap() {
                    false
                } else { true }
            }
        } else {
            self.is_nil()
        }
    }

    pub fn is_vector(&self) -> bool {
        if let Datum::Vector(_) = self { true } else { false }
    }

    pub fn as_vector(&self) -> Result<Vec<Value>, RuntimeError> {
        if let Datum::Vector(ref vector) = self { Ok(vector.clone()) } else { Err(RuntimeError::new(format!("Expected vector: {:?}", self))) }
    }

    pub fn as_vector_ref(&self) -> Result<&Vec<Value>, RuntimeError> {
        if let Datum::Vector(ref vector) = self { Ok(vector) } else { Err(RuntimeError::new(format!("Expected vector: {:?}", self))) }
    }

    pub fn as_vector_mut(&mut self) -> Result<&mut Vec<Value>, RuntimeError> {
        if let Datum::Vector(ref mut vector) = self { Ok(vector) } else { Err(RuntimeError::new(format!("Expected vector: {:?}", self))) }
    }

    pub fn is_procedure(&self) -> bool {
        if let Datum::Builtin(_) = self { true }
        else if let Datum::Lambda(_) = self { true }
        else if let Datum::Continuation(_) = self { true }
        else { false }
    }

    pub fn is_string(&self) -> bool {
        if let Datum::String(_) = self { true } else { false }
    }

    pub fn as_string(&self) -> Result<String, RuntimeError> {
        if let Datum::String(ref id) = self { Ok(id.clone()) } else { Err(RuntimeError::new(format!("Expected string: {:?}", self))) }
    }

    pub fn as_string_mut(&mut self) -> Result<&mut String, RuntimeError> {
        if let Datum::String(ref mut id) = self { Ok(id) } else { Err(RuntimeError::new(format!("Expected string: {:?}", self))) }
    }


    pub fn is_number(&self) -> bool {
        if let Datum::Number(_) = self { true } else { false }
    }

    pub fn as_number(&self) -> Result<Number, RuntimeError> {
        if let Datum::Number(ref n) = self { Ok(*n) } else { Err(RuntimeError::new(format!("Expected number: {:?}", self))) }
    }

    pub fn is_symbol(&self) -> bool {
        if let Datum::Symbol(_) = self { true } else { false }
    }

    pub fn as_symbol(&self) -> Result<String, RuntimeError> {
        if let Datum::Symbol(ref id) = self { Ok(id.clone()) } else { Err(RuntimeError::new(format!("Expected symbol: {:?}", self))) }
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

    pub fn one(val: Value) -> Self {
        List::new().chain(iter::once(ListItem::Item(val))).collect()
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
                    last.borrow_mut().set_cdr(d.clone()).unwrap();
                    last = d;
                }
            } else if let ListItem::Ellipsis(x) = next {
                if ret.borrow().is_nil() {
                    ret = x;
                } else {
                    last.borrow_mut().set_cdr(x).unwrap();
                }
            }
        }
        List::from(ret)
    }
}

impl Extend<ListItem> for List {
    fn extend<I: IntoIterator<Item = ListItem>>(&mut self, list: I) {
        *self = self.chain(list.into_iter()).collect();
    }
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

#[derive(Clone)]
pub struct SyntaxRules {
    pub literals: Value,
    pub rules: Value,
    pub env: Env,
}

impl fmt::Debug for SyntaxRules {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<syntax {:?} {:?} {:?}>", self.literals.borrow(), self.rules.borrow(), self.env.borrow())
    }
}

pub type Cont = Box<Continuation>;

#[derive(Clone)]
pub enum Continuation {
    Return,
    EvaluateList(Value, Env, usize, Cont),
    EvaluateProcedure(Value, Value, Env, usize, Cont),
    EvaluateProcedureSplicing(Value, Value, Env, usize, Cont),
    EvaluateApply(Value, usize, Cont),
    EvaluateBegin(Value, Env, usize, Cont),
    EvaluateIf(Value, Env, usize, Cont),    
    EvaluateSet(Value, Env, usize, Cont),
    EvaluateSetSyntax(Value, Env, usize, Cont),
    EvaluateDefine(Value, Env, usize, Cont),
    EvaluateDefineSyntax(Value, Env, usize, Cont),
    EvaluateSyntax(Value, Env, usize, Cont),
    EvaluateCallCC(Cont),
    QuasiquoteList(Value, Value, Env, usize, Cont),
    QuasiquoteListSplicing(Value, Value, Env, usize, Cont),
}

impl Continuation {
    pub fn splicing(self) -> Result<Continuation, RuntimeError> {
        if let Continuation::QuasiquoteList(car, cdr, env, level, cont) = self {
            Ok(Continuation::QuasiquoteListSplicing(car, cdr, env, level, cont))
        } else {
            Err(RuntimeError::new("ice: failed to splice a non quasiquote list"))
        }
    }
}

impl fmt::Debug for Continuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Continuation::Return => write!(f, "<Cont/Return>"),
            Continuation::EvaluateList(ref expr, ref env, ref level, ref cont) => write!(f, "<Cont/EvalList {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateProcedure(ref car, ref cdr, ref env, ref level, ref cont) => write!(f, "<Cont/Proc {:?} - {:?} :{:?}>", car.borrow(), cdr.borrow(), *cont),
            Continuation::EvaluateProcedureSplicing(ref car, ref cdr, ref env, ref level, ref cont) => write!(f, "<Cont/ProcS {:?} - {:?} :{:?}>", car.borrow(), cdr.borrow(), *cont),
            Continuation::EvaluateApply(ref expr, ref level, ref cont) => write!(f, "<Cont/Apply {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateBegin(ref expr, ref env, ref level, ref cont) => write!(f, "<Cont/Begin {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateIf(ref expr, ref env, ref level, ref cont) => write!(f, "<Cont/If {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateSet(ref expr, ref env, ref level, ref cont) => write!(f, "<Cont/Set {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateSetSyntax(ref expr, ref env, ref level, ref cont) => write!(f, "<Cont/SetSyntax {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateDefine(ref expr, ref env, ref level, ref cont) => write!(f, "<Cont/Define {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateDefineSyntax(ref expr, ref env, ref level, ref cont) => write!(f, "<Cont/DefineSyntax {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateSyntax(ref expr, ref env, ref level, ref cont) => write!(f, "<Cont/Syntax {:?} :{:?}>", expr.borrow(), *cont),
            Continuation::EvaluateCallCC(ref cont) => write!(f, "<Cont/CallCC :{:?}>", *cont),
            Continuation::QuasiquoteList(ref car, ref cdr, ref env, ref level, ref cont) => write!(f, "<Cont/QList {:?} - {:?} :{:?}>", car.borrow(), cdr.borrow(), *cont),
            Continuation::QuasiquoteListSplicing(ref car, ref cdr, ref env, ref level, ref cont) => write!(f, "<Cont/QListS {:?} - {:?} :{:?}>", car.borrow(), cdr.borrow(), *cont),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SpecialForm {
    Begin,
    Lambda,
    If,
    SyntaxRules,
    Define,
    DefineSyntax,
    Set,
    SetSyntax,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    CallCC,
}

#[derive(Clone)]
pub enum Port {
    Output(Rc<RefCell<Write>>),
    Input(Rc<RefCell<Read>>),
    None
}

impl fmt::Debug for Port {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Port::Output(_) => write!(f, "<OutputPort>"),
            Port::Input(_) => write!(f, "<InputPort>"),
            Port::None => write!(f, "<EmptyPort>"),
        }
    }
}
