use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ClientgenError {
    IO(io::Error),
    ParsingTemplate(ParseError),
}

impl Error for ClientgenError {
    fn description(&self) -> &str {
        match *self {
            ClientgenError::IO(ref e) => e.description(),
            ClientgenError::ParsingTemplate(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ClientgenError::IO(ref e) => Some(e),
            ClientgenError::ParsingTemplate(ref e) => Some(e),
        }
    }
}

impl fmt::Display for ClientgenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ClientgenError::IO(ref e) => fmt::Display::fmt(e, f),
            ClientgenError::ParsingTemplate(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl From<io::Error> for ClientgenError {
    fn from(x: io::Error) -> Self {
        ClientgenError::IO(x)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub file_name: String,
    pub position: Option<(u32, u32)>,
    pub message: String,
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "API template parse error"
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some((l, c)) = self.position {
            write!(f, "{} {}:{} {}", self.file_name, l, c, self.message)
        } else {
            write!(f, "{} {}", self.file_name, self.message)
        }
    }
}
