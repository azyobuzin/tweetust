mod parser;

use std::error::Error;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum ClientgenError {
    IO(io::Error),
    ParsingTemplate,
}

impl Error for ClientgenError {
    fn description(&self) -> &str {
        match *self {
            ClientgenError::IO(ref e) => e.description(),
            ClientgenError::ParsingTemplate => "An error was caused in parsing templates"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ClientgenError::IO(ref e) => Some(e),
            _ => None
        }
    }
}

impl fmt::Display for ClientgenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ClientgenError::IO(ref e) => fmt::Display::fmt(e, f),
            _ => f.write_str(self.description())
        }
    }
}

impl From<io::Error> for ClientgenError {
    fn from(x: io::Error) -> Self {
        ClientgenError::IO(x)
    }
}

pub type ClientgenResult<T> = Result<T, ClientgenError>;

pub fn generate_clients<W: Write, P: AsRef<Path>>(writer: &mut W, templates_dir: P) -> ClientgenResult<()> {
    try!(writer.write_all(b"a"));
    Ok(())
}
