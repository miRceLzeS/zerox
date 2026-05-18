pub mod token;

use std::{collections::HashMap, sync::LazyLock};

pub use crate::Span;
pub use crate::{Error, Result};
pub use token::{Token, TokenKind};

pub type Tokens = Vec<Token>;

static KEYWORDS: LazyLock<HashMap<&'static str, TokenKind>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("true", TokenKind::TRUE);
    m.insert("false", TokenKind::FALSE);
    m.insert("nil", TokenKind::NIL);
    m.insert("var", TokenKind::VAR);
    m.insert("or", TokenKind::OR);
    m.insert("and", TokenKind::AND);
    m.insert("if", TokenKind::IF);
    m.insert("else", TokenKind::ELSE);
    m.insert("for", TokenKind::FOR);
    m.insert("while", TokenKind::WHILE);
    m.insert("fun", TokenKind::FUN);
    m.insert("return", TokenKind::RETURN);
    m.insert("class", TokenKind::CLASS);
    m.insert("super", TokenKind::SUPER);
    m.insert("this", TokenKind::THIS);
    m.insert("print", TokenKind::PRINT);

    m
});

pub struct Lexer<'s> {
    current: usize,
    line: usize,
    source: &'s str,
}

impl<'s> Lexer<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            current: 0,
            line: 1,
            source,
        }
    }

    pub fn lex(&mut self) -> Result<Tokens> {
        self.current = 0;
        self.line = 1;

        let mut start: usize;
        let mut tokens = vec![];

        while let Some(ss) = self.advance() {
            start = self.current - 1;

            match ss {
                " " | "\r" | "\t" => {}

                "\n" => {
                    self.line += 1;
                }

                "(" => tokens.push(Token::new(
                    TokenKind::LPAREN,
                    (start, self.current),
                    self.line,
                )),

                ")" => tokens.push(Token::new(
                    TokenKind::RPAREN,
                    (start, self.current),
                    self.line,
                )),

                "{" => tokens.push(Token::new(
                    TokenKind::LBRACE,
                    (start, self.current),
                    self.line,
                )),

                "}" => tokens.push(Token::new(
                    TokenKind::RBRACE,
                    (start, self.current),
                    self.line,
                )),

                "," => tokens.push(Token::new(
                    TokenKind::COMMA,
                    (start, self.current),
                    self.line,
                )),

                "." => tokens.push(Token::new(TokenKind::DOT, (start, self.current), self.line)),

                ";" => tokens.push(Token::new(
                    TokenKind::SEMICOLON,
                    (start, self.current),
                    self.line,
                )),

                "+" => tokens.push(Token::new(
                    TokenKind::PLUS,
                    (start, self.current),
                    self.line,
                )),

                "-" => tokens.push(Token::new(
                    TokenKind::MINUS,
                    (start, self.current),
                    self.line,
                )),

                "*" => tokens.push(Token::new(
                    TokenKind::STAR,
                    (start, self.current),
                    self.line,
                )),

                "/" => {
                    tokens.push(Token::new(
                        self.either(TokenKind::COMMENT, TokenKind::SLASH),
                        (start, self.current),
                        self.line,
                    ));
                }

                "=" => {
                    tokens.push(Token::new(
                        self.either(TokenKind::EEQUAL, TokenKind::EQUAL),
                        (start, self.current),
                        self.line,
                    ));
                }

                "!" => {
                    tokens.push(Token::new(
                        self.either(TokenKind::BEQUAL, TokenKind::BANG),
                        (start, self.current),
                        self.line,
                    ));
                }

                ">" => {
                    tokens.push(Token::new(
                        self.either(TokenKind::RANGLEEQUAL, TokenKind::RANGLE),
                        (start, self.current),
                        self.line,
                    ));
                }

                "<" => {
                    tokens.push(Token::new(
                        self.either(TokenKind::LANGLEEQUAL, TokenKind::LANGLE),
                        (start, self.current),
                        self.line,
                    ));
                }

                // literal
                _ => {
                    if ss == "\"" {
                        match self.match_string(&mut start) {
                            Some(tok) => tokens.push(tok),
                            None => {
                                return Err(Error::LexError(format!("unclosing quotes")));
                            }
                        }
                    } else if self.is_digit(ss) {
                        match self.match_number(&mut start) {
                            Some(tok) => tokens.push(tok),
                            None => {
                                return Err(Error::LexError(format!("empty number fraction")));
                            }
                        }
                    } else if self.is_alpha(ss) {
                        tokens.push(self.match_identifier(&mut start));
                    } else {
                        return Err(Error::LexError(format!(
                            "{}:{}: unknown character",
                            self.line, start
                        )));
                    }
                }
            }
        }

        // self.current == self.source.chars().count()
        tokens.push(Token::new(
            TokenKind::EOF,
            (self.current, self.current),
            self.line,
        ));

        Ok(tokens)
    }

    fn peek(&self, num: usize) -> Option<&'s str> {
        let mut it = self.source.char_indices();

        let (start, _) = it.nth(self.current + num)?;
        let end = it.next().map(|(i, _)| i).unwrap_or(self.source.len());

        Some(&self.source[start..end])
    }

    fn advance(&mut self) -> Option<&'s str> {
        let res = self.peek(0)?;
        self.current += 1;

        Some(res)
    }

    fn either(&mut self, long: TokenKind, short: TokenKind) -> TokenKind {
        match short {
            TokenKind::EQUAL | TokenKind::BANG | TokenKind::RANGLE | TokenKind::LANGLE => {
                if let Some(ss) = self.peek(0)
                    && ss == "="
                {
                    self.advance();
                    long
                } else {
                    short
                }
            }

            TokenKind::SLASH => {
                if let Some(ss) = self.peek(0)
                    && ss == "/"
                {
                    // consume the second "/"
                    self.advance();

                    while let Some(comment) = self.peek(0)
                        && comment != "\n"
                    {
                        self.advance();
                    }

                    long
                } else {
                    short
                }
            }

            _ => short,
        }
    }

    fn match_string(&mut self, start: &usize) -> Option<Token> {
        // when to break the while
        // 1. self.peek(0) -> None => at end
        // 2. ss == " => not at end
        while let Some(ss) = self.peek(0)
            && ss != "\""
        {
            if ss == "\n" {
                self.line += 1;
            }
            self.advance();
        }

        return match self.peek(0) {
            Some(_) => {
                self.advance();
                Some(Token::new(
                    TokenKind::STRING,
                    (*start, self.current),
                    self.line,
                ))
            }
            None => None,
        };
    }

    fn is_digit(&self, s: &str) -> bool {
        s.chars().all(|ch| ch.is_ascii_digit())
    }

    fn match_number(&mut self, start: &usize) -> Option<Token> {
        let consume_digits = |l: &mut Self| {
            while let Some(ss) = l.peek(0)
                && l.is_digit(ss)
            {
                l.advance();
            }
        };

        consume_digits(self);

        if let Some(ss) = self.peek(0)
            && ss == "."
        {
            if let Some(next) = self.peek(1)
                && !self.is_digit(next)
            {
                return None;
            }

            self.advance(); // consume the "."
            consume_digits(self);
        }

        Some(Token::new(
            TokenKind::NUMBER,
            (*start, self.current),
            self.line,
        ))
    }

    fn is_alpha(&self, s: &str) -> bool {
        s.chars().all(|ch| ch.is_ascii_alphabetic() || ch == '_')
    }

    fn match_identifier(&mut self, start: &usize) -> Token {
        while let Some(ss) = self.peek(0)
            && (self.is_alpha(ss) || self.is_digit(ss))
        {
            self.advance();
        }

        let ss = &self.source[*start..self.current];
        if let Some(&kind) = KEYWORDS.get(ss) {
            return Token::new(kind.clone(), (*start, self.current), self.line);
        }

        Token::new(TokenKind::IDENTIFIER, (*start, self.current), self.line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance() {
        let mut l = Lexer::new("你好，world!");

        assert_eq!(Some("你"), l.advance());
        assert_eq!(Some("好"), l.advance());
        assert_eq!(Some("，"), l.advance());
        assert_eq!(Some("w"), l.advance());
        assert_eq!(Some("o"), l.advance());
        assert_eq!(Some("r"), l.advance());
        assert_eq!(Some("l"), l.advance());
        assert_eq!(Some("d"), l.advance());
        assert_eq!(Some("!"), l.advance());
    }

    #[test]
    fn test_peek() {
        let mut l = Lexer::new("你好，world!");

        assert_eq!(Some("!"), l.peek(8));

        assert_eq!(Some("你"), l.advance());
        assert_eq!(Some("好"), l.advance());

        assert_eq!(Some("，"), l.peek(0));
        assert_eq!(Some("!"), l.peek(6));

        assert_eq!(None, l.peek(7));
    }

    #[test]
    fn test_match_string() {
        let mut l = Lexer::new("\"abc\ndef\"");
        let mut start = 0;

        assert_eq!(Some("\""), l.advance());

        let tok = l.match_string(&mut start).unwrap();
        assert_eq!((0, 9), (tok.span.start, tok.span.end));
    }

    #[test]
    fn test_match_number() {
        let mut l = Lexer::new("123.456");
        let mut start = 0;

        assert_eq!(Some("1"), l.advance());

        let tok = l.match_number(&mut start).unwrap();
        assert_eq!((0, 7), (tok.span.start, tok.span.end));
    }

    #[test]
    fn test_match_keywords() {
        // kevwords
        let mut l = Lexer::new(
            "
            true
            false
            nil
            var
            or
            and
            if
            else
            for
            while
            fun
            return
            class
            super
            this
            print
            ",
        );

        let tokens = l.lex().unwrap();
        assert_eq!(16 + 1, tokens.len());

        let mut it = tokens.into_iter();
        assert_eq!(TokenKind::TRUE, it.next().unwrap().kind);
        assert_eq!(TokenKind::FALSE, it.next().unwrap().kind);
        assert_eq!(TokenKind::NIL, it.next().unwrap().kind);
        assert_eq!(TokenKind::VAR, it.next().unwrap().kind);
        assert_eq!(TokenKind::OR, it.next().unwrap().kind);
        assert_eq!(TokenKind::AND, it.next().unwrap().kind);
        assert_eq!(TokenKind::IF, it.next().unwrap().kind);
        assert_eq!(TokenKind::ELSE, it.next().unwrap().kind);
        assert_eq!(TokenKind::FOR, it.next().unwrap().kind);
        assert_eq!(TokenKind::WHILE, it.next().unwrap().kind);
        assert_eq!(TokenKind::FUN, it.next().unwrap().kind);
        assert_eq!(TokenKind::RETURN, it.next().unwrap().kind);
        assert_eq!(TokenKind::CLASS, it.next().unwrap().kind);
        assert_eq!(TokenKind::SUPER, it.next().unwrap().kind);
        assert_eq!(TokenKind::THIS, it.next().unwrap().kind);
        assert_eq!(TokenKind::PRINT, it.next().unwrap().kind);
        assert_eq!(TokenKind::EOF, it.next().unwrap().kind);
    }

    #[test]
    fn test_match_identifier() {
        let mut l = Lexer::new("or orchid ororchid");

        let mut it = l.lex().unwrap().into_iter();
        assert_eq!(TokenKind::OR, it.next().unwrap().kind);
        assert_eq!(TokenKind::IDENTIFIER, it.next().unwrap().kind);
        assert_eq!(TokenKind::IDENTIFIER, it.next().unwrap().kind);
        assert_eq!(TokenKind::EOF, it.next().unwrap().kind);
    }
}
