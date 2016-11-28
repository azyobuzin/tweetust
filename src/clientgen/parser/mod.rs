mod internal;

#[derive(Debug)]
pub struct ApiTemplate {
    pub namespace: String,
    pub description: Option<String>,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug)]
pub struct Endpoint {
    pub return_type: String,
    pub name: String,
    pub endpoint_type: EndpointType,
    pub json_path: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub ignore: bool,
    pub description: Option<String>,
    pub returns: Option<String>,
    pub params: Vec<Param>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EndpointType {
    Get(String),
    Post(String),
    Impl,
}

#[derive(Debug)]
pub struct Param {
    pub kind: ParamKind,
    pub type_name_pairs: Vec<TypeNamePair>,
    pub when: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Required,
    Either(u8),
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeNamePair {
    pub param_type: String,
    pub name: String,
}

#[derive(Debug)]
pub enum ParseErrorKind<'a> {
    InternalParserError(::nom::Err<&'a str>),
    Missing(&'static str),
}

pub fn parse(input: &str) -> Result<ApiTemplate, ParseErrorKind> {
    let root = match internal::parse_api_template(input) {
        Ok(x) => x,
        Err(x) => return Err(ParseErrorKind::InternalParserError(x))
    };

    let namespace = {
        let ns = root.iter()
            .filter_map(|x| match *x {
                internal::RootElement::Namespace(x) => Some(x.to_owned()),
                _ => None
            })
            .nth(0);
        if let Some(x) = ns { x }
        else { return Err(ParseErrorKind::Missing("#namespace")) }
    };

    let description = root.iter()
        .filter_map(|x| match *x {
            internal::RootElement::Description(x) => Some(x.to_owned()),
            _ => None
        })
        .nth(0);

    let mut endpoints = Vec::with_capacity(root.len() - (if description.is_some() { 2 } else { 1 }));
    for x in root.into_iter() {
        if let internal::RootElement::Endpoint { header: h, elements: e } = x {
            let with = e.iter()
                .filter_map(|x| match *x {
                    internal::EndpointElement::With(ref x) => Some(x),
                    _ => None
                })
                .nth(0);

            endpoints.push(
                Endpoint {
                    return_type: h.return_type.to_owned(),
                    name: h.name.to_owned(),
                    endpoint_type: match h.endpoint_type {
                        internal::EndpointType::Get(x) => EndpointType::Get(x.to_owned()),
                        internal::EndpointType::Post(x) => EndpointType::Post(x.to_owned()),
                        internal::EndpointType::Impl => EndpointType::Impl,
                    },
                    json_path: with.and_then(|with| with.iter()
                        .filter_map(|x| match *x {
                            internal::WithElement::JsonPath(x) => Some(x.to_owned()),
                            _ => None
                        })
                        .nth(0)
                    ),
                    attributes: match with {
                        Some(with) => with.iter()
                            .filter_map(|x| match *x {
                                internal::WithElement::Attribute(x, y) => Some((x.to_owned(), y.to_owned())),
                                _ => None
                            })
                            .collect(),
                        None => Vec::new()
                    },
                    ignore: match with {
                        Some(with) => with.iter().any(|x| x == &internal::WithElement::Ignore),
                        None => false,
                    },
                    description: e.iter()
                        .filter_map(|x| match *x {
                            internal::EndpointElement::Description(x) => Some(x.to_owned()),
                            _ => None
                        })
                        .nth(0),
                    returns: e.iter()
                        .filter_map(|x| match *x {
                            internal::EndpointElement::Returns(x) => Some(x.to_owned()),
                            _ => None
                        })
                        .nth(0),
                    params: e.iter()
                        .filter_map(|x| match *x {
                            internal::EndpointElement::Params(ref x) => Some(x),
                            _ => None
                        })
                        .nth(0)
                        .map(|params| params.iter()
                            .map(|x| Param {
                                kind: x.kind,
                                type_name_pairs: x.type_name_pairs.iter()
                                    .map(|x| {
                                        let name = if x.name.starts_with('@') { &x.name[1..] } else { x.name };
                                        TypeNamePair {
                                            param_type: x.param_type.to_owned(),
                                            name: name.to_owned()
                                        }
                                    })
                                    .collect(),
                                when: x.when.map(|x| x.to_owned()),
                            })
                            .collect()
                        )
                        .unwrap_or_else(Vec::new),
                }
            );
        }
    }

    Ok(ApiTemplate {
        namespace: namespace,
        description: description,
        endpoints: endpoints,
    })
}
