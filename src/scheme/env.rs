use super::beam::*;
use super::symbol::*;
use super::core;
use crate::utils::*;

use std::fmt;
use std::rc::{Rc, Weak};
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
    syntaxs: HashMap<String, Rc<RefCell<Datum>>>,
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

            "quotient" => core::quotient,
            "remainder" => core::remainder,
            "modulo" => core::modulo,

            "floor" => core::floor,
            "ceiling" => core::ceiling,
            "round" => core::round,
            "truncate" => core::truncate,

            "numerator" => core::numerator,
            "denominator" => core::denominator,

            "make-rectangular" => core::make_rectangular,
            "make-polar" => core::make_polar,
            "real-part" => core::real_part,
            "imag-part" => core::imag_part,
            "magnitude" => core::magnitude,
            "angle" => core::angle,

            "sin" => core::sin,
            "cos" => core::cos,
            "tan" => core::tan,
            "asin" => core::asin,
            "acos" => core::acos,
            "atan" => core::atan,
            "atan2" => core::atan2,
            "log" => core::log,
            "exp" => core::exp,

            "not" => core::not,
            "cons" => core::cons,
            "car" => core::car,
            "cdr" => core::cdr,
            "cadr" => core::cadr,
            "list" => core::list,

            "char=" => core::char_eq,
            "char<" => core::char_lt,
            "char<=" => core::char_le,

            "null?" => core::is_null,
            "eq?" => core::is_eq,
            "eqv?" => core::is_eqv,
            "number?" => core::is_number,
            "integer?" => core::is_integer,
            "rational?" => core::is_rational,
            "real?" => core::is_real,
            "complex?" => core::is_complex,
            "boolean?" => core::is_boolean,
            "char?" => core::is_char,
            "string?" => core::is_string,
            "port?" => core::is_port,
            "input-port?" => core::is_input_port,
            "output-port?" => core::is_output_port,
            "symbol?" => core::is_symbol,
            "pair?" => core::is_pair,
            "list?" => core::is_list,
            "vector?" => core::is_vector,
            "procedure?" => core::is_procedure,

            "make-vector" => core::make_vector,
            "vector-ref" => core::vector_ref,
            "vector-set" => core::vector_set,
            "vector-length" => core::vector_length,

            "make-string" => core::make_string,
            "string-ref" => core::string_ref,
            "string-set" => core::string_set,
            "string-length" => core::string_length,

            "set-car!" => core::set_car,
            "set-cdr!" => core::set_cdr,

            "string->symbol" => core::string_to_symbol,
            "symbol->string" => core::symbol_to_string,
            "number->string" => core::number_to_string,
            "char->integer" => core::char_to_integer,
            "integer->char" => core::integer_to_char,


            "current-input-port" => |v| Ok(SymbolTable::stdin()),
            "current-output-port" => |v| Ok(SymbolTable::stdout()),
            
            "open-input-file" => core::open_input_file,
            "open-output-file" => core::open_output_file,
            "close-input-file" => core::close_input_file,
            "close-output-file" => core::close_output_file,

            "read-char" => core::read_char,
            "peek-char" => core::peek_char,
            "write-char" => core::write_char,
            "display" => |v| {
                print!("{:}", v.borrow().car()?.borrow());
                Ok(SymbolTable::unspecified())
            }
        ];

        m.insert("apply".to_string(), Datum::BuiltinExt(SpecialProcedure::Apply).wrap());
        m.insert("eval".to_string(), Datum::BuiltinExt(SpecialProcedure::Eval).wrap());
        m.insert("call/cc".to_string(), Datum::BuiltinExt(SpecialProcedure::CallCC).wrap());
        m.insert("call-with-currenet-continuation".to_string(), Datum::BuiltinExt(SpecialProcedure::CallCC).wrap());
        // m.insert("curr-env".to_string(), Datum::BuiltinExt(SpecialProcedure::CurrEnv).wrap());

        let sm = sforms![
            "begin" => Begin,
            "define" => Define,
            "lambda" => Lambda,
            "set!" => Set,
            "set-syntax!" => SetSyntax,
            // "set-car!" => SetCar,
            // "set-cdr!" => SetCdr,
            // "and" => And,
            // "or" => Or,
            "if" => If,
            // "else" => Else,
            // "cond" => Cond,
            "quote" => Quote,
            "quasiquote" => Quasiquote,
            "unquote" => Unquote,
            "unquote-splicing" => UnquoteSplicing,
            // "apply" => Apply,
            // "eval" => Eval,
            // "let" => Let,
            // "let*" => Letstar,
            // "let-rec" => Letrec,
            "define-syntax" => DefineSyntax,
            "syntax-rules" => SyntaxRules,
            // "call/cc" => CallCC,
            // "call-with-currenet-continuation" => CallCC
            "curr-env" => CurrEnv,
            "standard-env" => StandardEnv
        ];

        Environment {
            name: String::from(global_env),
            datas: m,
            syntaxs: sm,
            parent: None,
            ..Default::default()
        }.wrap()
    }

    pub fn new() -> Env {
        Environment::forward_with_name(Environment::global(), "program")
    }

    pub fn null() -> Env {
        Environment {
            parent: None,
            ..Default::default()
        }.wrap()
    }

    pub fn put(&mut self, name: String, data: Value) {
        self.datas.insert(name, data);
    }

    pub fn set(&mut self, name: &String, data: Value) -> Result<Value, RuntimeError> {
        if let Some(d) = self.datas.get_mut(name) {
            let old = d.clone();
            *d = data;
            Ok(old)
        } else if let Some(p) = &mut self.parent {
            p.borrow_mut().set(name, data)
        } else {
            Err(RuntimeError::new(format!("Unbound variable {:}", name)))
        }        
    }

    pub fn find(&self, name: &String) -> Result<Value, RuntimeError> {
        if let Some(d) = self.datas.get(name) {
            Ok(d.clone())
        } else if let Some(p) = &self.parent {
            p.borrow().find(name)
        } else {
            Err(RuntimeError::new(format!("Unbound variable {:}", name)))
        }
    }

    pub fn put_syntax(&mut self, name: String, data: Value) {
        self.syntaxs.insert(name, data);
    }

    pub fn set_syntax(&mut self, name: &String, data: Value) -> Result<Value, RuntimeError> {
        if let Some(d) = self.syntaxs.get_mut(name) {
            let old = d.clone();
            *d = data;
            Ok(old)
        } else if let Some(p) = &mut self.parent {
            p.borrow_mut().set_syntax(name, data)
        } else {
            Err(RuntimeError::new(format!("No keyword found {:}", name)))
        }        
    }

    pub fn find_syntax(&self, name: &String) -> Result<Value, RuntimeError> {
        if let Some(d) = self.syntaxs.get(name) {
            Ok(d.clone())
        } else if let Some(p) = &self.parent {
            p.borrow().find_syntax(name)
        } else {
            Err(RuntimeError::new(format!("No keyword found {:}", name)))
        }
    }

    pub fn forward(curr: Env) -> Env {
        Environment {
            parent: Some(curr.clone()),
            ..Default::default()
        }.wrap()
    }

    pub fn forward_with_name(curr: Env, name: impl Into<String>) -> Env {
        Environment {
            name: name.into(),
            parent: Some(curr.clone()),
            ..Default::default()    
        }.wrap()
    }
    
}   