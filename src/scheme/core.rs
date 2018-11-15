use super::beam::*;
//use crate::lexer;
//use super::parser;
use super::number::*;
use super::symbol::*;
use std::f64;
use std::iter;
use std::rc::Rc;
use std::cell::RefCell;
use std::io;
use std::io::prelude::*;
use std::fs::File;


pub fn not(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Boolean(b) = *operands.borrow().car()?.borrow() {
        return Ok(SymbolTable::bool(!b));
    }
    Err(RuntimeError::new("Not boolean"))
}


// Primitive numericial procedures
//
pub fn eq(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().as_number()? == operands.borrow().cadr()?.borrow().as_number()?))
}

pub fn lt(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().as_number()? < operands.borrow().cadr()?.borrow().as_number()?))
}

pub fn le(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().as_number()? <= operands.borrow().cadr()?.borrow().as_number()?))
}

macro_rules! real_unary_op {
    ($id: ident, $func: ident) => {
        pub fn $id(operands: Value) -> Result<Value, RuntimeError> {
            Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_real()?.$func()))
        }
    };
}

macro_rules! real_binary_op {
    ($id: ident, $func: ident) => {
        pub fn $id(operands: Value) -> Result<Value, RuntimeError> {
            Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_real()?.$func(operands.borrow().cadr()?.borrow().as_number()?.as_real()?)))
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

real_unary_op!(sin, sin);
real_unary_op!(cos, cos);
real_unary_op!(tan, tan);

real_unary_op!(asin, asin);
real_unary_op!(acos, acos);
real_unary_op!(atan, atan);
real_binary_op!(atan2, atan2);

real_unary_op!(log, ln);
real_unary_op!(exp, exp);

pub fn quotient(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_integer()? / operands.borrow().cadr()?.borrow().as_number()?.as_integer()?))
}

pub fn remainder(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_integer()? % operands.borrow().cadr()?.borrow().as_number()?.as_integer()?))
}

pub fn modulo(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_integer()?.mod_euc(operands.borrow().cadr()?.borrow().as_number()?.as_integer()?)))
}

pub fn floor(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_real()?.floor().as_integer()))
}

pub fn ceiling(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_real()?.ceil().as_integer()))
}

pub fn round(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_real()?.round().as_integer()))
}

pub fn truncate(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_real()?.trunc().as_integer()))
}

pub fn numerator(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_rational()?.numerator))
}

pub fn denominator(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_rational()?.denominator))
}

pub fn make_rectangular(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(Complex::Rectangular(operands.borrow().car()?.borrow().as_number()?.as_real()?, operands.borrow().cadr()?.borrow().as_number()?.as_real()?)))
}

pub fn make_polar(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(Complex::Polar(operands.borrow().car()?.borrow().as_number()?.as_real()?, operands.borrow().cadr()?.borrow().as_number()?.as_real()?)))
}

pub fn real_part(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_complex()?.real_part()))
}

pub fn imag_part(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_complex()?.imag_part()))
}

pub fn magnitude(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_complex()?.magnitude()))
}

pub fn angle(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_number()?.as_complex()?.angle()))
}

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

pub fn is_integer(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_number() && operands.borrow().car()?.borrow().as_number()?.as_integer().is_ok()))
}

pub fn is_rational(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_number() && operands.borrow().car()?.borrow().as_number()?.as_rational().is_ok()))
}

pub fn is_real(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_number() && operands.borrow().car()?.borrow().as_number()?.as_real().is_ok()))
}

pub fn is_complex(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_number() && operands.borrow().car()?.borrow().as_number()?.as_complex().is_ok()))
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

pub fn is_list(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_list()))
}

pub fn is_pair(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_pair()))
}

pub fn is_vector(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_vector()))
}

pub fn is_procedure(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_procedure()))
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

pub fn char_to_integer(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_character()? as u8 as i64))
}

pub fn integer_to_char(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::character(operands.borrow().car()?.borrow().as_number()?.as_integer()? as u8 as char))
}

pub fn char_eq(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().as_character()? == operands.borrow().cadr()?.borrow().as_character()?))
}


pub fn make_vector(operands: Value) -> Result<Value, RuntimeError> {
    if operands.borrow().len() == 1 {
        Ok(Datum::Vector(iter::repeat(SymbolTable::unspecified()).take(operands.borrow().car()?.borrow().as_number()?.as_integer()? as usize).collect()).wrap())
    } else if operands.borrow().len() == 2 {
        Ok(Datum::Vector(iter::repeat(operands.borrow().cadr()?).take(operands.borrow().car()?.borrow().as_number()?.as_integer()? as usize).collect()).wrap())
    } else {
        error!("Expected 1 or 2 arguments")
    }
}

pub fn vector_length(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(Number::Integer(operands.borrow().car()?.borrow().as_vector_ref()?.len() as i64)))
}

