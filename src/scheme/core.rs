use super::beam::*;
use super::symbol::*;

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

pub fn eq(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Number(x) = *operands.borrow().car()?.borrow() {
        if let Datum::Number(y) = *operands.borrow().cadr()?.borrow() {
            return Ok(SymbolTable::bool(x == y));
        }
    }
    Err(RuntimeError::new("Not numericial"))
}


pub fn lt(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Number(x) = *operands.borrow().car()?.borrow() {
        if let Datum::Number(y) = *operands.borrow().cadr()?.borrow() {
            return Ok(SymbolTable::bool(x < y));
        }
    }
    Err(RuntimeError::new("Not numericial"))
}


pub fn le(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Number(x) = *operands.borrow().car()?.borrow() {
        if let Datum::Number(y) = *operands.borrow().cadr()?.borrow() {
            return Ok(SymbolTable::bool(x <= y));
        }
    }
    Err(RuntimeError::new("Not numericial"))
}

pub fn add(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Number(x) = *operands.borrow().car()?.borrow() {
        if let Datum::Number(y) = *operands.borrow().cadr()?.borrow() {
            return Ok(SymbolTable::number(x + y));
        }
    }
    Err(RuntimeError::new("Not numericial"))
}
pub fn sub(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Number(x) = *operands.borrow().car()?.borrow() {
        if let Datum::Number(y) = *operands.borrow().cadr()?.borrow() {
            return Ok(SymbolTable::number(x - y));
        }
    }
    Err(RuntimeError::new("Not numericial"))
}

pub fn mul(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Number(x) = *operands.borrow().car()?.borrow() {
        if let Datum::Number(y) = *operands.borrow().cadr()?.borrow() {
            return Ok(SymbolTable::number(x * y));
        }
    }
    Err(RuntimeError::new("Not numericial"))
}

pub fn div(operands: Value) -> Result<Value, RuntimeError> {
    if let Datum::Number(x) = *operands.borrow().car()?.borrow() {
        if let Datum::Number(y) = *operands.borrow().cadr()?.borrow() {
            return Ok(SymbolTable::number(x / y));
        }
    }
    Err(RuntimeError::new("Not numericial"))
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

pub fn is_null(operands: Value) -> Result<Value, RuntimeError> {
    Ok(SymbolTable::bool(operands.borrow().car()?.borrow().is_nil()))
}

pub fn is_eq(operands: Value) -> Result<Value, RuntimeError> {
    let a = operands.borrow().car()?;
    let d = operands.borrow().cadr()?;
    return Ok(SymbolTable::bool(&*a.borrow() as *const _ == &*d.borrow() as *const _));
}