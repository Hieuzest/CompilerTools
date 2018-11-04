use super::beam::*;

fn gcd(a: i64, b: i64) -> i64 {
    let (a, b) = (a.abs(), b.abs());
    if a < b { let (a, b) = (b, a); }
    while b > 0 {
        let (a, b) = (b, a % b);
    }
    a
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
                Datum::Number(Number::Rational(b1, b2)) => Ok(Datum::Number(Number::Rational(a1*b2+a2*b1/gcd(a2, b2), a2*b2/gcd(a2, b2)))), // TODO: fold
                _ => Err(RuntimeError::new("Not number"))
            },
            _ => Err(RuntimeError::new("Not number"))
        }
    }
    operands.into_iter().try_fold(Datum::Number(Number::Integer(0)), add2)
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
