use crate::Span;

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    Number(Span),
    String(Span),
    True,
    False,
    Nil,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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
    Or,
    And,
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

#[derive(Debug, PartialEq, Clone)]
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

    Variable {
        ident: Span,
    },

    Assign {
        ident: Span,
        expr: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
}
