use super::beam::*;
use std::ops;
use std::fmt;
use std::cmp::{Eq, PartialEq, PartialOrd, Ord, Ordering};

#[derive(Debug, Copy, Clone)]
pub struct Rational {
    pub numerator: i64,
    pub denominator: i64,
}

impl Rational {

    fn new(a: i64, b: i64) -> Self {
        let gcd = Rational::gcd(a, b);
        Rational {
            numerator: a / gcd,
            denominator: b / gcd
        }
    }

    fn gcd(a: i64, b: i64) -> i64 {
        let (a, b) = (a.abs(), b.abs());
        let (mut a, mut b) = if a < b { (b, a) } else { (a, b) };
        while b > 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.numerator * other.denominator).partial_cmp(&(self.denominator * other.numerator))
    }
}


impl PartialEq for Rational {
    fn eq(&self, other: &Self) -> bool {
        (self.numerator * other.denominator).eq(&(self.denominator * other.numerator))
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}


impl<'a> From<&'a i64> for Rational {
    fn from(rhs: &'a i64) -> Rational {
        Rational {
            numerator: *rhs,
            denominator: 1
        }
    }
}

impl From<i64> for Rational {
    fn from(rhs: i64) -> Rational {
        Rational {
            numerator: rhs,
            denominator: 1
        }
    }
}

impl ops::Add for Rational {
    type Output = Rational;
    fn add(self, other: Self) -> Self::Output {
        let gcd = Rational::gcd(self.denominator, other.denominator);
        Rational {
            numerator: self.numerator * other.denominator + self.denominator * other.numerator / gcd,
            denominator: self.denominator * other.denominator / gcd,
        }
    }
}

impl ops::Sub for Rational {
    type Output = Rational;
    fn sub(self, other: Self) -> Self::Output {
        let gcd = Rational::gcd(self.denominator, other.denominator);
        Rational {
            numerator: self.numerator * other.denominator - self.denominator * other.numerator / gcd,
            denominator: self.denominator * other.denominator / gcd,
        }
    }
}

impl ops::Mul for Rational {
    type Output = Rational;
    fn mul(self, other: Self) -> Self::Output {
        let gcd = Rational::gcd(self.numerator * other.numerator, self.denominator * other.denominator);
        Rational {
            numerator: self.numerator * other.numerator / gcd,
            denominator: self.denominator * other.denominator / gcd,
        }
    }
}

impl ops::Div for Rational {
    type Output = Rational;
    fn div(self, other: Self) -> Self::Output {
        let gcd = Rational::gcd(self.numerator * other.denominator, self.denominator * other.numerator);
        Rational {
            numerator: self.numerator * other.denominator / gcd,
            denominator: self.denominator * other.numerator / gcd,
        }
    }
}



#[derive(Debug, Copy, Clone)]
pub enum Real {
    Single(f32),
    Double(f64),
}

impl fmt::Display for Real {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Real::Single(x) => write!(f, "{}", x),
            Real::Double(x) => write!(f, "{}", x),
            _ => panic!()
        }
    }
}

impl PartialOrd for Real {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Real::Single(x) => x.partial_cmp(&other.as_single()),
            Real::Double(x) => x.partial_cmp(&other.as_double()),
            _ => panic!()
        }
    }
}



impl PartialEq for Real {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Real::Single(x) => x.eq(&other.as_single()),
            Real::Double(x) => x.eq(&other.as_double()),
            _ => panic!()
        }
    }
}

macro_rules! impl_real {
    ($func: ident) => {
        pub fn $func(self) -> Real {
            match self {
                Real::Single(x) => Real::Single(x.$func()),
                Real::Double(x) => Real::Double(x.$func()),
                _ => panic!()
            }
        }
    };
}

impl Real {

    pub fn as_integer(self) -> i64 {
        match self {
            Real::Single(x) => x as i64,
            Real::Double(x) => x as i64,
            _ => panic!()
        }
    }

    pub fn as_single(self) -> f32 {
        match self {
            Real::Single(x) => x,
            Real::Double(x) => x as f32,
            _ => panic!()
        }
    }

