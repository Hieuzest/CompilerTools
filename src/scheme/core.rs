use super::beam::*;

fn gcd(a: i64, b: i64) -> i64 {
    let (a, b) = (a.abs(), b.abs());
    if a < b { let (a, b) = (b, a); }
    while b > 0 {
        let (a, b) = (b, a % b);
    }
    a
}


pub fn eq(operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    fn eq2(a: Datum, b: Datum) -> Result<Datum, RuntimeError> {
        match a {
            Datum::Number(Number::Integer(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a == b)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean(a as f64 == b)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a * b2 == b1)),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Real(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a == b as f64)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean(a == b)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a * b2 as f64 == b1 as f64)),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Rational(a1, a2)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a1 == b * a2)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean(a1 as f64 == b * a2 as f64)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a1 * b2 == b1 * a2)),
                _ => Err(RuntimeError::new("Not number"))
            },
            _ => Err(RuntimeError::new("Not number"))
        }
    }
    eq2(operands[0].clone(), operands[1].clone())
}


pub fn lt(operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    fn lt2(a: Datum, b: Datum) -> Result<Datum, RuntimeError> {
        match a {
            Datum::Number(Number::Integer(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a < b)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean((a as f64) < b)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a * b2 < b1)),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Real(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a < b as f64)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean(a < b)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a * (b2 as f64) < b1 as f64)),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Rational(a1, a2)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a1 < b * a2)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean((a1 as f64) < b * a2 as f64)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a1 * b2 < b1 * a2)),
                _ => Err(RuntimeError::new("Not number"))
            },
            _ => Err(RuntimeError::new("Not number"))
        }
    }
    lt2(operands[0].clone(), operands[1].clone())
}

pub fn le(operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    fn le2(a: Datum, b: Datum) -> Result<Datum, RuntimeError> {
        match a {
            Datum::Number(Number::Integer(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a <= b)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean(a as f64 <= b)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a * b2 <= b1)),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Real(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a <= b as f64)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean(a <= b)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a * b2 as f64 <= b1 as f64)),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Rational(a1, a2)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Boolean(a1 <= b * a2)),
                Datum::Number(Number::Real(b)) => Ok(Datum::Boolean(a1 as f64 <= b * a2 as f64)),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Boolean(a1 * b2 <= b1 * a2)),
                _ => Err(RuntimeError::new("Not number"))
            },
            _ => Err(RuntimeError::new("Not number"))
        }
    }
    le2(operands[0].clone(), operands[1].clone())
}

pub fn add(operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    fn add2(a: Datum, b: Datum) -> Result<Datum, RuntimeError> {
        match a {
            Datum::Number(Number::Integer(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Integer(a+b))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a as f64+b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational(a*b2+b1, b2))),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Real(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Real(a+b as f64))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a+b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Real(b1 as f64/b2 as f64 + a))),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Rational(a1, a2)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Rational(b*a2+a1, a2))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a1 as f64/a2 as f64 + b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational((a1*b2+a2*b1)/gcd(a2, b2), a2*b2/gcd(a2, b2)))), // TODO: fold
                _ => Err(RuntimeError::new("Not number"))
            },
            _ => Err(RuntimeError::new("Not number"))
        }
    }
    operands.into_iter().try_fold(Datum::Number(Number::Integer(0)), add2)
}


pub fn sub(mut operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    fn sub2(a: Datum, b: Datum) -> Result<Datum, RuntimeError> {
        match a {
            Datum::Number(Number::Integer(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Integer(a-b))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a as f64-b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational(a*b2-b1, b2))),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Real(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Real(a-b as f64))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a-b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Real(b1 as f64/b2 as f64 - a))),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Rational(a1, a2)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Rational(b*a2-a1, a2))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a1 as f64/a2 as f64 - b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational((a1*b2-a2*b1)/gcd(a2, b2), a2*b2/gcd(a2, b2)))), // TODO: fold
                _ => Err(RuntimeError::new("Not number"))
            },
            _ => Err(RuntimeError::new("Not number"))
        }
    }
    let first = operands.remove(0);
    operands.into_iter().try_fold(first, sub2)
}

pub fn mul(operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    fn mul2(a: Datum, b: Datum) -> Result<Datum, RuntimeError> {
        match a {
            Datum::Number(Number::Integer(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Integer(a*b))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a as f64*b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational(a*b1, b2))),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Real(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Real(a*b as f64))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a*b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Real(b1 as f64*a/b2 as f64))),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Rational(a1, a2)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Rational(b*a1, a2))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a1 as f64*b/a2 as f64))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational(a1*b1/gcd(a1*b1, a2*b2), a2*b2/gcd(a1*b1, a2*b2)))), // TODO: fold
                _ => Err(RuntimeError::new("Not number"))
            },
            _ => Err(RuntimeError::new("Not number"))
        }
    }
    operands.into_iter().try_fold(Datum::Number(Number::Integer(1)), mul2)
}

pub fn div(mut operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    fn div2(a: Datum, b: Datum) -> Result<Datum, RuntimeError> {
        match a {
            Datum::Number(Number::Integer(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(if gcd(a, b) == b { Number::Integer(a/b) } else { Number::Rational(a, b) } )),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a as f64/b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational(a*b2, b1))),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Real(a)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Real(a/b as f64))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a/b))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Real(b2 as f64*a/b1 as f64))),
                _ => Err(RuntimeError::new("Not number"))
            },
            Datum::Number(Number::Rational(a1, a2)) => match b {
                Datum::Number(Number::Integer(b)) => Ok(Datum::Number(Number::Rational(a1, a2*b))),
                Datum::Number(Number::Real(b)) => Ok(Datum::Number(Number::Real(a1 as f64*b/a2 as f64))),
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational(a1*b2/gcd(a1*b2, a2*b1), a2*b1/gcd(a1*b2, a2*b1)))), // TODO: fold
                _ => Err(RuntimeError::new("Not number"))
            },
            _ => Err(RuntimeError::new("Not number"))
        }
    }
    let first = operands.remove(0);
    operands.into_iter().try_fold(first, div2)
}

pub fn car(mut operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    Ok(operands.remove(0))
}

pub fn cdr(mut operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    operands.remove(0);
    Ok(Datum::List(operands))
}

pub fn list(operands: Vec<Datum>) -> Result<Datum, RuntimeError> {
    Ok(Datum::List(operands))
}


