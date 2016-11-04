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

    writer.write_all(b"}\n")
}

pub fn request_builders<W: Write>(writer: &mut W, input: &parser::ApiTemplate) -> io::Result<()> {
    let endpoints: Vec<_> = input.endpoints.iter()
        .filter_map(|x| create_endpoint(x, input))
        .collect();

    try!(client_struct(writer, input));
    try!(client_impl(writer, input, &endpoints));

    for x in endpoints {
        try!(request_builder_struct(writer, &x, input));
        try!(request_builder_impl(writer, &x, input));
    }

    Ok(())
}

fn document<W: Write>(writer: &mut W, content: &str, indent: usize) -> io::Result<()> {
    let mut indent_str = String::with_capacity(indent);
    for _ in 0..indent { indent_str.push(' '); }

    for line in content.trim().lines() {
        try!(writeln!(writer, "{}/// {}", indent_str, line.trim()));
    }

    Ok(())
}

#[derive(Debug)]
struct Endpoint<'a> {
    pub name: &'a str,
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
    fn core<'a>(ty: &'a str) -> Cow<'a, str> {
        match ty {
            "int" => Cow::Borrowed("i32"),
            "long" => Cow::Borrowed("i64"),
            "double" => Cow::Borrowed("f64"),
            x => Cow::Borrowed(x),
        }
    }

    match tn.param_type.as_ref() {
        "string" => Some(ParamType::String),
        "Stream" => {
            warn!("Unsupported parameter type `Stream`: {}.{}", api_template.namespace, endpoint.name);
            None
        },
        "IEnumerable<string>" => Some(ParamType::StringList),
        x if x.starts_with("IEnumerable<") => Some(ParamType::List(core(&x[12..x.len() - 1]))),
        x => Some(ParamType::Normal(core(x))),
    }
}

fn create_endpoint<'a>(endpoint: &'a parser::Endpoint, api_template: &'a parser::ApiTemplate) -> Option<Endpoint<'a>> {
    if endpoint.endpoint_type == parser::EndpointType::Impl {
        warn!("Requires custom execute function: {}.{}", api_template.namespace, endpoint.name);
        return None;
    }

    for &(ref attr_name, _) in endpoint.attributes.iter() {
        if attr_name == "Obsolete" {
            info!("Ignored obsolete member: {}.{}", api_template.namespace, endpoint.name);
            return None;
        }
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
        name: &endpoint.name,
        description: &endpoint.description,
        method: &endpoint.endpoint_type,
        return_type: return_type,
        reserved_parameter: reserved_parameter,
        required_parameters: required_parameters,
        optional_parameters: optional_parameters,
    })
}

