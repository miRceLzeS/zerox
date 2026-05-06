#[derive(Debug)]
pub enum ErrorKind {
    IOError(std::io::Error),
    CLIErrorr(String),
    LexError(String),
    ParseError(String),
    CogegenError(String),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::IOError(err) => write!(f, "io error\n{}", err),
            ErrorKind::CLIErrorr(msg) => write!(f, "cli error\n{}", msg),
            ErrorKind::LexError(msg) => write!(f, "lex error\n{}", msg),
            ErrorKind::ParseError(msg) => write!(f, "parse error\n{}", msg),
            ErrorKind::CogegenError(msg) => write!(f, "code gen error\n{}", msg),
        }
    }
}

#[derive(Debug)]
pub struct Error(ErrorKind);

impl Error {
    pub fn new(err_kind: ErrorKind) -> Self {
        Self(err_kind)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "zerox: {}\n", self.0)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self(ErrorKind::IOError(value.into()))
    }
}
