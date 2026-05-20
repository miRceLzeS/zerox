use crate::{
    Error, Result,
    parse::{BinaryOperator, Expr, LiteralValue, Stmt, UnaryOperator, stmt},
};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Sub},
};

#[derive(Debug, Clone)]
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

        Err(Error::RuntimeError(format!("Operand must be boolean.")))
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

        if let EvalResult::String(mut s1) = self
            && let EvalResult::String(s2) = rhs
        {
            s1.push_str(&s2);
            return Ok(EvalResult::String(s1));
        }

        Err(Error::RuntimeError(format!(
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

        Err(Error::RuntimeError(format!("Operands must be number.")))
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

        Err(Error::RuntimeError(format!("Operands must be number.")))
    }
}

impl Div for EvalResult {
    type Output = Result<EvalResult>;

    fn div(self, rhs: Self) -> Self::Output {
        if let EvalResult::Number(f1) = self
            && let EvalResult::Number(f2) = rhs
        {
            if f2 == 0.0 {
                return Err(Error::RuntimeError(format!("Can not divide by 0.")));
            }
            return Ok(EvalResult::Number(f1 / f2));
        }

        Err(Error::RuntimeError(format!("Operands must be number.")))
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

pub type Program = Vec<stmt::Stmt>;

#[derive(Debug)]
pub struct Env<'i> {
    vars: HashMap<&'i str, EvalResult>,
}

impl<'i> Env<'i> {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Interpreter<'i> {
    env: Env<'i>,
}

impl<'i> Interpreter<'i> {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    pub fn interpret(&mut self, source: &'i str, prog: Program) -> crate::Result<()> {
        for statement in prog {
            match statement {
                Stmt::VarDeclStmt { ident, init_expr } => {
                    let name = ident.lexeme(source);
                    if name != "" {
                        self.env.vars.insert(name, EvalResult::Nil);
                    }

                    match init_expr {
                        Some(expr) => {
                            let val = self.eval(source, expr)?;
                            self.env.vars.insert(name, val);
                        }
                        None => {}
                    }
                }

                Stmt::ExprStmt(expr) => {
                    self.eval(source, expr)?;
                }

                Stmt::PrintStmt(expr) => {
                    let val = self.eval(source, expr)?;
                    println!("{}", val);
                }

                _ => return Err(Error::RuntimeError(format!("Unknown syntax."))),
            }
        }

        Ok(())
    }

    fn eval(&mut self, source: &'i str, expr: Expr) -> crate::Result<EvalResult> {
        match expr {
            Expr::Literal { value } => match value {
                LiteralValue::Number(tok_span) => {
                    let raw = tok_span.lexeme(source);
                    match raw.parse::<f64>() {
                        Ok(f) => Ok(EvalResult::Number(f)),
                        Err(err) => Err(Error::RuntimeError(format!(
                            "{}:{}: failed to evaluate number '{}', {}",
                            tok_span.start, tok_span.end, raw, err
                        ))),
                    }
                }

                LiteralValue::String(tok_span) => {
                    Ok(EvalResult::String(format!("{}", tok_span.lexeme(source))))
                }

                LiteralValue::True => Ok(EvalResult::Bool(true)),

                LiteralValue::False => Ok(EvalResult::Bool(false)),

                LiteralValue::Nil => Ok(EvalResult::Nil),
            },

            Expr::Group { inner } => self.eval(source, *inner),

            Expr::Unary { op, expr } => match op {
                UnaryOperator::Negative => {
                    let val = self.eval(source, *expr)?;
                    -val
                }
                UnaryOperator::Not => {
                    let val = self.eval(source, *expr)?;
                    !val
                }
            },

            Expr::Binary { left, op, right } => {
                let l_val = self.eval(source, *left)?;
                let r_val = self.eval(source, *right)?;

                match op {
                    BinaryOperator::Add => l_val + r_val,

                    BinaryOperator::Minus => l_val - r_val,

                    BinaryOperator::Multiply => l_val * r_val,

                    BinaryOperator::Divide => l_val / r_val,

                    BinaryOperator::Equal => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val == r_val))
                    }

                    BinaryOperator::NotEqual => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(!(l_val == r_val)))
                    }

                    BinaryOperator::Less => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val < r_val))
                    }

                    BinaryOperator::LessEqual => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val <= r_val))
                    }

                    BinaryOperator::Greater => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val > r_val))
                    }

                    BinaryOperator::GreaterEqual => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val >= r_val))
                    }
                }
            }

            Expr::Variable { ident } => {
                let name = ident.lexeme(source);
                match self.env.vars.get(name) {
                    Some(val) => Ok(val.clone()),
                    None => Err(Error::RuntimeError(format!("Undefined variable {}.", name))),
                }
            }

            Expr::Assign { ident, expr } => {
                let name = ident.lexeme(source);
                let new_val = self.eval(source, *expr)?;

                match self.env.vars.get(name) {
                    Some(_) => {
                        self.env.vars.insert(name, new_val);
                        Ok(EvalResult::Nil)
                    }
                    None => Err(Error::RuntimeError(format!("Undefined variable {}.", name))),
                }
            }
        }
    }
}
