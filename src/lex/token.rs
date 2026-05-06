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
    GREATER,
    GEQUAL,
    LESS,
    LEQUAL,

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

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: super::Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: (usize, usize)) -> Token {
        Token {
            kind,
            span: super::Span {
                start: span.0,
                end: span.1,
            },
        }
    }
}
