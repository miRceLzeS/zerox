pub mod expr;
pub mod stmt;

use crate::Span;
pub use crate::lex::{Token, TokenKind, Tokens};
pub use crate::{Error, Result};
pub use expr::{BinaryOperator, Expr, LiteralValue, UnaryOperator};
pub use stmt::Stmt;

#[derive(Debug)]
pub struct Parser {
    current: usize,
    tokens: Tokens,
}

impl Parser {
    pub fn new(tokens: Tokens) -> Self {
        Self { current: 0, tokens }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut prog = vec![];

        let n = self.tokens.len();
        loop {
            if self.current + 1 >= n {
                break;
            }

            match self.parse_decl() {
                Some(valid_stmt) => match valid_stmt {
                    Stmt::Unknown(msg) => return Err(Error::ParseError(msg)),
                    _ => prog.push(valid_stmt),
                },
                None => return Err(Error::ParseError(format!("Unknown syntax."))),
            }
        }

        Ok(prog)
    }

    pub fn state(&self) -> Option<(usize, Token)> {
        let tok = self.peek()?;
        Some((self.current, tok))
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Expr> {
        let exp = self.parse_logic_or()?;

        let next = self.peek()?;
        if next.kind == TokenKind::EQUAL {
            self.advance();

            let assign_expr = self.parse_assignment()?;
            return match exp {
                Expr::Variable { ident } => Some(Expr::Assign {
                    ident,
                    expr: Box::new(assign_expr),
                }),
                _ => None,
            };
        }

        Some(exp)
    }

    fn parse_logic_or(&mut self) -> Option<Expr> {
        let mut expr = self.parse_logic_and()?;

        loop {
            let tok = self.peek()?;
            if tok.kind == TokenKind::OR {
                self.advance();

                let frag = self.parse_logic_and()?;

                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOperator::Or,
                    right: Box::new(frag),
                };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_logic_and(&mut self) -> Option<Expr> {
        let mut expr = self.parse_equality()?;

        loop {
            let tok = self.peek()?;
            if tok.kind == TokenKind::AND {
                self.advance()?;

                let frag = self.parse_equality()?;

                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOperator::And,
                    right: Box::new(frag),
                };
            } else {
                break;
            }
        }

        Some(expr)
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
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> Option<Expr> {
        let expr = self.parse_primary()?;

        if self.peek()?.kind == TokenKind::LPAREN {
            self.advance();

            let mut args: Vec<Expr> = vec![];
            if let Some(first_arg) = self.parse_expression() {
                args.push(first_arg);
            }

            loop {
                let tok = self.peek()?;
                match tok.kind {
                    TokenKind::RPAREN => {
                        self.advance();
                        return Some(Expr::Call {
                            callee: Box::new(expr),
                            args,
                        });
                    }

                    TokenKind::COMMA => {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }

                    _ => return None,
                };
            }
        }

        Some(expr)
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

            TokenKind::IDENTIFIER => {
                self.advance();
                Some(Expr::Variable { ident: tok.span })
            }

            _ => None,
        }
    }

    fn parse_decl(&mut self) -> Option<Stmt> {
        let tok = self.peek()?;
        match tok.kind {
            TokenKind::VAR => {
                self.advance();
                return self.parse_var_decl();
            }

            TokenKind::FUN => {
                self.advance();
                return self.parse_fun_decl();
            }

            _ => {}
        }

        self.parse_stmt()
    }

    fn parse_var_decl(&mut self) -> Option<Stmt> {
        let tok = self.peek()?;
        if tok.kind != TokenKind::IDENTIFIER {
            return Some(Stmt::Unknown(format!("expect identifier after 'var'")));
        }

        self.advance();

        let mut stat = Stmt::VarDeclStmt {
            ident: tok.span,
            init_expr: None,
        };

        match self.peek()?.kind {
            TokenKind::EQUAL => {
                self.advance();
                let init_expr = self.parse_expression()?;
                stat = Stmt::VarDeclStmt {
                    ident: tok.span,
                    init_expr: Some(init_expr),
                };
            }
            _ => {}
        }

        match self.advance()?.kind {
            TokenKind::SEMICOLON => Some(stat),
            _ => Some(Stmt::Unknown(format!(
                "expect ';' for varieble declaration"
            ))),
        }
    }

    fn parse_fun_decl(&mut self) -> Option<Stmt> {
        let tok = self.peek()?;
        if tok.kind != TokenKind::IDENTIFIER {
            return Some(Stmt::Unknown(format!("expect identifier after 'fun'")));
        }

        self.advance();

        if self.advance()?.kind != TokenKind::LPAREN {
            return Some(Stmt::Unknown(format!("expect '(' after identifier")));
        }

        let mut params: Vec<Span> = vec![];
        if self.peek()?.kind == TokenKind::IDENTIFIER {
            let first_param = self.advance()?;
            params.push(first_param.span);
        }

        loop {
            match self.peek()?.kind {
                TokenKind::RPAREN => {
                    self.advance();
                    break;
                }

                TokenKind::COMMA => {
                    self.advance();
                    let param = self.advance()?;
                    if param.kind != TokenKind::IDENTIFIER {
                        return Some(Stmt::Unknown(format!("expect identer after ','")));
                    }
                    params.push(param.span);
                }

                _ => {
                    return Some(Stmt::Unknown(format!("unexpect character")));
                }
            }
        }

        if self.peek()?.kind != TokenKind::LBRACE {
            return Some(Stmt::Unknown(format!(
                "function declaration requires \"{{ ... }}\" as body"
            )));
        }

        let block = self.parse_stmt()?;

        Some(Stmt::FunDeclStmt {
            ident: tok.span,
            params,
            body: Box::new(block),
        })
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        let tok = self.peek()?;
        match tok.kind {
            TokenKind::PRINT => {
                self.advance();
                self.parse_print_stmt()
            }

            TokenKind::LBRACE => {
                self.advance();
                self.parse_block()
            }

            TokenKind::IF => {
                self.advance();
                self.parse_if_stmt()
            }

            TokenKind::WHILE => {
                self.advance();
                self.parse_while_stmt()
            }

            TokenKind::FOR => {
                self.advance();
                self.parse_for_stmt()
            }

            _ => self.parse_expression_stmt(),
        }
    }

    fn parse_print_stmt(&mut self) -> Option<Stmt> {
        let expr = self.parse_expression()?;
        let tok = self.advance()?;
        match tok.kind {
            TokenKind::SEMICOLON => Some(Stmt::PrintStmt(expr)),
            _ => Some(Stmt::Unknown(format!("expect ';' after expression"))),
        }
    }

    fn parse_block(&mut self) -> Option<Stmt> {
        let mut stmts: Vec<Stmt> = vec![];
        while let Some(decl) = self.parse_decl() {
            stmts.push(decl);
        }

        return match self.advance()?.kind {
            TokenKind::RBRACE => Some(Stmt::BlockStmt { stmts }),
            _ => Some(Stmt::Unknown(format!("expect '}}' after block"))),
        };
    }

    fn parse_expression_stmt(&mut self) -> Option<Stmt> {
        let expr = self.parse_expression()?;
        match self.advance()?.kind {
            TokenKind::SEMICOLON => Some(Stmt::ExprStmt(expr)),
            _ => Some(Stmt::Unknown(format!("expect ';' after expression"))),
        }
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        if self.advance()?.kind != TokenKind::LPAREN {
            return Some(Stmt::Unknown(format!("expect '(' after 'if'")));
        }

        let cond = self.parse_expression()?;

        if self.advance()?.kind != TokenKind::RPAREN {
            return Some(Stmt::Unknown(format!(
                "expect ')' for closing if condition expression"
            )));
        }

        let then_stmt = self.parse_stmt()?;
        let mut if_stmt = Stmt::IfStmt {
            cond,
            then_branch: Box::new(then_stmt),
            else_branch: None,
        };

        let tok = self.peek()?;
        if tok.kind == TokenKind::ELSE {
            self.advance()?;
            let else_stmt = self.parse_stmt()?;
            if let Stmt::IfStmt {
                cond: _,
                then_branch: _,
                else_branch,
            } = &mut if_stmt
            {
                *else_branch = Some(Box::new(else_stmt));
            }
        }

        Some(if_stmt)
    }

    fn parse_while_stmt(&mut self) -> Option<Stmt> {
        if self.advance()?.kind != TokenKind::LPAREN {
            return Some(Stmt::Unknown(format!("expect '(' after 'while'")));
        }

        let cond = self.parse_expression()?;

        if self.advance()?.kind != TokenKind::RPAREN {
            return Some(Stmt::Unknown(format!(
                "expect ')' for closing while condition expression"
            )));
        }

        let body = self.parse_stmt()?;

        Some(Stmt::WhileStmt {
            cond,
            body: Box::new(body),
        })
    }

    // desugared using while statement
    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        if self.advance()?.kind != TokenKind::LPAREN {
            return Some(Stmt::Unknown(format!("expect '(' after 'for'")));
        }

        let mut init: Option<Stmt> = None;
        let tok = self.peek()?;
        match tok.kind {
            TokenKind::SEMICOLON => {
                self.advance();
            }

            TokenKind::VAR => {
                self.advance();
                init = self.parse_var_decl();
            }

            _ => {
                init = self.parse_expression_stmt();
            }
        }

        let mut cond: Option<Expr> = None;
        let tok = self.peek()?;
        match tok.kind {
            TokenKind::SEMICOLON => {
                self.advance();
            }

            _ => {
                cond = self.parse_expression();

                if self.advance()?.kind != TokenKind::SEMICOLON {
                    return Some(Stmt::Unknown(format!(
                        "expect ';' to end condtition expression in 'for' clauses"
                    )));
                }
            }
        }

        let mut incr: Option<Expr> = None;
        let tok = self.peek()?;
        match tok.kind {
            TokenKind::RPAREN => {
                self.advance();
            }

            _ => {
                incr = self.parse_expression();

                if self.advance()?.kind != TokenKind::RPAREN {
                    return Some(Stmt::Unknown(format!("expect ')' to close 'for' clauses")));
                }
            }
        }

        let mut body = self.parse_stmt()?;
        if let Some(incr_stmt) = incr {
            body = Stmt::BlockStmt {
                stmts: vec![body, Stmt::ExprStmt(incr_stmt)],
            };
        }

        if let Some(cond_expr) = cond {
            body = Stmt::WhileStmt {
                cond: cond_expr,
                body: Box::new(body),
            };
        } else {
            body = Stmt::WhileStmt {
                cond: Expr::Literal {
                    value: LiteralValue::True,
                },
                body: Box::new(body),
            };
        }

        if let Some(init_stmt) = init {
            body = Stmt::BlockStmt {
                stmts: vec![init_stmt, body],
            };
        }

        Some(body)
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.current)?;
        self.current += 1;
        Some(*tok)
    }

    fn peek(&self) -> Option<Token> {
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