    pub fn as_double(self) -> f64 {
        match self {
            Real::Single(x) => x as f64,
            Real::Double(x) => x,
            _ => panic!()
        }
    }


    pub fn atan2(self, other: Self) -> Real {
        match self {
            Real::Single(x) => Real::Single(x.atan2(other.as_single())),
            Real::Double(x) => Real::Double(x.atan2(other.as_double())),
            _ => panic!()
        }
    }

    impl_real!(sqrt);
    impl_real!(sin);
    impl_real!(cos);
    impl_real!(tan);
    impl_real!(asin);
    impl_real!(acos);
    impl_real!(atan);
    impl_real!(ln);
    impl_real!(exp);

    impl_real!(floor);
    impl_real!(ceil);
    impl_real!(round);
    impl_real!(trunc);

}

impl<'a> From<&'a i64> for Real {
    fn from(rhs: &'a i64) -> Real {
        Real::Double(*rhs as f64)
    }
}

impl From<i64> for Real {
    fn from(rhs: i64) -> Real {
        Real::Double(rhs as f64)
    }
}

impl<'a> From<&'a f32> for Real {
    fn from(rhs: &'a f32) -> Real {
        Real::Single(*rhs)
    }
}

impl From<f32> for Real {
    fn from(rhs: f32) -> Real {
        Real::Single(rhs)
    }
}

impl<'a> From<&'a f64> for Real {
    fn from(rhs: &'a f64) -> Real {
        Real::Double(*rhs)
    }
}

impl From<f64> for Real {
    fn from(rhs: f64) -> Real {
        Real::Double(rhs)
    }
}

impl<'a> From<&'a Rational> for Real {
    fn from(rhs: &'a Rational) -> Real {
        Real::Double(rhs.numerator as f64 / rhs.denominator as f64)
    }
}

impl From<Rational> for Real {
    fn from(rhs: Rational) -> Real {
        Real::Double(rhs.numerator as f64 / rhs.denominator as f64)
    }
}

macro_rules! impl_real_op {
    ($trait: ident, $func: ident) => {
        impl ops::$trait for Real {
            type Output = Real;
            fn $func(self, other: Real) -> Self::Output {
                match self {
                    Real::Single(x) => Real::Single(x.$func(other.as_single())),
                    Real::Double(x) => Real::Double(x.$func(other.as_double())),
                    _ => panic!()
                }
            }
        }
        impl<'a, 'b> ops::$trait<&'a Real> for &'b Real {
            type Output = Real;
            fn $func(self, other: &'a Real) -> Self::Output {
                match self {
                    Real::Single(x) => Real::Single(x.$func(other.as_single())),
                    Real::Double(x) => Real::Double(x.$func(other.as_double())),
                    _ => panic!()
                }
            }
        }
        impl<'a> ops::$trait<&'a Real> for Real {
            type Output = Real;
            fn $func(self, other: &'a Real) -> Self::Output {
                match self {
                    Real::Single(x) => Real::Single(x.$func(other.as_single())),
                    Real::Double(x) => Real::Double(x.$func(other.as_double())),
                    _ => panic!()
                }
            }
        }
        impl<'a> ops::$trait<Real> for &'a Real {
            type Output = Real;
            fn $func(self, other: Real) -> Self::Output {
                match self {
                    Real::Single(x) => Real::Single(x.$func(other.as_single())),
                    Real::Double(x) => Real::Double(x.$func(other.as_double())),
                    _ => panic!()
                }
            }
        }
    };
}

impl_real_op!(Add, add);
impl_real_op!(Sub, sub);
impl_real_op!(Mul, mul);
impl_real_op!(Div, div);


#[derive(Debug, Copy, Clone)]
pub enum Complex {
    Rectangular(Real, Real),
    Polar(Real, Real)
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Complex::Rectangular(r, i) => write!(f, "{}+{}i", r, i),
            Complex::Polar(m, a) => write!(f, "{}*e^i{}", m, a),
            _ => panic!()
        }
    }
}

