use super::ParamKind;
use std::str;
use std::str::FromStr;
use nom::*;

#[cfg(test)] mod tests;

#[derive(Debug)]
pub enum RootElement<'a> {
    Namespace(&'a str),
    Description(&'a str),
    Endpoint { header: EndpointHeader<'a>, elements: Vec<EndpointElement<'a>> },
    Raw(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct EndpointHeader<'a> {
    pub return_type: &'a str,
    pub name: &'a str,
    pub endpoint_type: EndpointType<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EndpointType<'a> {
    Get(&'a str),
    Post(&'a str),
    Impl,
}

#[derive(Debug)]
pub enum EndpointElement<'a> {
    With(Vec<WithElement<'a>>),
    Description(&'a str),
    Returns(&'a str),
    Params(Vec<Param<'a>>),
    Other(&'a str, &'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum WithElement<'a> {
    JsonPath(&'a str),
    OmitExcept(&'a str),
    Attribute(&'a str, &'a str),
}

#[derive(Debug)]
pub struct Param<'a> {
    pub kind: ParamKind,
    pub type_name_pairs: Vec<TypeNamePair<'a>>,
    pub when: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypeNamePair<'a> {
    pub param_type: &'a str,
    pub name: &'a str,
}

#[allow(dead_code)] pub const ERR_MANY0_IGNORE: u32 = 1;
#[allow(dead_code)] pub const ERR_MANY1_IGNORE: u32 = 2;
#[allow(dead_code)] pub const ERR_TAKE_UNTIL_AND_CONSUME_S2: u32 = 3;
#[allow(dead_code)] pub const ERR_NEITHER_SPACE_NOR_COMMENT: u32 = 10;

macro_rules! many0_ignore {
    ($i:expr, $submac:ident!( $($args:tt)* )) => ({
        let ret;
        let mut input = $i;

        loop {
            if input.input_len() == 0 {
                ret = IResult::Done(input, ());
                break;
            }

            match $submac!(input, $($args)*) {
                IResult::Error(_) => {
                    ret = IResult::Done(input, ());
                    break;
                },
                IResult::Incomplete(Needed::Unknown) => {
                    ret = IResult::Incomplete(Needed::Unknown);
                    break;
                },
                IResult::Incomplete(Needed::Size(i)) => {
                    ret = IResult::Incomplete(Needed::Size(i + $i.input_len() - input.input_len()));
                    break;
                },
                IResult::Done(i, _) if i.input_len() == input.input_len() => {
                    ret = IResult::Error(Err::Position(ErrorKind::Custom(ERR_MANY0_IGNORE), input));
                    break;
                },
                IResult::Done(i, _) => input = i
            }
        }

        ret
    });
    ($i:expr, $f:expr) => (many0_ignore!($i, call!($f));)
}

macro_rules! many1_ignore {
    ($i:expr, $submac:ident!( $($args:tt)* )) => ({
        match $submac!($i, $($args)*) {
            IResult::Error(e) => IResult::Error(Err::NodePosition(ErrorKind::Custom(ERR_MANY1_IGNORE), $i, Box::new(e))),
            IResult::Incomplete(i) => IResult::Incomplete(i),
            IResult::Done(i1, _) => {
                let ret;
                let mut input = i1;

                loop {
                    if input.input_len() == 0 {
                        ret = IResult::Done(input, ());
                        break;
                    }

                    match $submac!(input, $($args)*) {
                        IResult::Error(_) => {
                            ret = IResult::Done(input, ());
                            break;
                        },
                        IResult::Incomplete(Needed::Unknown) => {
                            ret = IResult::Incomplete(Needed::Unknown);
                            break;
                        },
                        IResult::Incomplete(Needed::Size(i)) => {
                            ret = IResult::Incomplete(Needed::Size(i + $i.input_len() - input.input_len()));
                            break;
                        },
                        IResult::Done(i, _) if i.input_len() == input.input_len() => {
                            ret = IResult::Done(input, ());
                            break;
                        },
                        IResult::Done(i, _) => input = i
                    }
                }

                ret
            }
        }
    });
    ($i:expr, $f:expr) => (many1_ignore!($i, call!($f));)
}

/// Work around the bug of take_until_and_consume_s
macro_rules! take_until_and_consume_s2 {
    ($input:expr, $substr:expr) => ({
        let substr = $substr;
        if $input.input_len() < substr.len() {
            IResult::Incomplete(Needed::Size(substr.len() - $input.input_len()))
        } else {
            match $input.find(substr) {
                Some(i) => IResult::Done(&$input[i + substr.len()..], &$input[..i]),
                None => IResult::Error(Err::Position(ErrorKind::Custom(ERR_TAKE_UNTIL_AND_CONSUME_S2), $input))
            }
        }
    })
}

named!(take_until_line_ending<&str, &str>, take_till_s!(call!(|c| matches!(c, '\r' | '\n'))));

named!(slash_comment<&str, ()>, chain!(complete!(tag_s!("//")) ~ take_until_line_ending, || ()));

named!(hash_comment<&str, ()>, chain!(
    complete!(tag_s!("#")) ~
    // alt_complete! does not complete! parsers later than first
    not!(alt!(complete!(tag_s!("namespace")) | complete!(tag_s!("description")) | complete!(tag_s!("raw")) | complete!(tag_s!("endraw")))) ~
    take_until_line_ending,
    || ()
));

named!(multi_comment<&str, ()>, chain!(
    complete!(tag_s!("/*")) ~
    take_until_and_consume_s2!("*/"),
    || ()
));

named!(pub space_or_comment<&str, ()>, many1_ignore!(alt!(value!((), multispace) | slash_comment | hash_comment | multi_comment)));
named!(space_or_comment0<&str, Option<()> >, opt!(space_or_comment));

pub fn neither_space_nor_comment(input: &str) -> IResult<&str, &str, u32> {
    if input.is_empty() {
        return IResult::Incomplete(Needed::Size(1));
    }

    // first time
    match space_or_comment(input) {
        IResult::Done(_, _) => return IResult::Error(Err::Position(ErrorKind::Custom(ERR_NEITHER_SPACE_NOR_COMMENT), input)),
        IResult::Incomplete(x) => return IResult::Incomplete(x),
        IResult::Error(_) => ()
    }

    for (i, _) in input.char_indices().skip(1) {
        match space_or_comment(&input[i..]) {
            IResult::Done(_, _) => return IResult::Done(&input[i..], &input[..i]),
            IResult::Incomplete(Needed::Unknown) => return IResult::Incomplete(Needed::Unknown),
            IResult::Incomplete(Needed::Size(s)) => return IResult::Incomplete(Needed::Size(s + i)),
            IResult::Error(_) => ()
        }
    }

    IResult::Done(&input[input.len()..], input)
}

named!(pub namespace<&str, RootElement>, chain!(
    complete!(tag_s!("#namespace")) ~
    space ~
    x: take_until_line_ending,
    || RootElement::Namespace(x.trim())
));

named!(pub description<&str, RootElement>, chain!(
    complete!(tag_s!("#description")) ~
    space ~
    x: take_until_line_ending,
    || RootElement::Description(x.trim())
));

named!(pub raw<&str, RootElement>, chain!(
    complete!(tag_s!("#raw")) ~
    x: take_until_and_consume_s2!("#endraw"),
    || RootElement::Raw(x)
));

named!(pub json_path<&str, WithElement>, chain!(
    complete!(tag_s!("JsonPath=")) ~
    x: take_until_line_ending,
    || WithElement::JsonPath(x.trim())
));

named!(pub omit_except<&str, WithElement>, chain!(
    complete!(tag_s!("OmitExcept=")) ~
    x: take_until_line_ending,
    || WithElement::OmitExcept(x.trim())
));

named!(pub attribute<&str, WithElement>, chain!(
    complete!(tag_s!("[")) ~
    name: take_until_and_consume_s2!("]") ~
    tag_s!("=") ~
    value: take_until_line_ending,
    || WithElement::Attribute(name.trim(), value.trim())
));

named!(with<&str, EndpointElement>, chain!(
    complete!(tag_s!("with")) ~
    space_or_comment0 ~
    tag_s!("{") ~
    space_or_comment0 ~
    x: many0!(terminated!(
        alt!(json_path | omit_except | attribute),
        space_or_comment0
    )) ~
    tag_s!("}"),
    || EndpointElement::With(x)
));

fn is_valid_for_param_ident(c: char) -> bool {
    !matches!(c, ' ' | '\t' | '\r' | '\n' | ',' | '}')
}

named!(pub param<&str, Param>, chain!(
    k: alt!(
        value!(ParamKind::Required, complete!(tag_s!("required"))) |
        value!(ParamKind::Optional, alt!(complete!(tag_s!("semi-optional")) | complete!(tag_s!("optional")))) |
        chain!(
            complete!(tag_s!("either")) ~
            x: opt!(map_res!(
                delimited!(complete!(tag_s!("[")), digit, tag_s!("]")),
                u8::from_str
            )),
            || ParamKind::Either(x.unwrap_or(0))
        )
    ) ~
    tn: opt!(preceded!(
        complete!(space_or_comment),
        separated_nonempty_list!(
            chain!(space_or_comment0 ~ complete!(tag_s!(",")) ~ space_or_comment0, || ()),
            chain!(
                t: take_while1_s!(is_valid_for_param_ident) ~
                space_or_comment ~
                n: take_while1_s!(is_valid_for_param_ident),
                || TypeNamePair { param_type: t, name: n }
            )
        )
    )) ~
    w: opt!(chain!(
        complete!(space_or_comment) ~
        complete!(tag_s!("when")) ~
        space ~
        x: take_until_line_ending,
        || x.trim()
    )),
    move || Param { kind: k, type_name_pairs: tn.unwrap_or_else(Vec::new), when: w }
));

named!(pub params<&str, EndpointElement>, chain!(
    complete!(tag_s!("params")) ~
    space_or_comment0 ~
    tag_s!("{") ~
    space_or_comment0 ~
    x: many0!(terminated!(
        flat_map!(
            take_until_line_ending,
            param
        ),
        space_or_comment0
    )) ~
    tag_s!("}"),
    || EndpointElement::Params(x)
));

named!(pub text_endpoint_element<&str, EndpointElement>, chain!(
    n: alphanumeric ~ // do not complete! because it won't return Incomplete
    space_or_comment0 ~
    tag_s!("{") ~
    c: take_until_and_consume_s2!("}"),
    || match n {
        "description" => EndpointElement::Description(c),
        "returns" => EndpointElement::Returns(c),
        n => EndpointElement::Other(n, c)
    }
));

named!(pub endpoint_header<&str, EndpointHeader>, chain!(
    complete!(tag_s!("endpoint")) ~
    space_or_comment ~
    rt: neither_space_nor_comment ~
    space_or_comment ~
    n: alphanumeric ~
    space_or_comment0 ~
    tag_s!(":") ~
    space_or_comment0 ~
    et: alt!(
        value!(EndpointType::Impl, tag_s!("Impl")) |
        chain!(
            tag_s!("Get") ~
            space_or_comment ~
            x: neither_space_nor_comment,
            || EndpointType::Get(x)
        ) |
        chain!(
            tag_s!("Post") ~
            space_or_comment ~
            x: neither_space_nor_comment,
            || EndpointType::Post(x)
        )
    ),
    || EndpointHeader { return_type: rt, name: n, endpoint_type: et }
));

named!(endpoint<&str, RootElement>, chain!(
    h: endpoint_header ~
    space_or_comment0 ~
    tag_s!("{") ~
    space_or_comment0 ~
    e: many0!(terminated!(
        alt!(with | params | text_endpoint_element),
        space_or_comment0
    )) ~
    tag_s!("}"),
    || RootElement::Endpoint { header: h, elements: e }
));

named!(pub root<&str, Vec<RootElement> >, complete!(terminated!(
    many1!(terminated!(
        alt!(namespace | description | endpoint | raw),
        space_or_comment0
    )),
    eof
)));

pub fn parse_api_template(input: &str) -> Result<Vec<RootElement>, Err<&str>> {
    match root(input) {
        IResult::Done(_, x) => Ok(x),
        IResult::Error(x) => Err(x),
        IResult::Incomplete(_) => unreachable!() // root is wrapping parsers with complete!
    }
}
