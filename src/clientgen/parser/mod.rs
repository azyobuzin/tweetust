mod internal;

#[derive(Debug)]
pub struct ApiTemplate<'a> {
    pub namespace: &'a str,
    pub description: Option<&'a str>,
    pub endpoints: Vec<Endpoint<'a>>,
}

#[derive(Debug)]
pub struct Endpoint<'a> {
    pub return_type: &'a str,
    pub name: &'a str,
    pub endpoint_type: EndpointType<'a>,
    pub json_path: Option<&'a str>,
    pub attributes: Vec<(&'a str, &'a str)>,
    pub description: Option<&'a str>,
    pub returns: Option<&'a str>,
    pub params: Vec<Param<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EndpointType<'a> {
    Get(&'a str),
    Post(&'a str),
    Impl,
}

#[derive(Debug)]
pub struct Param<'a> {
    pub kind: ParamKind,
    pub type_name_pairs: Vec<TypeNamePair<'a>>,
    pub when: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParamKind {
    Required,
    Either(u8),
    Optional,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypeNamePair<'a> {
    pub param_type: &'a str,
    pub name: &'a str,
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
                internal::RootElement::Namespace(x) => Some(x),
                _ => None
            })
            .nth(0);
        if let Some(x) = ns { x }
        else { return Err(ParseErrorKind::Missing("#namespace")) }
    };

    let description = root.iter()
        .filter_map(|x| match *x {
            internal::RootElement::Description(x) => Some(x),
            _ => None
        })
        .nth(0);

    let mut endpoints = Vec::with_capacity(root.len() - (if description.is_some() { 2 } else { 1 }));
    for x in root.into_iter() {
        if let internal::RootElement::Endpoint { header: h, elements: e } = x {
            let (json_path, attributes) = {
                let with = e.iter()
                    .filter_map(|x| match *x {
                        internal::EndpointElement::With(ref x) => Some(x),
                        _ => None
                    })
                    .nth(0);
                let json_path = with.and_then(|with| with.iter()
                    .filter_map(|x| match *x {
                        internal::WithElement::JsonPath(x) => Some(x),
                        _ => None
                    })
                    .nth(0)
                );
                let attributes = match with {
                    Some(with) => with.iter()
                        .filter_map(|x| match *x {
                            internal::WithElement::Attribute(x, y) => Some((x, y)),
                            _ => None
                        })
                        .collect(),
                    None => Vec::new()
                };
                (json_path, attributes)
            };

            endpoints.push(
                Endpoint {
                    return_type: h.return_type,
                    name: h.name,
                    endpoint_type: h.endpoint_type,
                    json_path: json_path,
                    attributes: attributes,
                    description: e.iter()
                        .filter_map(|x| match *x {
                            internal::EndpointElement::Description(x) => Some(x),
                            _ => None
                        })
                        .nth(0),
                    returns: e.iter()
                        .filter_map(|x| match *x {
                            internal::EndpointElement::Returns(x) => Some(x),
                            _ => None
                        })
                        .nth(0),
                    params: e.into_iter()
                        .filter_map(|x| match x {
                            internal::EndpointElement::Params(x) => Some(x),
                            _ => None
                        })
                        .nth(0)
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
