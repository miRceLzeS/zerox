use crate::{Error, Result};
use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Sub},
};

#[derive(Debug)]
pub enum EvalResult {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl Display for EvalResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalResult::Number(val) => write!(f, "{}", val),
            EvalResult::String(val) => write!(f, "{}", val),
            EvalResult::Bool(val) => write!(f, "{}", val),
            EvalResult::Nil => write!(f, "nil"),
        }
    }
}

impl Neg for EvalResult {
    type Output = Result<EvalResult>;

    fn neg(self) -> Self::Output {
        if let EvalResult::Number(f) = self {
            return Ok(EvalResult::Number(-f));
        }

        Err(Error::EvalError(format!("Operand must be boolean.")))
    }
}

impl Not for EvalResult {
    type Output = Result<EvalResult>;

    fn not(self) -> Self::Output {
        if matches!(self, EvalResult::Nil) {
            // null => false (only in bool exprission)
            // !null => true
            return Ok(EvalResult::Bool(true));
        }

        if let EvalResult::Bool(b) = self {
            Ok(EvalResult::Bool(!b))
        } else {
            // x is neither null nor bool => true
            // !x => false
            Ok(EvalResult::Bool(false))
        }
    }
}

impl Add for EvalResult {
    type Output = Result<EvalResult>;

    fn add(self, rhs: Self) -> Self::Output {
        if let EvalResult::Number(f1) = self
            && let EvalResult::Number(f2) = rhs
        {
            return Ok(EvalResult::Number(f1 + f2));
        }

        if let EvalResult::String(s1) = self
            && let EvalResult::String(s2) = rhs
        {
            return Ok(EvalResult::String(format!("{}{}", s1, s2)));
        }

        Err(Error::EvalError(format!(
            "Operands must be number or string."
        )))
    }
}

impl Sub for EvalResult {
    type Output = Result<EvalResult>;

    fn sub(self, rhs: Self) -> Self::Output {
        if let EvalResult::Number(f1) = self
            && let EvalResult::Number(f2) = rhs
        {
            return Ok(EvalResult::Number(f1 - f2));
        }

        Err(Error::EvalError(format!("Operands must be number.")))
    }
}

impl Mul for EvalResult {
    type Output = Result<EvalResult>;

    fn mul(self, rhs: Self) -> Self::Output {
        if let EvalResult::Number(f1) = self
            && let EvalResult::Number(f2) = rhs
        {
            return Ok(EvalResult::Number(f1 * f2));
        }

        Err(Error::EvalError(format!("Operands must be number.")))
    }
}

impl Div for EvalResult {
    type Output = Result<EvalResult>;

    fn div(self, rhs: Self) -> Self::Output {
        if let EvalResult::Number(f1) = self
            && let EvalResult::Number(f2) = rhs
        {
            if f2 == 0.0 {
                return Err(Error::EvalError(format!("Can not divide by 0.")));
            }
            return Ok(EvalResult::Number(f1 / f2));
        }

        Err(Error::EvalError(format!("Operands must be number.")))
    }
}

impl PartialEq for EvalResult {
    fn eq(&self, other: &Self) -> bool {
        if matches!(self, EvalResult::Nil) && matches!(other, EvalResult::Nil) {
            return true;
        } else if matches!(self, EvalResult::Nil) {
            return false;
        }

        if let EvalResult::Number(f1) = self
            && let EvalResult::Number(f2) = other
        {
            return f1 == f2;
        }

        if let EvalResult::String(s1) = self
            && let EvalResult::String(s2) = other
        {
            return s1 == s2;
        }

        if let EvalResult::Bool(b1) = self
            && let EvalResult::Bool(b2) = other
        {
            return b1 == b2;
        }

        false
    }
}

impl PartialOrd for EvalResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if matches!(self, EvalResult::Nil) && matches!(other, EvalResult::Nil) {
            return Some(Ordering::Equal);
        }

        if let EvalResult::Number(f1) = self
            && let EvalResult::Number(f2) = other
        {
            return f1.partial_cmp(f2);
        }

        None
    }
}

pub trait Evaluator {
    fn eval(&self, source: &str) -> crate::Result<EvalResult>;
}