fn client_struct<W: Write>(writer: &mut W, api_template: &parser::ApiTemplate) -> io::Result<()> {
    try!(writer.write_all(b"\n"));

    if let Some(ref x) = api_template.description {
        try!(document(writer, &x, 0));
    }

    write!(
        writer,
        "#[derive(Clone, Debug)]
pub struct {}Client<'a, T: Authenticator + 'a> {{ auth: &'a T }}
",
        api_template.namespace
    )
}

fn client_impl<W: Write>(writer: &mut W, api_template: &parser::ApiTemplate, endpoints: &[Endpoint]) -> io::Result<()> {
    try!(write!(
        writer,
        "\nimpl<'a, T: Authenticator> {0}Client<'a, T> {{",
        api_template.namespace
    ));

    for x in endpoints {
        try!(client_impl_fn(writer, x, api_template));
    }

    writer.write_all(b"}\n")
}

fn client_impl_fn<W: Write>(writer: &mut W, endpoint: &Endpoint, api_template: &parser::ApiTemplate) -> io::Result<()> {
    let mut p = FnParametersGenerator::new();
    for &(n, ref ty) in endpoint.required_parameters.iter() {
        p.add_parameter(n, ty);
    }

    try!(writer.write_all(b"\n"));
    if let &Some(ref x) = endpoint.description {
        try!(document(writer, &x, 4));
    }
    try!(write!(
        writer,
        "    pub fn {}",
        endpoint.name.to_snake_case()
    ));
    try!(p.write_type_parameters(writer));
    try!(writer.write_all(b"(&self"));
    try!(p.write_parameters(writer));
    try!(writeln!(
        writer,
        ") -> {0}{1}RequestBuilder<'a, T> {{
        {0}{1}RequestBuilder {{
            _auth: self.auth,",
        api_template.namespace,
        endpoint.name
    ));

    for &(n, ref t) in endpoint.required_parameters.iter() {
        try!(write!(writer, "            {}: ", n));
        try!(match *t {
            ParamType::String => writeln!(writer, "{}.into(),", n),
            ParamType::List(_) => writeln!(writer, "collection_paramter({}),", n),
            ParamType::StringList => writeln!(writer, "str_collection_parameter({}),", n),
            _ => writeln!(writer, "{},", n),
        });
    }

    for &(n, _) in endpoint.optional_parameters.iter() {
        try!(writeln!(writer, "            {0}: None,", n));
    }

    writer.write_all(b"        }
    }
")
}

fn request_builder_struct<W: Write>(writer: &mut W, endpoint: &Endpoint, api_template: &parser::ApiTemplate) -> io::Result<()> {
    fn field_type<'a>(pt: &'a ParamType<'a>) -> Cow<'a, str> {
        match *pt {
            ParamType::Normal(ref x) => Cow::Borrowed(x.as_ref()),
            ParamType::String => Cow::Borrowed("Cow<'a, str>"),
            ParamType::List(_) | ParamType::StringList => Cow::Borrowed("String"),
        }
    }

    try!(write!(
        writer,
        "
pub struct {}{}RequestBuilder<'a, T: Authenticator + 'a> {{
    _auth: &'a T,
",
        api_template.namespace,
        endpoint.name
    ));

    for &(n, ref t) in endpoint.required_parameters.iter() {
        try!(writeln!(writer, "    {}: {},", n, field_type(t)));
    }

    for &(n, ref t) in endpoint.optional_parameters.iter() {
        try!(writeln!(writer, "    {}: Option<{}>,", n, field_type(t)));
    }

    writer.write_all(b"}\n")
}

fn request_builder_impl<W: Write>(writer: &mut W, endpoint: &Endpoint, api_template: &parser::ApiTemplate) -> io::Result<()> {
    try!(write!(
        writer,
        "\nimpl<'a, T: Authenticator> {}{}RequestBuilder<'a, T> {{",
        api_template.namespace,
        endpoint.name
    ));

    for &(n, ref t) in endpoint.optional_parameters.iter() {
        try!(request_builder_setter(writer, n, t));
    }

    if endpoint.method == &parser::EndpointType::Impl {
        unimplemented!();
    } else {
        try!(request_builder_execute(writer, endpoint));
    }

    writer.write_all(b"}\n")
}

fn request_builder_setter<W: Write>(writer: &mut W, name: &str, ty: &ParamType) -> io::Result<()> {
    let mut p = FnParametersGenerator::new();
    p.add_parameter("val", ty);

    try!(write!(writer, "\n    pub fn {}", name));
    try!(p.write_type_parameters(writer));
    try!(writer.write_all(b"(&mut self"));
    try!(p.write_parameters(writer));
    write!(
        writer,
        ") -> &mut Self {{
        self.{} = Some({});
        self
    }}
",
        name,
        match *ty {
            ParamType::String => "val.into()",
            ParamType::List(_) => "collection_paramter(val)",
            ParamType::StringList => "str_collection_parameter(val)",
            _ => "val",
        }
    )
}

fn request_builder_execute<W: Write>(writer: &mut W, endpoint: &Endpoint) -> io::Result<()> {
    let (method, url) = match *endpoint.method {
        parser::EndpointType::Get(ref x) => ("Get", x),
        parser::EndpointType::Post(ref x) => ("Post", x),
        _ => unreachable!(),
    };

    try!(write!(
        writer,
         "
    pub fn execute(&self) -> TwitterResult<{}> {{
        ",
        endpoint.return_type
    ));

    let capacity = endpoint.required_parameters.len() + endpoint.optional_parameters.len()
        - if endpoint.reserved_parameter.is_some() { 1 } else { 0 };

    if capacity > 0 { try!(writeln!(writer, "let mut params = Vec::with_capacity({});", capacity)) }
    else { try!(writer.write_all(b"let params = Vec::<Parameter>::new();\n")) }

    for &(p, _) in endpoint.required_parameters.iter() {
        if endpoint.reserved_parameter == Some(p) { continue; }
        try!(writeln!(
            writer,
            "        params.push(self.{0}.to_parameter(\"{0}\"));",
            p
        ));
    }

    for &(p, _) in endpoint.optional_parameters.iter() {
        try!(writeln!(
            writer,
            "        if let Some(ref x) = self.{0} {{ params.push(x.to_parameter(\"{0}\")) }}",
            p
        ));
    }

    try!(writer.write_all(b"        let url = "));

    if let Some(reserved) = endpoint.reserved_parameter {
        try!(writeln!(
            writer,
            "format!(\"https://api.twitter.com/1.1/{}.json\", {1} = self.{1});",
            url, reserved
        ));
    } else {
        try!(writeln!(
            writer,
            "\"https://api.twitter.com/1.1/{}.json\";",
            url
        ));
    }

    write!(
        writer,
        "        execute_core(self._auth, {}, url, &params)
    }}
",
        method
    )
}
