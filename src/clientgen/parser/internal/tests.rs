use super::*;
use nom::*;

macro_rules! assert_match {
    ($e:expr, $($p:tt)*) => (
        match $e {
            $($p)* => (),
            x => panic!("Actual: {:?}\nExpected: {}", x, stringify!($($p)*))
        }
    )
}

#[test]
fn comment_test() {
    assert_match!(space_or_comment(b" \t\r\n"), IResult::Done(b"", ()));
    assert_match!(space_or_comment(b"//test\r\na"), IResult::Done(b"a", ()));
    assert_match!(space_or_comment(b"#comment\r\na"), IResult::Done(b"a", ()));
    // #namespace is not a comment
    assert_match!(space_or_comment(b"#namespace\r\na"), IResult::Error(_));
    assert_match!(space_or_comment(b"/*a\r\nb*/c"), IResult::Done(b"c", ()));
}

#[test]
fn neither_space_nor_comment_test() {
    assert_match!(neither_space_nor_comment(b""), IResult::Incomplete(Needed::Size(1)));
    assert_match!(neither_space_nor_comment(b" a"), IResult::Error(Err::Position(ErrorKind::Custom(ERR_NEITHER_SPACE_NOR_COMMENT), b" a")));
    assert_match!(neither_space_nor_comment(b"a "), IResult::Done(b" ", b"a"));
}

#[test]
fn namespace_test() {
    assert_match!(
        namespace(&b"#namespace RestTest\r\n"[..]),
        IResult::Done(b"\r\n", RootElement::Namespace(b"RestTest"))
    );
}

#[test]
fn description_test() {
    assert_match!(
        description(&b"#description This contains several types of api for testing.\r\n"[..]),
        IResult::Done(b"\r\n", RootElement::Description(b"This contains several types of api for testing."))
    );
}

#[test]
fn raw_test() {
    assert_match!(
        raw(&b"#raw\r\nx\r\n#endraw\r\n"[..]),
        IResult::Done(b"\r\n", RootElement::Raw(b"\r\nx\r\n"))
    );
}

#[test]
fn json_path_test() {
    assert_match!(
        json_path(b"JsonPath=resources\r\n"),
        IResult::Done(b"\r\n", WithElement::JsonPath(b"resources"))
    );
}

#[test]
fn omit_except_test() {
    assert_match!(
        omit_except(b"OmitExcept=static,asyncstatic\r\n"),
        IResult::Done(b"\r\n", WithElement::OmitExcept(b"static,asyncstatic"))
    );
}

#[test]
fn attribute_test() {
    assert_match!(
        attribute(b"[Obsolete]=Use Media.Upload and Statuses.Update.\r\n"),
        IResult::Done(b"\r\n", WithElement::Attribute { name: b"Obsolete", value: b"Use Media.Upload and Statuses.Update." })
    );
}

#[test]
fn param_test() {
    assert_match!(
        param(&b"required int required_number\r\n"[..]),
        IResult::Done(b"\r\n", Param {
            kind: ParamKind::Required,
            type_name_pairs: ref tn,
            when: None,
        })
        if &tn[..] == &[TypeNamePair { param_type: b"int", name: b"required_number" }] 
    );

    assert_match!(
        param(&b"optional string optional_string\r\n"[..]),
        IResult::Done(b"\r\n", Param {
            kind: ParamKind::Optional,
            type_name_pairs: ref tn,
            when: None,
        })
        if &tn[..] == &[TypeNamePair { param_type: b"string", name: b"optional_string" }]
    );

    assert_match!(
        param(&b"either FileInfo media when FILEINFO\r\n"[..]),
        IResult::Done(b"\r\n", Param {
            kind: ParamKind::Either(0),
            type_name_pairs: ref tn,
            when: Some(b"FILEINFO"),
        })
        if &tn[..] == &[TypeNamePair { param_type: b"FileInfo", name: b"media" }]
    );

    assert_match!(
        param(&b"either\r\n"[..]),
        IResult::Done(b"\r\n", Param {
            kind: ParamKind::Either(0),
            type_name_pairs: ref tn,
            when: None,
        })
        if tn.is_empty()
    );

    assert_match!(
        param(&b"either string slug, string owner_screen_name\r\n"[..]),
        IResult::Done(b"\r\n", Param {
            kind: ParamKind::Either(0),
            type_name_pairs: ref tn,
            when: None,
        })
        if &tn[..] == &[
            TypeNamePair { param_type: b"string", name: b"slug" },
            TypeNamePair { param_type: b"string", name: b"owner_screen_name" },
        ]
    );

    assert_match!(
        param(&b"either[1] int id_2\r\n"[..]),
        IResult::Done(b"\r\n", Param {
            kind: ParamKind::Either(1),
            type_name_pairs: ref tn,
            when: None,
        })
        if &tn[..] == &[TypeNamePair { param_type: b"int", name: b"id_2" }]
    );
}

#[test]
fn text_endpoint_element_test() {
    assert_match!(
        text_endpoint_element(&b"description\r\n{\r\nDescription of the endpoint.\r\n}\r\n"[..]),
        IResult::Done(b"\r\n", EndpointElement::Description(b"\r\nDescription of the endpoint.\r\n"))
    );

    assert_match!(
        text_endpoint_element(&b"returns\r\n{\r\nDescription of returning value.\r\n}\r\n"[..]),
        IResult::Done(b"\r\n", EndpointElement::Returns(b"\r\nDescription of returning value.\r\n"))
    );

    assert_match!(
        text_endpoint_element(&b"pe // optional\r\n{\r\ncustom.MethodBody(\"for params Expression<>[] overload\");\r\n}\r\n"[..]),
        IResult::Done(b"\r\n", EndpointElement::Other(b"pe", b"\r\ncustom.MethodBody(\"for params Expression<>[] overload\");\r\n"))
    );
}

#[test]
fn endpoint_header_test() {
    assert_match!(
        endpoint_header(&b"endpoint Dictionary<Dictionary<string,RateLimit>> RateLimitStatus : Get application/rate_limit_status\r\n"[..]),
        IResult::Done(b"\r\n", EndpointHeader {
            return_type: b"Dictionary<Dictionary<string,RateLimit>>",
            name: b"RateLimitStatus",
            endpoint_type: EndpointType::Get(b"application/rate_limit_status"),
        })
    );

    // TODO: Post, Impl
}
