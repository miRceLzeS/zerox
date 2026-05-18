use crate::{Error, EvalResult, Span};

#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    Number(Span),
    String(Span),
    True,
    False,
    Nil,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Negative,
    Not,
}

impl TryFrom<super::TokenKind> for UnaryOperator {
    type Error = &'static str;

    fn try_from(value: super::TokenKind) -> Result<Self, Self::Error> {
        match value {
            super::TokenKind::MINUS => Ok(Self::Negative),
            super::TokenKind::BANG => Ok(Self::Not),
            _ => Err("expect unary operator"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Minus,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl TryFrom<super::TokenKind> for BinaryOperator {
    type Error = &'static str;

    fn try_from(value: super::TokenKind) -> Result<Self, Self::Error> {
        match value {
            super::TokenKind::EEQUAL => Ok(Self::Equal),
            super::TokenKind::BEQUAL => Ok(Self::NotEqual),
            super::TokenKind::LANGLE => Ok(Self::Less),
            super::TokenKind::LANGLEEQUAL => Ok(Self::LessEqual),
            super::TokenKind::RANGLE => Ok(Self::Greater),
            super::TokenKind::RANGLEEQUAL => Ok(Self::GreaterEqual),
            super::TokenKind::PLUS => Ok(Self::Add),
            super::TokenKind::MINUS => Ok(Self::Minus),
            super::TokenKind::STAR => Ok(Self::Multiply),
            super::TokenKind::SLASH => Ok(Self::Divide),
            _ => Err("expected binary operator"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal {
        value: LiteralValue,
    },

    Group {
        inner: Box<Expr>,
    },

    Unary {
        op: UnaryOperator,
        expr: Box<Expr>,
    },

    Binary {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
}

impl crate::Evaluator for Expr {
    fn eval(&self, source: &str) -> crate::Result<crate::EvalResult> {
        match self {
            Expr::Literal { value } => match value {
                LiteralValue::Number(tok_span) => {
                    let raw = tok_span.lexeme(source);
                    match raw.parse::<f64>() {
                        Ok(f) => Ok(EvalResult::Number(f)),
                        Err(err) => Err(Error::EvalError(format!(
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

            Expr::Group { inner } => inner.eval(source),

            Expr::Unary { op, expr } => match op {
                UnaryOperator::Negative => {
                    let val = expr.eval(source)?;
                    -val
                }
                UnaryOperator::Not => {
                    let val = expr.eval(source)?;
                    !val
                }
            },

            Expr::Binary { left, op, right } => {
                let l_val = left.eval(source)?;
                let r_val = right.eval(source)?;

                match op {
                    BinaryOperator::Add => l_val + r_val,

                    BinaryOperator::Minus => l_val - r_val,

                    BinaryOperator::Multiply => l_val * r_val,

                    BinaryOperator::Divide => l_val / r_val,

                    BinaryOperator::Equal => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::EvalError(format!("uncomparable")));
                        }

                        Ok(EvalResult::Bool(l_val == r_val))
                    }

                    BinaryOperator::NotEqual => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::EvalError(format!("uncomparable")));
                        }

                        Ok(EvalResult::Bool(!(l_val == r_val)))
                    }

                    BinaryOperator::Less => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::EvalError(format!("uncomparable")));
                        }

                        Ok(EvalResult::Bool(l_val < r_val))
                    }

                    BinaryOperator::LessEqual => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::EvalError(format!("uncomparable")));
                        }

                        Ok(EvalResult::Bool(l_val <= r_val))
                    }

                    BinaryOperator::Greater => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::EvalError(format!("uncomparable")));
                        }

                        Ok(EvalResult::Bool(l_val > r_val))
                    }

                    BinaryOperator::GreaterEqual => {
                        if l_val.partial_cmp(&r_val).is_none() {
                            return Err(Error::EvalError(format!("uncomparable")));
                        }

                        Ok(EvalResult::Bool(l_val >= r_val))
                    }
                }
            }
        }
    }
}
