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

impl EvalResult {
    pub fn as_bool(&self) -> EvalResult {
        if matches!(self, EvalResult::Nil) {
            return EvalResult::Bool(false);
        }

        if let EvalResult::Bool(b) = self {
            return EvalResult::Bool(*b);
        }

        EvalResult::Bool(true)
    }
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
        if let EvalResult::Bool(b) = self.as_bool() {
            return Ok(EvalResult::Bool(!b));
        }

        Err(Error::RuntimeError(format!(
            "Operand can not be evaluated to boolean"
        )))
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
    env: Vec<Env<'i>>,
}

impl<'i> Interpreter<'i> {
    pub fn new() -> Self {
        let mut env = Vec::new();
        env.push(Env::new());
        Self { env }
    }

    pub fn interpret(&mut self, source: &'i str, prog: Program) -> crate::Result<()> {
        for statement in prog {
            match statement {
                Stmt::VarDeclStmt { ident, init_expr } => {
                    let name = ident.lexeme(source);
                    if name != "" {
                        if let Some(env) = self.env.last_mut() {
                            env.vars.insert(name, EvalResult::Nil);
                        }
                    }

                    match init_expr {
                        Some(expr) => {
                            let val = self.eval(source, &expr)?;
                            if let Some(env) = self.env.last_mut() {
                                env.vars.insert(name, val);
                            }
                        }
                        None => {}
                    }
                }

                Stmt::BlockStmt { stmts } => {
                    self.env.push(Env::new());
                    self.interpret(source, stmts)?;
                    self.env.pop();
                }

                Stmt::ExprStmt(expr) => {
                    self.eval(source, &expr)?;
                }

                Stmt::PrintStmt(expr) => {
                    let val = self.eval(source, &expr)?;
                    print!("{}", val);
                }

                Stmt::IfStmt {
                    cond,
                    then_branch,
                    else_branch,
                } => {
                    if let EvalResult::Bool(b) = self.eval(source, &cond)?.as_bool() {
                        if b {
                            let then_prog = vec![*then_branch];
                            self.interpret(source, then_prog)?;
                        } else if let Some(else_stmt) = else_branch {
                            let else_prog = vec![*else_stmt];
                            self.interpret(source, else_prog)?;
                        }
                    }
                }

                Stmt::WhileStmt { cond, body } => {
                    while let EvalResult::Bool(b) = self.eval(source, &cond)?.as_bool()
                        && b
                    {
                        let body_prog = vec![*(body.clone())];
                        self.interpret(source, body_prog)?;
                    }
                }

                Stmt::Unknown(msg) => return Err(Error::RuntimeError(format!("{}", msg))),
            }
        }

        Ok(())
    }

    pub fn eval(&mut self, source: &'i str, expr: &Expr) -> crate::Result<EvalResult> {
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

            Expr::Group { inner } => self.eval(source, inner),

            Expr::Unary { op, expr } => match op {
                UnaryOperator::Negative => {
                    let val = self.eval(source, expr)?;
                    -val
                }
                UnaryOperator::Not => {
                    let val = self.eval(source, expr)?;
                    !val
                }
            },

            Expr::Binary { left, op, right } => {
                let l_val = self.eval(source, left)?;

                match op {
                    BinaryOperator::Add => {
                        let r_val = self.eval(source, right)?;
                        l_val + r_val
                    }

                    BinaryOperator::Minus => {
                        let r_val = self.eval(source, right)?;
                        l_val - r_val
                    }

                    BinaryOperator::Multiply => {
                        let r_val = self.eval(source, right)?;
                        l_val * r_val
                    }

                    BinaryOperator::Divide => {
                        let r_val = self.eval(source, right)?;
                        l_val / r_val
                    }

                    BinaryOperator::Equal => {
                        let r_val = self.eval(source, right)?;
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val == r_val))
                    }

                    BinaryOperator::NotEqual => {
                        let r_val = self.eval(source, right)?;
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(!(l_val == r_val)))
                    }

                    BinaryOperator::Less => {
                        let r_val = self.eval(source, right)?;
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val < r_val))
                    }

                    BinaryOperator::LessEqual => {
                        let r_val = self.eval(source, right)?;
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val <= r_val))
                    }

                    BinaryOperator::Greater => {
                        let r_val = self.eval(source, right)?;
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val > r_val))
                    }

                    BinaryOperator::GreaterEqual => {
                        let r_val = self.eval(source, right)?;
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::RuntimeError(format!("Uncomparable.")));
                        }

                        Ok(EvalResult::Bool(l_val >= r_val))
                    }

                    BinaryOperator::Or => match l_val.as_bool() {
                        EvalResult::Bool(b1) => {
                            if b1 {
                                return Ok(l_val);
                            }

                            self.eval(source, right)
                        }
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "Unevaluable left expression of 'or'"
                            )));
                        }
                    },

                    BinaryOperator::And => match l_val.as_bool() {
                        EvalResult::Bool(b1) => {
                            if !b1 {
                                return Ok(l_val);
                            }

                            self.eval(source, right)
                        }
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "Unevaluable left expression of 'and'"
                            )));
                        }
                    },
                }
            }

            Expr::Variable { ident } => {
                let name = ident.lexeme(source);
                for env in self.env.iter().rev() {
                    if let Some(val) = env.vars.get(name) {
                        return Ok(val.clone());
                    }
                }

                Err(Error::RuntimeError(format!("Undefined variable {}.", name)))
            }

            Expr::Assign { ident, expr } => {
                let name = ident.lexeme(source);
                let new_val = self.eval(source, expr)?;

                for env in self.env.iter_mut() {
                    if let Some(_) = env.vars.get(name) {
                        env.vars.insert(name, new_val);
                        return Ok(EvalResult::Nil);
                    }
                }

                Err(Error::RuntimeError(format!("Undefined variable {}.", name)))
            }
        }
    }
}
