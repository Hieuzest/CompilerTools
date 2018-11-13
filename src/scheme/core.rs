use super::beam::*;
use super::symbol::*;
use std::f64;
use std::iter;
use std::rc::Rc;
use std::cell::RefCell;
use std::io;
use std::io::prelude::*;
use std::fs::File;


fn gcd(a: i64, b: i64) -> i64 {
    let (a, b) = (a.abs(), b.abs());
    if a < b { let (a, b) = (b, a); }
    while b > 0 {
        let (a, b) = (b, a % b);
    }
    a
}

pub fn not(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Boolean(b) = *operands.borrow().car()?.borrow() {
        return Ok(SymbolTable::bool(!b));
    }
    Err(RuntimeError::new("Not boolean"))
}


// Primitive numericial procedures

pub fn eq(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().as_number()? == operands.borrow().cadr()?.borrow().as_number()?))
}


pub fn lt(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().as_number()? < operands.borrow().cadr()?.borrow().as_number()?))
}


pub fn le(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().as_number()? <= operands.borrow().cadr()?.borrow().as_number()?))
}

macro_rules! num_unary_op {
    ($id: ident, $func: expr) => {
        pub fn $id(operands: Value) -> Result<Value, RuntimeError> {
            Ok(SymbolTable::number($func(operands.borrow().car()?.borrow().as_number()?)))
        }
    };
}

macro_rules! num_binary_op {
    ($id: ident, $func: expr) => {
        pub fn $id(operands: Value) -> Result<Value, RuntimeError> {
            Ok(SymbolTable::number($func(operands.borrow().car()?.borrow().as_number()?, operands.borrow().cadr()?.borrow().as_number()?)))
        }
    };
}

num_binary_op!(add, |x, y| x + y);
num_binary_op!(sub, |x, y| x - y);
num_binary_op!(mul, |x, y| x * y);
num_binary_op!(div, |x, y| x / y);
num_binary_op!(modulo, |x, y| x % y);

num_unary_op!(sin, f64::sin);
num_unary_op!(cos, f64::cos);
num_unary_op!(tan, f64::tan);

num_unary_op!(asin, f64::asin);
num_unary_op!(acos, f64::acos);
num_unary_op!(atan, f64::atan);
num_binary_op!(atan2, f64::atan2);

num_unary_op!(log, f64::ln);
num_unary_op!(exp, f64::exp);


pub fn car(operands: Value) -> Result<Value, RuntimeError> {
    operands.borrow().car()?.borrow().car()
}

pub fn cdr(operands: Value) -> Result<Value, RuntimeError> {
    operands.borrow().car()?.borrow().cdr()
}

pub fn cadr(operands: Value) -> Result<Value, RuntimeError> {
    operands.borrow().car()?.borrow().cadr()
}

pub fn list(operands: Value) -> Result<Value, RuntimeError> {
    Ok(operands)
}

pub fn cons(operands: Value) -> Result<Value, RuntimeError> {
    Ok(Datum::Pair(operands.borrow().car()?, operands.borrow().cadr()?).wrap())
}

// Primitive type predicate

pub fn is_null(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_nil()))
}

pub fn is_boolean(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_boolean()))
}

pub fn is_number(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_number()))
}

pub fn is_string(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_string()))
}

pub fn is_port(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_port()))
}

pub fn is_input_port(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(if let Port::Input(_) = operands.borrow().car()?.borrow().as_port()? { true } else { false }))
}

pub fn is_output_port(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(if let Port::Output(_) = operands.borrow().car()?.borrow().as_port()? { true } else { false }))
}

pub fn is_symbol(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_symbol()))
}

pub fn is_pair(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_pair()))
}

pub fn is_vector(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_vector()))
}


pub fn is_eqv(operands: Value) -> Result<Value, RuntimeError> {
    let a = operands.borrow().car()?;
    let d = operands.borrow().cadr()?;
    if a.borrow().is_number() && d.borrow().is_number() { eq(operands) }
    else if a.borrow().is_character() && d.borrow().is_character() { char_eq(operands) }
    else if a.borrow().is_boolean() && d.borrow().is_boolean() { Ok(SymbolTable::bool(a.borrow().as_boolean()? == d.borrow().as_boolean()?)) }
    else { Ok(SymbolTable::bool(&*a.borrow() as *const _ == &*d.borrow() as *const _)) }
}


