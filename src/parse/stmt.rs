use super::Expr;
use crate::Span;

#[derive(Debug, PartialEq, Clone)]
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

    IfStmt {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    WhileStmt {
        cond: Expr,
        body: Box<Stmt>,
    },

    Unknown(String),
}
