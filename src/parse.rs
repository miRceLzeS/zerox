pub mod expr;
pub mod stmt;

pub use crate::lex::{Token, TokenKind, Tokens};
pub use crate::{Error, Result};
pub use expr::{BinaryOperator, Expr, LiteralValue, UnaryOperator};

#[derive(Debug)]
pub struct Parser {
    current: usize,
    tokens: Tokens,
}

impl Parser {
    pub fn new(tokens: Tokens) -> Self {
        Self { current: 0, tokens }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        if let Some(expr) = self.parse_expression() {
            return Ok(expr);
        }

        let line = self.peek().unwrap().line;
        Err(Error::ParseError(format!("line {}: unknown syntax", line)))
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Option<Expr> {
        let mut expr = self.parse_comparison()?;

        loop {
            let tok = self.peek()?;
            if tok.kind == TokenKind::BEQUAL || tok.kind == TokenKind::EEQUAL {
                let op: BinaryOperator = tok.kind.try_into().ok()?;
                self.advance();

                let rhs = self.parse_comparison()?;

                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut expr = self.parse_term()?;

        loop {
            let tok = self.peek()?;
            if tok.kind == TokenKind::LANGLE
                || tok.kind == TokenKind::LANGLEEQUAL
                || tok.kind == TokenKind::RANGLE
                || tok.kind == TokenKind::RANGLEEQUAL
            {
                let op: BinaryOperator = tok.kind.try_into().ok()?;
                self.advance();

                let rhs = self.parse_term()?;

                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_term(&mut self) -> Option<Expr> {
        let mut expr = self.parse_factory()?;

        loop {
            let tok = self.peek()?;
            if tok.kind == TokenKind::MINUS || tok.kind == TokenKind::PLUS {
                let op: BinaryOperator = tok.kind.try_into().ok()?;
                self.advance();

                let rhs = self.parse_factory()?;

                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_factory(&mut self) -> Option<Expr> {
        let mut expr = self.parse_unary()?;

        loop {
            let tok = self.peek()?;
            if tok.kind == TokenKind::STAR || tok.kind == TokenKind::SLASH {
                let op: BinaryOperator = tok.kind.try_into().ok()?;
                self.advance();

                let rhs = self.parse_unary()?;

                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        let tok = self.peek()?;
        if tok.kind == TokenKind::BANG || tok.kind == TokenKind::MINUS {
            let op: UnaryOperator = tok.kind.try_into().ok()?;
            self.advance();

            let expr = self.parse_unary()?;
            return Some(Expr::Unary {
                op,
                expr: Box::new(expr),
            });
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let tok = self.peek()?;
        match tok.kind {
            TokenKind::TRUE => {
                self.advance();
                Some(Expr::Literal {
                    value: LiteralValue::True,
                })
            }

            TokenKind::FALSE => {
                self.advance();
                Some(Expr::Literal {
                    value: LiteralValue::False,
                })
            }

            TokenKind::NIL => {
                self.advance();
                Some(Expr::Literal {
                    value: LiteralValue::Nil,
                })
            }

            TokenKind::NUMBER => {
                self.advance();
                Some(Expr::Literal {
                    value: LiteralValue::Number(tok.span),
                })
            }

            TokenKind::STRING => {
                self.advance();
                Some(Expr::Literal {
                    value: LiteralValue::String(tok.span),
                })
            }

            TokenKind::LPAREN => {
                self.advance();
                let expr = self.parse_expression()?;

                match self.advance()?.kind {
                    TokenKind::RPAREN => Some(Expr::Group {
                        inner: Box::new(expr),
                    }),
                    _ => None,
                }
            }

            _ => None,
        }
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.current)?;
        self.current += 1;
        Some(*tok)
    }

    fn peek(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.current)?;
        Some(*tok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Span;

    #[test]
    fn test_parse_primary() {
        let mut l = crate::Lexer::new("true false nil 1 (1) \"1\"");
        let tokens = l.lex().unwrap();
        let mut p = Parser::new(tokens);

        assert_eq!(
            Some(Expr::Literal {
                value: LiteralValue::True
            }),
            p.parse_primary()
        );
        assert_eq!(
            Some(Expr::Literal {
                value: LiteralValue::False
            }),
            p.parse_primary()
        );
        assert_eq!(
            Some(Expr::Literal {
                value: LiteralValue::Nil
            }),
            p.parse_primary()
        );
        assert_eq!(
            Some(Expr::Literal {
                value: LiteralValue::Number(Span::new(15, 16))
            }),
            p.parse_primary()
        );
        assert_eq!(
            Some(Expr::Group {
                inner: Box::new(Expr::Literal {
                    value: LiteralValue::Number(Span::new(18, 19))
                })
            }),
            p.parse_primary()
        );

        assert_eq!(
            Some(Expr::Literal {
                value: LiteralValue::String(Span::new(21, 24))
            }),
            p.parse_primary()
        );
    }

    #[test]
    fn test_parse_unary() {
        let mut l = crate::Lexer::new("-1 (!2)");
        let tokens = l.lex().unwrap();
        let mut p = Parser::new(tokens);

        assert_eq!(
            Some(Expr::Unary {
                op: UnaryOperator::Negative,
                expr: Box::new(Expr::Literal {
                    value: LiteralValue::Number(Span::new(1, 2))
                })
            }),
            p.parse_unary()
        );
        assert_eq!(
            Some(Expr::Group {
                inner: Box::new(Expr::Unary {
                    op: UnaryOperator::Not,
                    expr: Box::new(Expr::Literal {
                        value: LiteralValue::Number(Span::new(5, 6))
                    })
                })
            }),
            p.parse_unary()
        );
    }

    #[test]
    fn test_parse_factory() {
        let mut l = crate::Lexer::new("1 * (-2)");
        let tokens = l.lex().unwrap();
        let mut p = Parser::new(tokens);

        assert_eq!(
            Some(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: LiteralValue::Number(Span::new(0, 1))
                }),
                op: BinaryOperator::Multiply,
                right: Box::new(Expr::Group {
                    inner: Box::new(Expr::Unary {
                        op: UnaryOperator::Negative,
                        expr: Box::new(Expr::Literal {
                            value: LiteralValue::Number(Span::new(6, 7))
                        })
                    })
                })
            }),
            p.parse_factory()
        );
    }

    #[test]
    fn test_parse_term() {
        let mut l = crate::Lexer::new("1 * (-2) + -3 / -4");
        let tokens = l.lex().unwrap();
        let mut p = Parser::new(tokens);

        assert_eq!(
            Some(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: LiteralValue::Number(Span::new(0, 1))
                    }),
                    op: BinaryOperator::Multiply,
                    right: Box::new(Expr::Group {
                        inner: Box::new(Expr::Unary {
                            op: UnaryOperator::Negative,
                            expr: Box::new(Expr::Literal {
                                value: LiteralValue::Number(Span::new(6, 7))
                            })
                        })
                    })
                }),
                op: BinaryOperator::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Unary {
                        op: UnaryOperator::Negative,
                        expr: Box::new(Expr::Literal {
                            value: LiteralValue::Number(Span::new(12, 13))
                        })
                    }),
                    op: BinaryOperator::Divide,
                    right: Box::new(Expr::Unary {
                        op: UnaryOperator::Negative,
                        expr: Box::new(Expr::Literal {
                            value: LiteralValue::Number(Span::new(17, 18))
                        })
                    }),
                })
            }),
            p.parse_term()
        );
    }

    #[test]
    fn test_parse_comparison() {
        let mut l = crate::Lexer::new("-1 >= (2)");
        let tokens = l.lex().unwrap();
        let mut p = Parser::new(tokens);

        assert_eq!(
            Some(Expr::Binary {
                left: Box::new(Expr::Unary {
                    op: UnaryOperator::Negative,
                    expr: Box::new(Expr::Literal {
                        value: LiteralValue::Number(Span::new(1, 2))
                    })
                }),
                op: BinaryOperator::GreaterEqual,
                right: Box::new(Expr::Group {
                    inner: Box::new(Expr::Literal {
                        value: LiteralValue::Number(Span::new(7, 8))
                    })
                })
            }),
            p.parse_comparison()
        );
    }

    #[test]
    fn test_parse_equality() {
        let mut l = crate::Lexer::new("true == !false");
        let tokens = l.lex().unwrap();
        let mut p = Parser::new(tokens);

        assert_eq!(
            Some(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: LiteralValue::True
                }),
                op: BinaryOperator::Equal,
                right: Box::new(Expr::Unary {
                    op: UnaryOperator::Not,
                    expr: Box::new(Expr::Literal {
                        value: LiteralValue::False
                    })
                })
            }),
            p.parse_equality()
        );
    }
}
