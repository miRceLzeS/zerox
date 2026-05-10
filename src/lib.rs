pub mod error;
pub mod lex;
pub mod parse;
pub mod span;

pub use error::Error;
pub use lex::Lexer;
pub use parse::Parser;
pub use span::Span;

pub type Result<T> = std::result::Result<T, Error>;
