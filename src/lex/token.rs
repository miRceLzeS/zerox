#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,

    COMMA,
    DOT,
    SEMICOLON,

    PLUS,
    MINUS,
    SLASH,
    STAR,

    EQUAL,
    EEQUAL,
    BANG,
    BEQUAL,
    RANGLE,
    RANGLEEQUAL,
    LANGLE,
    LANGLEEQUAL,

    // keywords
    TRUE,
    FALSE,
    NIL,
    VAR,
    OR,
    AND,
    IF,
    ELSE,
    FOR,
    WHILE,
    FUN,
    RETURN,
    CLASS,
    SUPER,
    THIS,
    PRINT,

    // literals
    IDENTIFIER,
    STRING,
    NUMBER,

    EOF,

    // misc
    COMMENT,
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub span: super::Span,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, span: (usize, usize), line: usize) -> Token {
        Token {
            kind,
            span: super::Span::new(span.0, span.1),
            line: line,
        }
    }
}
