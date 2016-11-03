use super::parser;
use std;
use std::borrow::Cow;
use std::io;
use std::io::prelude::*;
use inflector::Inflector;

pub fn twitter_client<W: Write>(writer: &mut W, input: &[parser::ApiTemplate]) -> io::Result<()> {
    try!(writer.write_all(b"\
#[derive(Clone, Debug)]
pub struct TwitterClient<T: Authenticator> { auth: T }

impl<T: Authenticator> TwitterClient<T> {
    pub fn new(authenticator: T) -> TwitterClient<T> {
        TwitterClient { auth: authenticator }
    }
"));

    for api in input {
        try!(writeln!(
            writer,
            "
    pub fn {}(&self) -> {}Client<T> {{
        {1}Client {{ auth: &self.auth }}
    }}",
            api.namespace.to_snake_case(),
            api.namespace
        ));
    }

    writer.write_all(&b"}\n"[..])
}

pub fn request_builders<W: Write>(writer: &mut W, input: &parser::ApiTemplate) -> io::Result<()> {
    let endpoints: Vec<_> = input.endpoints.iter()
        .filter_map(|x| create_endpoint(x, input))
        .collect();

    // TODO
    Ok(())
}

fn document<W: Write>(writer: &mut W, content: &str, indent: u32) -> io::Result<()> {
    let mut indent_str = String::with_capacity(indent as usize);
    for _ in 0..indent { indent_str.push(' '); }

    for line in content.trim().lines() {
        try!(writeln!(writer, "{}/// {}", indent_str, line));
    }

    Ok(())
}

#[derive(Debug)]
struct Endpoint<'a> {
    pub namespace: &'a str,
    pub fn_name: String,
    pub description: &'a Option<String>,
    pub method: &'a parser::EndpointType,
    pub return_type: Cow<'a, str>,
    pub reserved_parameter: Option<&'a str>,
    pub required_parameters: Vec<(&'a str, ParamType<'a>)>,
    pub optional_parameters: Vec<(&'a str, ParamType<'a>)>,
}

#[derive(Debug)]
enum ParamType<'a> {
    Normal(Cow<'a, str>),
    String,
    List(Cow<'a, str>),
    StringList,
}

#[derive(Debug)]
struct FnParametersGenerator {
    type_parameters: Vec<u8>,
    type_parameter_count: u32,
    parameters: Vec<u8>,
}

impl FnParametersGenerator {
    pub fn new() -> FnParametersGenerator {
        FnParametersGenerator {
            type_parameters: Vec::new(),
            type_parameter_count: 0,
            parameters: Vec::new(),
        }
    }

    fn add_type_parameter<T: std::fmt::Display>(&mut self, constaint: T) -> u32 {
        if self.type_parameter_count > 0 {
            self.type_parameters.extend_from_slice(b", ");
        }

        self.type_parameter_count += 1;
        write!(self.type_parameters, "T{}: {}", self.type_parameter_count, constaint).unwrap();
        self.type_parameter_count
    }

    fn write_type_param(&mut self, index: u32) {
        write!(self.parameters, "T{}", index).unwrap();
    }

    pub fn add_parameter(&mut self, name: &str, ty: &ParamType) {
        write!(self.parameters, ", {}: ", name).unwrap();

        match *ty {
            ParamType::Normal(ref x) => self.parameters.extend_from_slice(x.as_ref().as_bytes()),
            ParamType::String => {
                let t = self.add_type_parameter("Into<Cow<'a, str>>");
                self.write_type_param(t);
            }
            ParamType::List(ref x) => {
                let t = self.add_type_parameter(format!("IntoIterator<Item = {}>", x));
                self.write_type_param(t);
            }
            ParamType::StringList => {
                let as_ref_index = self.add_type_parameter("AsRef<str>");
                let into_iter_index = self.add_type_parameter(format!("IntoIterator<Item = T{}>", as_ref_index));
                self.write_type_param(into_iter_index);
            }
        }
    }

    pub fn write_type_parameters<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.type_parameter_count > 0 {
            try!(writer.write_all(b"<"));
            try!(writer.write_all(&self.type_parameters));
            try!(writer.write_all(b">"));
        }

        Ok(())
    }

    pub fn write_parameters<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.parameters)
    }
}

