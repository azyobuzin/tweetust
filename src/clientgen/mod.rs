pub mod errors;
mod generators;
mod parser;

use self::errors::*;
use std::io::prelude::*;
use std::fs;
use std::path::Path;

pub type ClientgenResult<T> = Result<T, ClientgenError>;

pub fn generate_clients<W, T, O>(writer: &mut W, templates_dir: T, override_dir: O) -> ClientgenResult<()>
    where W: Write, T: AsRef<Path>, O: AsRef<Path>
{
    let mut api_templates = try!(load_templates(templates_dir));
    let overrides = try!(load_templates(override_dir));

    for o in overrides {
        if let Some(t) = api_templates.iter_mut().find(|x| x.namespace == o.namespace) {
            for oe in o.endpoints {
                if oe.ignore {
                    // Remove ignored endpoint
                    t.endpoints.retain(|x| x.name != oe.name);
                    continue;
                }

                if let Some(e) = t.endpoints.iter_mut().find(|x| x.name == oe.name) {
                    *e = oe;
                    continue;
                }
                
                t.endpoints.push(oe);
            }

            continue;
        }

        api_templates.push(o);
    }

    try!(generators::twitter_client(writer, &api_templates));

    for x in api_templates {
        try!(generators::request_builders(writer, &x));
    }

    Ok(())
}

fn load_templates<P: AsRef<Path>>(template_dir: P) -> ClientgenResult<Vec<parser::ApiTemplate>> {
    fn pos(base: &str, position: &str) -> (u32, u32) {
        let base_ptr = base.as_ptr() as usize;
        let mut target_ptr = position.as_ptr() as usize;

        if target_ptr < base_ptr || target_ptr > base_ptr + base.len() {
            target_ptr = base_ptr + base.len();
        }

        let mut line = 1;
        let mut column = 1;

        for (i, c) in base.char_indices() {
            if base_ptr + i > target_ptr { break; }

            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        (line, column)
    }

    const EXCLUDE: [&'static str; 3] = ["test.api", "collections.api", "media.api"];

    let mut v = Vec::new();
    let mut buf = String::new();

    for entry in try!(fs::read_dir(template_dir)) {
        let entry = try!(entry);
        
        let file_name = entry.file_name();
        if EXCLUDE.iter().any(|&x| x == &file_name) { continue; }

        if let Ok(file_type) = entry.file_type() {
            if !file_type.is_file() { continue; }
        } else {
            continue;
        }

        let mut file = try!(fs::File::open(entry.path()));
        buf.clear();
        try!(file.read_to_string(&mut buf));

        match parser::parse(&buf) {
            Ok(x) => v.push(x),
            Err(x) => {
                let (position, message) = match x {
                    parser::ParseErrorKind::InternalParserError(x) => {
                        let position = match x {
                            ::nom::Err::Position(_, p) | ::nom::Err::NodePosition(_, p, _) => Some(pos(&buf, p)),
                            _ => None
                        };
                        (position, x.to_string())
                    }
                    parser::ParseErrorKind::Missing(x) => (None, format!("missing field: {}", x)),
                };

                return Err(ClientgenError::ParsingTemplate(ParseError {
                    file_name: file_name.to_string_lossy().into_owned(),
                    position: position,
                    message: message,
                }));
            }
        }
    }

    Ok(v)
}
