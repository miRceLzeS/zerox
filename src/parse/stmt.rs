use super::Expr;
use crate::Span;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    VarDeclStmt {
        ident: Span,
        init_expr: Option<Expr>,
    },
    BlockStmt {
        stmts: Vec<Stmt>,
    },
    ExprStmt(Expr),
    PrintStmt(Expr),

    Unknown(String),
}