pub fn is_eq(operands: Value) -> Result<Value, RuntimeError> {
    let a = operands.borrow().car()?;
    let d = operands.borrow().cadr()?;
    return Ok(SymbolTable::bool(&*a.borrow() as *const _ == &*d.borrow() as *const _));
}

pub fn set_car(operands: Value) -> Result<Value, RuntimeError> {
    let a = operands.borrow().car()?;
    let d = operands.borrow().cadr()?;
    a.borrow_mut().set_car(d)?;    
    Ok(SymbolTable::unspecified())
}

pub fn set_cdr(operands: Value) -> Result<Value, RuntimeError> {
    let a = operands.borrow().car()?;
    let d = operands.borrow().cadr()?;
    a.borrow_mut().set_cdr(d)?;    
    Ok(SymbolTable::unspecified())
}

pub fn number_to_string(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::string(operands.borrow().car()?.borrow().as_number()?.to_string()))
}

pub fn symbol_to_string(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::string(operands.borrow().car()?.borrow().as_symbol()?))
}

pub fn string_to_symbol(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::symbol(operands.borrow().car()?.borrow().as_string()?))
}

pub fn char_eq(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().as_character()? == operands.borrow().cadr()?.borrow().as_character()?))
}


pub fn make_vector(operands: Value) -> Result<Value, RuntimeError> {
    if operands.borrow().len() == 1 {
        Ok(Datum::Vector(iter::repeat(SymbolTable::unspecified()).take(operands.borrow().car()?.borrow().as_number()? as usize).collect()).wrap())
    } else if operands.borrow().len() == 2 {
        Ok(Datum::Vector(iter::repeat(operands.borrow().cadr()?).take(operands.borrow().car()?.borrow().as_number()? as usize).collect()).wrap())
    } else {
        error!("Expected 1 or 2 arguments")
    }
}

pub fn vector_length(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_vector_ref()?.len() as f64))
}

pub fn vector_ref(operands: Value) -> Result<Value, RuntimeError> {
    Ok(operands.borrow().car()?.borrow().as_vector_ref()?[operands.borrow().cadr()?.borrow().as_number()? as usize].clone())
}

pub fn vector_set(operands: Value) -> Result<Value, RuntimeError> {
    let op = operands.clone().borrow().car()?;
    let mut opv = op.borrow_mut();
    let mut vector = opv.as_vector_mut()?;
    vector.push(operands.borrow().cdr()?.borrow().cadr()?);
    vector.swap_remove(operands.borrow().cadr()?.borrow().as_number()? as usize);
    Ok(SymbolTable::unspecified())
}

// Port IO

pub fn open_input_file(operands: Value) -> Result<Value, RuntimeError> {
    Ok(Datum::Port(Port::Input(Rc::new(RefCell::new(File::open(operands.borrow().car()?.borrow().as_string()?).or(error!("unable to open input file"))?)))).wrap())
}

pub fn open_output_file(operands: Value) -> Result<Value, RuntimeError> {
    Ok(Datum::Port(Port::Output(Rc::new(RefCell::new(File::open(operands.borrow().car()?.borrow().as_string()?).or(error!("unable to open output file"))?)))).wrap())
}

pub fn close_input_file(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Port(ref mut p) = *operands.borrow().car()?.borrow_mut() {
        let port = std::mem::replace(p, Port::None);
    }
    Ok(SymbolTable::unspecified())
}

pub fn close_output_file(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Port(ref mut p) = *operands.borrow().car()?.borrow_mut() {
        let port = std::mem::replace(p, Port::None);
    }
    Ok(SymbolTable::unspecified())
}


pub fn read_char(operands: Value) -> Result<Value, RuntimeError> {
    let len = operands.borrow().len();
        match len {
            0 => {
                if let Port::Input(input) = SymbolTable::stdin().borrow().as_port()? {
                    let mut buffer = [0; 1];
                    if let Ok(_) = input.borrow_mut().read(&mut buffer) {
                        Ok(SymbolTable::character(buffer[0] as char))
                    } else {
                        error!("Read from stdin error")
                    }
                } else {
                    error!("Expected input port")
                }
            },
            1 => {
                if let Port::Input(input) = operands.borrow().car()?.borrow().as_port()? {
                    let mut buffer = [0; 1];
                    if let Ok(_) = input.borrow_mut().read(&mut buffer) {
                        Ok(SymbolTable::character(buffer[0] as char))
                    } else {
                        error!("Read error")
                    }
                } else {
                    error!("Expected input port")
                }
            },
            _ => {
                error!("unexpected arguments number in read-char")
            }
        }
}