/// Returns None if the return type is not supported.
fn create_return_type<'a>(endpoint: &'a parser::Endpoint, api_template: &parser::ApiTemplate) -> Option<Cow<'a, str>> {
    macro_rules! sb {
        ($x:expr) => (Some(Cow::Borrowed($x)))
    }

    fn core<'a>(ty: &'a str) -> Option<Cow<'a, str>> {
        match ty {
            "string" => sb!("String"),
            "int" => sb!("i32"),
            "long" => sb!("i64"),
            "double" => sb!("f64"),
            "Status" => sb!("Tweet"),
            "Embed" => sb!("OEmbed"),
            "SearchResult" => sb!("SearchResponse"),
            "Configurations" => sb!("Configuration"),
            "TrendLocation" => sb!("TrendPlace"),
            "SearchQuery" => sb!("SavedSearch"),
            "Setting" => sb!("AccountSettings"),
            "Category" => sb!("UserCategory"),
            "Relationship" => sb!("FriendshipResponse"),
            "GeoResult" => sb!("GeoResponse"),
            x => Some(Cow::Borrowed(x)),
        }
    }

    match endpoint.return_type.as_ref() {
        "void" => sb!("()"),
        "StringResponse" => Some(Cow::Owned(format!("{}Response", endpoint.name))),
        "CategoryResponse" => sb!("SuggestedUsers"),
        "Cursored<long>" => sb!("CursorIds"),
        x if x.starts_with("Cursored<") => Some(Cow::Owned(format!("Cursor{}s", &x[9..x.len() - 1]))),
        x if x.starts_with("Listed<") => core(&x[7..x.len() - 1]).map(|x| Cow::Owned(format!("Vec<{}>", x))),
        x if x.starts_with("Dictionary<") => {
            warn!("Unsupported return type `{}`: {}.{}", x, api_template.namespace, endpoint.name);
            None
        }
        x => core(x.trim_right_matches("Response")),
    }
}

/// Returns None if the return type is not supported.
fn create_param_type<'a>(tn: &'a parser::TypeNamePair, endpoint: &parser::Endpoint, api_template: &parser::ApiTemplate) -> Option<ParamType<'a>> {
    match tn.param_type.as_ref() {
        "string" => Some(ParamType::String),
        "int" => Some(ParamType::Normal(Cow::Borrowed("i32"))),
        "long" => Some(ParamType::Normal(Cow::Borrowed("i64"))),
        "double" => Some(ParamType::Normal(Cow::Borrowed("f64"))),
        "Stream" => {
            warn!("Unsupported parameter type `Stream`: {}.{}", api_template.namespace, endpoint.name);
            None
        },
        "IEnumerable<string>" => Some(ParamType::StringList),
        x if x.starts_with("IEnumerable<") => Some(ParamType::List(Cow::Borrowed(&x[12..x.len() - 1]))),
        x => Some(ParamType::Normal(Cow::Borrowed(x))),
    }
}

fn create_endpoint<'a>(endpoint: &'a parser::Endpoint, api_template: &'a parser::ApiTemplate) -> Option<Endpoint<'a>> {
    if endpoint.endpoint_type == parser::EndpointType::Impl {
        warn!("Requires custom execute function: {}.{}", api_template.namespace, endpoint.name);
        return None;
    }

    if endpoint.json_path.is_some() {
        info!("Has JSON path: {}.{}", api_template.namespace, endpoint.name);
    }

    let return_type = match create_return_type(endpoint, api_template) {
        Some(x) => x,
        None => return None,
    };

    let mut required_parameters = Vec::new();

    for p in endpoint.params.iter() {
        if p.kind == parser::ParamKind::Required {
            for tn in p.type_name_pairs.iter() {
                if let Some(x) = create_param_type(tn, endpoint, api_template) {
                    required_parameters.push((tn.name.as_ref(), x));
                } else {
                    return None;
                }
            }
        }
    }

    let mut optional_parameters = Vec::new();

    {
        let mut set = std::collections::HashSet::new();

        let tns = endpoint.params.iter()
            .filter(|x| x.kind != parser::ParamKind::Required)
            .flat_map(|x| x.type_name_pairs.iter());

        for tn in tns {
            if set.contains(tn) { continue; }

            if let Some(x) = create_param_type(tn, endpoint, api_template) {
                optional_parameters.push((tn.name.as_ref(), x));
            } else {
                return None;
            }

            set.insert(tn);
        }
    };

    let reserved_parameter = match endpoint.endpoint_type {
        parser::EndpointType::Get(ref x) | parser::EndpointType::Post(ref x) => {
            x.find('{').and_then(|lb| {
                let s = &x[lb + 1..];
                s.find('}').map(|rb| &s[..rb])
            })
        }
        parser::EndpointType::Impl => None,
    };

    Some(Endpoint {
        namespace: &api_template.namespace,
        fn_name: endpoint.name.to_snake_case(),
        description: &endpoint.description,
        method: &endpoint.endpoint_type,
        return_type: return_type,
        reserved_parameter: reserved_parameter,
        required_parameters: required_parameters,
        optional_parameters: optional_parameters,
    })
}
