pub mod error;
pub mod lex;

pub use error::Error;
pub use lex::Lexer;

pub type Result<T> = std::result::Result<T, Error>;