impl PartialEq for Complex {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Complex::Rectangular(r, i) => r.eq(&other.real_part()) && i.eq(&other.real_part()),
            Complex::Polar(m, a) => m.eq(&other.magnitude()) && a.eq(&other.angle()),
            _ => panic!()
        }
    }
}

impl Complex {
    pub fn real_part(&self) -> Real {
        match self {
            Complex::Rectangular(r, _) => *r,
            Complex::Polar(m, a) => m * a.cos(),
            _ => panic!()
        }
    }

    pub fn imag_part(&self) -> Real {
        match self {
            Complex::Rectangular(_, i) => *i,
            Complex::Polar(m, a) => m * a.sin(),
            _ => panic!()
        }
    }

    pub fn magnitude(&self) -> Real {
        match self {
            Complex::Rectangular(r, i) => (r*r + i*i).sqrt(),
            Complex::Polar(m, a) => *m,
            _ => panic!()
        }
    }

    pub fn angle(&self) -> Real {
        match self {
            Complex::Rectangular(r, i) => r.atan2(*i),
            Complex::Polar(m, a) => *a,
            _ => panic!()
        }
    }
}

impl<'a> From<&'a i64> for Complex {
    fn from(rhs: &'a i64) -> Complex {
        Complex::from(Real::from(*rhs))
    }
}

impl From<i64> for Complex {
    fn from(rhs: i64) -> Complex {
        Complex::from(Real::from(rhs))
    }
}

impl<'a> From<&'a Rational> for Complex {
    fn from(rhs: &'a Rational) -> Complex {
        Complex::from(Real::from(*rhs))
    }
}

impl From<Rational> for Complex {
    fn from(rhs: Rational) -> Complex {
        Complex::from(Real::from(rhs))
    }
}

impl<'a> From<&'a Real> for Complex {
    fn from(rhs: &'a Real) -> Complex {
        Complex::from(Real::from(*rhs))
    }
}

impl From<Real> for Complex {
    fn from(rhs: Real) -> Complex {
        match rhs {
            Real::Double(_) => Complex::Rectangular(rhs, Real::Double(0.0)),
            Real::Single(_) => Complex::Rectangular(rhs, Real::Single(0.0)),
            _ => panic!()
        }
    }
}

impl ops::Add for Complex {
    type Output = Complex;
    fn add(self, other: Self) -> Self::Output {
        match self {
            Complex::Rectangular(r, i) => {
                let (or, oi) = (other.real_part(), other.imag_part());
                Complex::Rectangular(r + or, i + oi)
            },
            Complex::Polar(m, a) => Complex::Rectangular(self.real_part() + other.real_part(), self.imag_part() + other.imag_part()),
            _ => panic!()
        }
    }
}

impl ops::Sub for Complex {
    type Output = Complex;
    fn sub(self, other: Self) -> Self::Output {
        match self {
            Complex::Rectangular(r, i) => {
                let (or, oi) = (other.real_part(), other.imag_part());
                Complex::Rectangular(r - or, i - oi)
            },
            Complex::Polar(m, a) => Complex::Rectangular(self.real_part() - other.real_part(), self.imag_part() - other.imag_part()),
            _ => panic!()
        }
    }
}

impl ops::Mul for Complex {
    type Output = Complex;
    fn mul(self, other: Self) -> Self::Output {
        match self {
            Complex::Rectangular(r, i) => {
                let (or, oi) = (other.real_part(), other.imag_part());
                Complex::Rectangular(r * or - i * oi, r * oi + i * or)
            },
            Complex::Polar(m, a) => Complex::Polar(m * other.magnitude(), a + other.angle()),
            _ => panic!()
        }
    }
}

impl ops::Div for Complex {
    type Output = Complex;
    fn div(self, other: Self) -> Self::Output {
        match self {
            Complex::Rectangular(r, i) => {
                let (or, oi) = (other.real_part(), other.imag_part());
                let m = other.magnitude();
                Complex::Rectangular((r * or + i * oi) / m, (r * oi - i * or) / m)
            },
            Complex::Polar(m, a) => Complex::Polar(m / other.magnitude(), a - other.angle()),
            _ => panic!()
        }
    }
}



