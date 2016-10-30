pub mod errors;
mod parser;

use self::errors::*;
use std::io;
use std::io::prelude::*;
use std::fs;
use std::path::Path;

pub type ClientgenResult<T> = Result<T, ClientgenError>;

pub fn generate_clients<W: Write, P: AsRef<Path>>(writer: &mut W, templates_dir: P) -> ClientgenResult<()> {
    try!(writer.write_all(b"a"));
    Ok(())
}
