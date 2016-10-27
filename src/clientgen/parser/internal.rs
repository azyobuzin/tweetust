use nom::*;

pub enum RootElement<'a> {
    Namespace(&'a [u8]),
    Description(&'a [u8]),
    Endpoint {
        return_type: &'a [u8],
        name: &'a [u8],
        endpoint_type: EndpointType<'a>,
        elements: Vec<EndpointElement<'a>>,
    },
    Raw(&'a [u8]),
}

pub enum EndpointType<'a> {
    Impl,
    Get(&'a [u8]),
    Post(&'a [u8]),
}

pub enum EndpointElement<'a> {
    With(Vec<WithElement<'a>>),
    Description(&'a [u8]),
    Returns(&'a [u8]),
    Params(Vec<Param<'a>>),
    Other { name: &'a [u8], content: &'a [u8] },
}

pub enum WithElement<'a> {
    JsonPath(&'a [u8]),
    OmitExcept(&'a [u8]),
    Attribute { key: &'a [u8], value: &'a [u8] },
}

pub struct Param<'a> {
    pub kind: ParamKind,
    pub type_name_pairs: Vec<TypeNamePair<'a>>,
    pub when: Option<&'a [u8]>
}

pub enum ParamKind {
    Required,
    Either,
    EitherIndex(u8),
    Optional,
}

pub struct TypeNamePair<'a> {
    pub param_type: &'a [u8],
    pub name: &'a [u8],
}

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
                IResult::Error(_) | IResult::Incomplete(_) => {
                    ret = IResult::Done(input, ());
                    break;
                },
                IResult::Done(i, _) if i == input => {
                    ret = IResult::Done(input, ());
                    break;
                },
                IResult::Done(i, _) => { input = i; }
            }
        }

        ret
    });
    ($i:expr, $f:expr) => (many0_ignore!($i, call!($f));)
}

named!(slash_comment<()>, preceded!(tag!("//"), many0_ignore!(not_line_ending)));

named!(hash_comment<()>, chain!(
    char!('#') ~
    not!(alt_complete!(tag!("namespace") | tag!("description") | tag!("raw") | tag!("endraw"))) ~
    many0_ignore!(not_line_ending),
    || ()
));

named!(multi_comment<()>, delimited!(tag!("/*"), many0_ignore!(terminated!(not!(tag!("*/")), anychar)), tag!("*/")));

named!(space_or_comment<()>, many0_ignore!(alt_complete!(map!(multispace, |_| ()) | slash_comment | hash_comment | multi_comment)));

#[test]
fn comment_test() {
    assert_eq!(space_or_comment(&b" \t\r\n"[..]), IResult::Done(&b""[..], ()));
    assert_eq!(space_or_comment(&b"//test\r\na"[..]), IResult::Done(&b"a"[..], ()));
    assert_eq!(space_or_comment(&b"#comment\r\na"[..]), IResult::Done(&b"a"[..], ()));
    // #namespace is not a comment
    assert_eq!(space_or_comment(&b"#namespace\r\na"[..]), IResult::Done(&b"#namespace\r\na"[..], ()));
    assert_eq!(space_or_comment(&b"/*a\r\nb*/c"[..]), IResult::Done(&b"c"[..], ()));
}