#[derive(Debug, Copy, Clone)]
pub enum Number {
    Integer(i64),
    Rational(Rational),
    Real(Real),
    Complex(Complex)
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Number::Integer(x) => write!(f, "{}", x),
            Number::Rational(x) => write!(f, "{}", x),
            Number::Real(x) => write!(f, "{}", x),
            Number::Complex(x) => write!(f, "{}", x),
            _ => panic!()
        }
    }
}


impl Number {

    pub fn as_integer(&self) -> Result<i64, RuntimeError> {
        if let Number::Integer(x) = self { Ok(*x) }
        else { error!("Expected integer") }
    }

    pub fn as_rational(&self) -> Result<Rational, RuntimeError> {
        match self {
            Number::Integer(x) => Ok(Rational::from(x)),
            Number::Rational(x) => Ok(*x),
            _ => error!("Expected real")
        }
    }

    pub fn as_real(&self) -> Result<Real, RuntimeError> {
        match self {
            Number::Integer(x) => Ok(Real::from(x)),
            Number::Rational(x) => Ok(Real::from(x)),
            Number::Real(x) => Ok(*x),
            _ => error!("Expected real")
        }
    }

    pub fn as_complex(&self) -> Result<Complex, RuntimeError> {
        match self {
            Number::Integer(x) => Ok(Complex::from(x)),
            Number::Rational(x) => Ok(Complex::from(x)),
            Number::Real(x) => Ok(Complex::from(x)),
            Number::Complex(x) => Ok(*x),
            _ => error!("Expected real")
        }
    }

}

impl From<i64> for Number {
    fn from(rhs: i64) -> Number {
        Number::Integer(rhs)
    }
}

impl From<Real> for Number {
    fn from(rhs: Real) -> Number {
        Number::Real(rhs)
    }
}

impl From<Rational> for Number {
    fn from(rhs: Rational) -> Number {
        Number::Rational(rhs)
    }
}

impl From<Complex> for Number {
    fn from(rhs: Complex) -> Number {
        Number::Complex(rhs)
    }
}


impl ops::Div for Number {
    type Output = Number;
    fn div(self, other: Self) -> Self::Output {
        match self {
            Number::Integer(x) => {
                match other {
                    Number::Integer(y) => Number::Rational(Rational::new(x, y)),
                    Number::Rational(y) => Number::Rational(Rational::from(x).div(y)),
                    Number::Real(y) => Number::Real(Real::from(x).div(y)),
                    Number::Complex(y) => Number::Complex(Complex::from(Real::from(x)).div(y)),
                }
            },
            Number::Rational(x) => {
                match other {
                    Number::Integer(y) => Number::Rational(x.div(Rational::from(y))),
                    Number::Rational(y) => Number::Rational(x.div(y)),
                    Number::Real(y) => Number::Real(Real::from(x).div(y)),
                    Number::Complex(y) => Number::Complex(Complex::from(x).div(y)),
                }
            },
            Number::Real(x) => {
                match other {
                    Number::Integer(y) => Number::Real(x.div(Real::from(y))),
                    Number::Rational(y) => Number::Real(x.div(Real::from(y))),
                    Number::Real(y) => Number::Real(x.div(y)),
                    Number::Complex(y) => Number::Complex(Complex::from(x).div(y)),
                }
            },
            Number::Complex(x) => {
                match other {
                    Number::Integer(y) => Number::Complex(x.div(Complex::from(y))),
                    Number::Rational(y) => Number::Complex(x.div(Complex::from(y))),
                    Number::Real(y) => Number::Complex(x.div(Complex::from(y))),
                    Number::Complex(y) => Number::Complex(x.div(y)),
                }
            },
            _ => panic!()
        }
    }
}

