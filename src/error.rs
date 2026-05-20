#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    CLIErrorr(String),
    LexError(String),
    ParseError(String),
    RuntimeError(String),
    CogegenError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(err) => write!(f, "io error\n{}", err),
            Error::CLIErrorr(msg) => write!(f, "cli error\n{}", msg),
            Error::LexError(msg) => write!(f, "lex error\n{}", msg),
            Error::ParseError(msg) => write!(f, "parse error\n{}", msg),
            Error::RuntimeError(msg) => write!(f, "eval error\n{}", msg),
            Error::CogegenError(msg) => write!(f, "code gen error\n{}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IOError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}