pub fn vector_ref(operands: Value) -> Result<Value, RuntimeError> {
    Ok(operands.borrow().car()?.borrow().as_vector_ref()?[operands.borrow().cadr()?.borrow().as_number()?.as_integer()? as usize].clone())
}

pub fn vector_set(operands: Value) -> Result<Value, RuntimeError> {
    let op = operands.clone().borrow().car()?;
    let mut opv = op.borrow_mut();
    let mut vector = opv.as_vector_mut()?;
    vector.push(operands.borrow().cdr()?.borrow().cadr()?);
    vector.swap_remove(operands.borrow().cadr()?.borrow().as_number()?.as_integer()? as usize);
    Ok(SymbolTable::unspecified())
}


pub fn make_string(operands: Value) -> Result<Value, RuntimeError> {
    if operands.borrow().len() == 1 {
        Ok(Datum::String(iter::repeat(0 as char).take(operands.borrow().car()?.borrow().as_number()?.as_integer()? as usize).collect()).wrap())
    } else if operands.borrow().len() == 2 {
        Ok(Datum::String(iter::repeat(operands.borrow().cadr()?.borrow().as_character()?).take(operands.borrow().car()?.borrow().as_number()?.as_integer()? as usize).collect()).wrap())
    } else {
        error!("Expected 1 or 2 arguments")
    }
}

pub fn string_length(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::number(operands.borrow().car()?.borrow().as_string()?.len() as i64))
}

pub fn string_ref(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::character(operands.borrow().car()?.borrow().as_string()?[operands.borrow().cadr()?.borrow().as_number()?.as_integer()? as usize..operands.borrow().cadr()?.borrow().as_number()?.as_integer()? as usize+1].chars().next().unwrap()))
}

pub fn string_set(operands: Value) -> Result<Value, RuntimeError> {
    let op = operands.clone().borrow().car()?;
    let mut opv = op.borrow_mut();
    let mut string = opv.as_string_mut()?;
    let index = operands.borrow().cadr()?.borrow().as_number()?.as_integer()? as usize;
    string.remove(index);
    string.insert(index, operands.borrow().cdr()?.borrow().cadr()?.borrow().as_character()?);
    Ok(SymbolTable::unspecified())
}

// Port IO

pub fn open_input_file(operands: Value) -> Result<Value, RuntimeError> {
    Ok(Datum::Port(Port::Input(Rc::new(RefCell::new(File::open(operands.borrow().car()?.borrow().as_string()?).or(error!("unable to open input file"))?)))).wrap())
}

pub fn open_output_file(operands: Value) -> Result<Value, RuntimeError> {
    Ok(Datum::Port(Port::Output(Rc::new(RefCell::new(File::create(operands.borrow().car()?.borrow().as_string()?).or(error!("unable to open output file"))?)))).wrap())
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
                if let Port::Input(f) = SymbolTable::stdin().borrow().as_port()? {
                    let mut buffer = [0; 1];
                    if let Ok(_) = f.borrow_mut().read(&mut buffer) {
                        Ok(SymbolTable::character(buffer[0] as char))
                    } else {
                        error!("Read from stdin error")
                    }
                } else {
                    error!("Expected input port")
                }
            },
            1 => {
                if let Port::Input(f) = operands.borrow().car()?.borrow().as_port()? {
                    let mut buffer = [0; 1];
                    if let Ok(_) = f.borrow_mut().read(&mut buffer) {
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

pub fn write_char(operands: Value) -> Result<Value, RuntimeError> {
    let len = operands.borrow().len();
    match len {
        1 => {
            if let Port::Output(f) = SymbolTable::stdout().borrow().as_port()? {
                let mut buffer = [operands.borrow().car()?.borrow().as_character()? as u8; 1];
                if let Ok(_) = f.borrow_mut().write(&mut buffer) {
                    Ok(SymbolTable::unspecified())
                } else {
                    error!("Write to stdout error")
                }
            } else {
                error!("Expected output port")
            }
        },
        2 => {
            if let Port::Output(f) = operands.borrow().cadr()?.borrow().as_port()? {
                let mut buffer = [operands.borrow().car()?.borrow().as_character()? as u8; 1];
                if let Ok(_) = f.borrow_mut().write(&mut buffer) {
                    Ok(SymbolTable::unspecified())
                } else {
                    error!("Write error")
                }
            } else {
                error!("Expected output port")
            }
        },
        _ => {
            error!("unexpected arguments number in write-char")
        }
    }
}

//pub fn load(operands: Value) -> Result<Value, RuntimeError> {
//    let mut f = File::open(operands.borrow().car()?.borrow().as_string()?).or(error!("unable to open input file"))?;
//    let mut contents = String::new();
//    f.read_to_string(&mut contents).or(error!("Read error"))?;
//
//    Ok(Datum::Port(Port::Input(Rc::new(RefCell::new(File::open(operands.borrow().car()?.borrow().as_string()?).or(error!("unable to open input file"))?)))).wrap())
//}
