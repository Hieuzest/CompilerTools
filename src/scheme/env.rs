use super::beam::*;
use super::symbol::*;
use super::core;
use crate::utils::*;

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;

pub type Env = Rc<RefCell<Environment>>;
pub const global_env: &'static str = "global";

macro_rules! builtins(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key.to_string(), Datum::Builtin(Box::new($value)).wrap());
            )+
            m
        }
     };
);

macro_rules! sforms(
    { $($key:expr => $value:tt),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key.to_string(), Datum::SpecialForm(SpecialForm::$value).wrap());
            )+
            m
        }
     };
);

#[derive(Default, Clone)]
pub struct Environment {
    name: String,
    datas: HashMap<String, Rc<RefCell<Datum>>>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !DEBUG!() || !VERBOSE!() {
            write!(f, "")
        } else if let Some(ref p) = self.parent {
            write!(f, "\n\t<env {:p} {:} {{{:}}}> ->{:?}", self, self.name, self.datas.iter().fold(String::new(), |s, (k, v)| format!("{:}\n\t\t{:}: {:}, ", s, k, v.borrow())), *p.borrow())
        } else {
            write!(f, "\n\t<env {}>", global_env)
        }
    }
}

impl Environment {

    pub fn wrap(self) -> Env {
        Rc::new(RefCell::new(self))
    }

    fn global() -> Env {
        let mut m = builtins![
            "=" => core::eq,
            "<" => core::lt,
            "<=" => core::le,
            "+" => core::add,
            "-" => core::sub,
            "*" => core::mul,
            "/" => core::div,
            "not" => core::not,
            "cons" => core::cons,
            "car" => core::car,
            "cdr" => core::cdr,
            "cadr" => core::cadr,
            "list" => core::list,
            "null?" => core::is_null,
            "eq?" => core::is_eq,
            "number?" => core::is_number,
            "string?" => core::is_string,
            "symbol?" => core::is_symbol,
            "pair?" => core::is_pair,

            "display" => |v| {
                print!("{:}", v.borrow().car()?.borrow());
                Ok(SymbolTable::unspecified())
            }
        ];
        m.extend(sforms![
            "begin" => Begin,
            "define" => Define,
            "lambda" => Lambda,
            "set!" => Set,
            "set-car!" => SetCar,
            "set-cdr!" => SetCdr,
            "and" => And,
            "or" => Or,
            "if" => If,
            "else" => Else,
            "cond" => Cond,
            "quote" => Quote,
            "quasiquote" => Quasiquote,
            "unquote" => Unquote,
            "unquote-splicing" => UnquoteSplicing,
            "apply" => Apply,
            "eval" => Eval,
            "let" => Let,
            "let*" => Letstar,
            "let-rec" => Letrec,
            "define-syntax" => DefineSyntax,
            "syntax-rules" => SyntaxRules
        ]);

        Environment {
            name: String::from(global_env),
            datas: m,
            parent: None,
            ..Default::default()
        }.wrap()
    }

    pub fn new() -> Env {
        Environment::forward_with_name(Environment::global(), "program")
    }

    pub fn put(&mut self, name: String, data: Rc<RefCell<Datum>>) {
        self.datas.insert(name, data);
    }

    pub fn set(&mut self, name: &String, data: Rc<RefCell<Datum>>) -> Result<Rc<RefCell<Datum>>, RuntimeError> {
        if let Some(d) = self.datas.get_mut(name) {
            let old = d.clone();
            *d = data;
            Ok(old)
        } else if let Some(p) = &mut self.parent {
            p.borrow_mut().set(name, data)
        } else {
            Err(RuntimeError::new(format!("No variable named {:}", name)))
        }        
    }

    pub fn find(&self, name: &String) -> Result<Rc<RefCell<Datum>>, RuntimeError> {
        if let Some(d) = self.datas.get(name) {
            Ok(d.clone())
        } else if let Some(p) = &self.parent {
            p.borrow().find(name)
        } else {
            Err(RuntimeError::new(format!("No variable named {:}", name)))
        }
    }

    pub fn forward(curr: Env) -> Env {
        Environment {
            datas: HashMap::new(),
            parent: Some(curr.clone()),
            ..Default::default()
        }.wrap()
    }

    pub fn forward_with_name(curr: Env, name: impl Into<String>) -> Env {
        Environment {
            name: name.into(),
            datas: HashMap::new(),
            parent: Some(curr.clone()),
            ..Default::default()    
        }.wrap()
    }
    
    // pub fn clone(curr: Env) -> Env {
    //     Environment {
    //         datas: curr.clone().borrow().datas.clone(),
    //         parent: Some(curr.clone()),
    //         ..Default::default()
    //     }.wrap()
    // }

    // pub fn clone_with_name(curr: Env, name: impl Into<String>) -> Env {
    //     Environment {
    //         name: name.into(),
    //         datas: curr.clone().borrow().datas.clone(),
    //         parent: Some(curr.clone()),
    //         ..Default::default()
    //     }.wrap()
    // }

}   