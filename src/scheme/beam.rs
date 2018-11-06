use std::str::FromStr;

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

    pub fn number_parse_error(num: impl Into<String>) -> Self {
        Self::new(format!("Number Parse Error when parsing {:}", num.into()))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Number {
    Integer(i64),
    Real(f64),
    Rational(i64, i64),
    Complex(f64, f64)
}

impl FromStr for Number {
    type Err = RuntimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('/') {
            let n1 = s.splitn(2, '/').next().unwrap();
            let n2 = s.splitn(2, '/').skip(1).next().unwrap();
            Ok(Number::Rational(n1.parse().map_err(|_| RuntimeError::number_parse_error(n1))?, n2.parse().map_err(|_| RuntimeError::number_parse_error(n2))?))
        } else if s[1..].contains('+') && s.ends_with('i') {
            let n1 = s.rsplitn(2, '+').skip(1).next().unwrap();
            let n2 = s[..s.len()-1].rsplitn(2, '+').next().unwrap();
            Ok(Number::Complex(n1.parse().map_err(|_| RuntimeError::number_parse_error(n1))?, n2.parse().map_err(|_| RuntimeError::number_parse_error(n2))?))
        } else if s[1..].contains('-') && s.ends_with('i') {
            let n1 = s.rsplitn(2, '-').skip(1).next().unwrap();
            let n2 = s[..s.len()-1].rsplitn(2, '-').next().unwrap();
            Ok(Number::Complex(n1.parse().map_err(|_| RuntimeError::number_parse_error(n1))?, -n2.parse().map_err(|_| RuntimeError::number_parse_error(n2))?))
        } else if s.contains('.') {
            Ok(Number::Real(s.parse().map_err(|_| RuntimeError::number_parse_error(s))?))
        } else {
            Ok(Number::Integer(s.parse().map_err(|_| RuntimeError::number_parse_error(s))?))
        }
    }

}

#[derive(Debug, Copy, Clone)]
pub enum AbbrevPrefix {
    Quote,
    Template,
    Comma,
    CommaSplicing,
}

#[derive(Debug, Clone)]
pub enum Datum {
    SpecialForm(SpecialForm),
    Boolean(bool),
    Number(Number),
    Character(char),
    String(String),
    Symbol(String),
    List(Vec<Datum>),
    Pair(Box<Datum>, Box<Datum>),
    Abbreviation(AbbrevPrefix, Box<Datum>),
    Vector(Vec<Datum>),
    Builtin(Box<fn(Vec<Datum>) -> Result<Datum, RuntimeError>>),
    Lambda(LambdaExpression),
    Syntax(SyntaxRule),
}

impl Datum {
    pub fn new() -> Self {
        Datum::List(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct LambdaExpression {
    pub formals: Vec<String>,
    pub expr: Box<Datum>
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
    Let,
    Letstar,
    Letrec,
    And,
    Or,
    Cond,
    If,
    Quote,
    DefineSyntax
}