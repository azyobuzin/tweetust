use nom::*;

#[derive(Debug)]
pub enum RootElement<'a> {
    Namespace(&'a [u8]),
    Description(&'a [u8]),
    Endpoint { header: EndpointHeader<'a>, elements: Vec<EndpointElement<'a>> },
    Raw(&'a [u8]),
}

#[derive(Debug, PartialEq, Eq)]
pub struct EndpointHeader<'a> {
    pub return_type: &'a [u8],
    pub name: &'a [u8],
    pub endpoint_type: EndpointType<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EndpointType<'a> {
    Impl,
    Get(&'a [u8]),
    Post(&'a [u8]),
}

#[derive(Debug)]
pub enum EndpointElement<'a> {
    With(Vec<WithElement<'a>>),
    Description(&'a [u8]),
    Returns(&'a [u8]),
    Params(Vec<Param<'a>>),
    Other { name: &'a [u8], content: &'a [u8] },
}

#[derive(Debug, PartialEq, Eq)]
pub enum WithElement<'a> {
    JsonPath(&'a [u8]),
    OmitExcept(&'a [u8]),
    Attribute { name: &'a [u8], value: &'a [u8] },
}

#[derive(Debug)]
pub struct Param<'a> {
    pub kind: ParamKind,
    pub type_name_pairs: Vec<TypeNamePair<'a>>,
    pub when: Option<&'a [u8]>
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParamKind {
    Required,
    Either,
    EitherIndex(u8),
    Optional,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypeNamePair<'a> {
    pub param_type: &'a [u8],
    pub name: &'a [u8],
}

const ERR_MANY0_IGNORE: u32 = 1;
const ERR_MANY1_IGNORE: u32 = 2;
const ERR_NEITHER_SPACE_NOR_COMMENT: u32 = 10;

macro_rules! assert_match {
    ($e:expr, $p:pat) => (
        match $e {
            $p => (),
            x => panic!("{:?}", x)
        }
    )
}

macro_rules! many0_ignore {
    ($i:expr, $submac:ident!( $($args:tt)* )) => ({
        let ret;
        let first_input = $i;
        let mut input = first_input;

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
                    ret = IResult::Incomplete(Needed::Size(i + first_input.input_len() - input.input_len()));
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
        let first_input = $i;
        match $submac!(first_input, $($args)*) {
            IResult::Error(e) => IResult::Error(Err::NodePosition(ErrorKind::Custom(ERR_MANY1_IGNORE), first_input, Box::new(e))),
            IResult::Incomplete(i) => IResult::Incomplete(i),
            IResult::Done(i1,o1) => {
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
                            ret = IResult::Incomplete(Needed::Size(i + first_input.input_len() - input.input_len()));
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

named!(slash_comment<()>, chain!(complete!(tag!("//")) ~ not_line_ending, || ()));

named!(hash_comment<()>, chain!(
    complete!(char!('#')) ~
    not!(alt_complete!(tag!("namespace") | tag!("description") | tag!("raw") | tag!("endraw"))) ~
    not_line_ending,
    || ()
));


named!(multi_comment<()>, chain!(
    complete!(tag!("/*")) ~
    take_until_and_consume!("*/"),
    || ()
));

named!(space_or_comment<()>, many1_ignore!(alt!(map!(multispace, |_| ()) | slash_comment | hash_comment | multi_comment)));
named!(space_or_comment0<Option<()> >, opt!(space_or_comment));

#[test]
fn comment_test() {
    assert_eq!(space_or_comment(&b" \t\r\n"[..]), IResult::Done(&b""[..], ()));
    assert_eq!(space_or_comment(&b"//test\r\na"[..]), IResult::Done(&b"a"[..], ()));
    assert_eq!(space_or_comment(&b"#comment\r\na"[..]), IResult::Done(&b"a"[..], ()));
    // #namespace is not a comment
    assert_match!(space_or_comment(&b"#namespace\r\na"[..]), IResult::Error(_));
    assert_eq!(space_or_comment(&b"/*a\r\nb*/c"[..]), IResult::Done(&b"c"[..], ()));
}

fn neither_space_nor_comment(input: &[u8]) -> IResult<&[u8], &[u8], u32> {
    if input.is_empty() {
        return IResult::Incomplete(Needed::Size(1));
    }

    // first time
    match space_or_comment(input) {
        IResult::Done(_, _) => return IResult::Error(Err::Position(ErrorKind::Custom(ERR_NEITHER_SPACE_NOR_COMMENT), input)),
        IResult::Incomplete(x) => return IResult::Incomplete(x),
        IResult::Error(_) => ()
    }

    for taken in 1..input.len() {
        match space_or_comment(&input[taken..]) {
            IResult::Done(_, _) => return IResult::Done(&input[taken..], &input[..taken]),
            IResult::Incomplete(Needed::Unknown) => return IResult::Incomplete(Needed::Unknown),
            IResult::Incomplete(Needed::Size(i)) => return IResult::Incomplete(Needed::Size(i + taken)),
            IResult::Error(_) => ()
        }
    }

    IResult::Done(&input[input.len()..], input)
}

#[test]
fn neither_space_nor_comment_test() {
    assert_eq!(neither_space_nor_comment(&b""[..]), IResult::Incomplete(Needed::Size(1)));
    assert_eq!(neither_space_nor_comment(&b" a"[..]), IResult::Error(Err::Position(ErrorKind::Custom(ERR_NEITHER_SPACE_NOR_COMMENT), &b" a"[..])));
    assert_eq!(neither_space_nor_comment(&b"a "[..]), IResult::Done(&b" "[..], &b"a"[..]));
}

named!(namespace<RootElement>, chain!(
    complete!(tag!("#namespace")) ~
    space ~
    x: not_line_ending,
    || RootElement::Namespace(x)
));

#[test]
fn namespace_test() {
    assert_match!(
        namespace(&b"#namespace RestTest\r\n"[..]),
        IResult::Done(b"\r\n", RootElement::Namespace(b"RestTest"))
    );
}

named!(description<RootElement>, chain!(
    complete!(tag!("#description")) ~
    space ~
    x: not_line_ending,
    || RootElement::Description(x)
));

#[test]
fn description_test() {
    assert_match!(
        description(&b"#description This contains several types of api for testing.\r\n"[..]),
        IResult::Done(b"\r\n", RootElement::Description(b"This contains several types of api for testing."))
    );
}

named!(raw<RootElement>, chain!(
    complete!(tag!("#raw")) ~
    x: take_until_and_consume!("#endraw"),
    || RootElement::Raw(x)
));

#[test]
fn raw_test() {
    assert_match!(
        raw(&b"#raw\r\nx\r\n#endraw\r\n"[..]),
        IResult::Done(b"\r\n", RootElement::Raw(b"\r\nx\r\n"))
    );
}

named!(json_path<WithElement>, chain!(
    complete!(tag!("JsonPath=")) ~
    x: not_line_ending,
    || WithElement::JsonPath(x)
));

#[test]
fn json_path_test() {
    assert_eq!(
        json_path(&b"JsonPath=resources\r\n"[..]),
        IResult::Done(&b"\r\n"[..], WithElement::JsonPath(&b"resources"[..]))
    );
}

named!(omit_except<WithElement>, chain!(
    complete!(tag!("OmitExcept=")) ~
    x: not_line_ending,
    || WithElement::OmitExcept(x)
));

#[test]
fn omit_except_test() {
    assert_eq!(
        omit_except(&b"OmitExcept=static,asyncstatic\r\n"[..]),
        IResult::Done(&b"\r\n"[..], WithElement::OmitExcept(&b"static,asyncstatic"[..]))
    );
}

named!(attribute<WithElement>, chain!(
    complete!(char!('[')) ~
    name: take_until_and_consume!("]") ~
    char!('=') ~
    value: not_line_ending,
    || WithElement::Attribute { name: name, value: value }
));

#[test]
fn attribute_test() {
    assert_eq!(
        attribute(&b"[Obsolete]=Use Media.Upload and Statuses.Update.\r\n"[..]),
        IResult::Done(&b"\r\n"[..], WithElement::Attribute { name: &b"Obsolete"[..], value: &b"Use Media.Upload and Statuses.Update."[..] })
    );
}