macro_rules! impl_number_op {
    ($trait: ident, $func: ident) => {
        impl ops::$trait for Number {
            type Output = Number;
            fn $func(self, other: Self) -> Self::Output {
                match self {
                    Number::Integer(x) => {
                        match other {
                            Number::Integer(y) => Number::Integer(x.$func(y)),
                            Number::Rational(y) => Number::Rational(Rational::from(x).$func(y)),
                            Number::Real(y) => Number::Real(Real::from(x).$func(y)),
                            Number::Complex(y) => Number::Complex(Complex::from(Real::from(x)).$func(y)),
                        }
                    },
                    Number::Rational(x) => {
                        match other {
                            Number::Integer(y) => Number::Rational(x.$func(Rational::from(y))),
                            Number::Rational(y) => Number::Rational(x.$func(y)),
                            Number::Real(y) => Number::Real(Real::from(x).$func(y)),
                            Number::Complex(y) => Number::Complex(Complex::from(x).$func(y)),
                        }
                    },
                    Number::Real(x) => {
                        match other {
                            Number::Integer(y) => Number::Real(x.$func(Real::from(y))),
                            Number::Rational(y) => Number::Real(x.$func(Real::from(y))),
                            Number::Real(y) => Number::Real(x.$func(y)),
                            Number::Complex(y) => Number::Complex(Complex::from(x).$func(y)),
                        }
                    },
                    Number::Complex(x) => {
                        match other {
                            Number::Integer(y) => Number::Complex(x.$func(Complex::from(y))),
                            Number::Rational(y) => Number::Complex(x.$func(Complex::from(y))),
                            Number::Real(y) => Number::Complex(x.$func(Complex::from(y))),
                            Number::Complex(y) => Number::Complex(x.$func(y)),
                        }
                    },
                    _ => panic!()
                }
            }
        }

    };
}



impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Number::Integer(x) => {
                match other {
                    Number::Integer(y) => x.partial_cmp(y),
                    Number::Rational(y) => Rational::from(x).partial_cmp(&y),
                    Number::Real(y) => Real::from(x).partial_cmp(&y),
                    Number::Complex(y) => None,
                }
            },
            Number::Rational(x) => {
                match other {
                    Number::Integer(y) => x.partial_cmp(&Rational::from(y)),
                    Number::Rational(y) => x.partial_cmp(&y),
                    Number::Real(y) => Real::from(x).partial_cmp(&y),
                    Number::Complex(y) => None,
                }
            },
            Number::Real(x) => {
                match other {
                    Number::Integer(y) => x.partial_cmp(&Real::from(y)),
                    Number::Rational(y) => x.partial_cmp(&Real::from(y)),
                    Number::Real(y) => x.partial_cmp(&y),
                    Number::Complex(y) => None,
                }
            },
            Number::Complex(x) => None,
            _ => panic!()
        }
    }
}



impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Number::Integer(x) => {
                match other {
                    Number::Integer(y) => x.eq(y),
                    Number::Rational(y) => Rational::from(x).eq(&y),
                    Number::Real(y) => Real::from(x).eq(&y),
                    Number::Complex(y) => Complex::from(Real::from(x)).eq(&y),
                }
            },
            Number::Rational(x) => {
                match other {
                    Number::Integer(y) => x.eq(&Rational::from(y)),
                    Number::Rational(y) => x.eq(&y),
                    Number::Real(y) => Real::from(x).eq(&y),
                    Number::Complex(y) => Complex::from(x).eq(&y),
                }
            },
            Number::Real(x) => {
                match other {
                    Number::Integer(y) => x.eq(&Real::from(y)),
                    Number::Rational(y) => x.eq(&Real::from(y)),
                    Number::Real(y) => x.eq(&y),
                    Number::Complex(y) => Complex::from(x).eq(&y),
                }
            },
            Number::Complex(x) => {
                match other {
                    Number::Integer(y) => x.eq(&Complex::from(y)),
                    Number::Rational(y) => x.eq(&Complex::from(y)),
                    Number::Real(y) => x.eq(&Complex::from(y)),
                    Number::Complex(y) => x.eq(&y),
                }
            },
            _ => panic!()
        }
    }
}


impl_number_op!(Add, add);
impl_number_op!(Sub, sub);
impl_number_op!(Mul, mul);
//impl_number_op!(Div, div);