use super::*;
use super::super::{EndpointType, Param, ParamKind, TypeNamePair};
use nom::*;

#[test]
fn comment_test() {
    assert_matches!(space_or_comment(" \t\r\n"), IResult::Done("", ()));
    assert_matches!(space_or_comment("//test\r\na"), IResult::Done("a", ()));
    assert_matches!(space_or_comment("#comment\r\na"), IResult::Done("a", ()));
    // #namespace is not a comment
    assert_matches!(space_or_comment("#namespace\r\na"), IResult::Error(_));
    assert_matches!(space_or_comment("/*a\r\nb*/c"), IResult::Done("c", ()));
}

#[test]
fn neither_space_nor_comment_test() {
    assert_matches!(neither_space_nor_comment(""), IResult::Incomplete(Needed::Size(1)));
    assert_matches!(neither_space_nor_comment(" a"), IResult::Error(Err::Position(ErrorKind::Custom(ERR_NEITHER_SPACE_NOR_COMMENT), " a")));
    assert_matches!(neither_space_nor_comment("a "), IResult::Done(" ", "a"));
}

#[test]
fn namespace_test() {
    assert_matches!(
        namespace("#namespace RestTest\r\n"),
        IResult::Done("\r\n", RootElement::Namespace(RestTest))
    );
}

#[test]
fn description_test() {
    assert_matches!(
        description("#description This contains several types of api for testing.\r\n"),
        IResult::Done("\r\n", RootElement::Description("This contains several types of api for testing."))
    );
}

#[test]
fn raw_test() {
    assert_matches!(
        raw("#raw\r\nx\r\n#endraw\r\n"),
        IResult::Done("\r\n", RootElement::Raw("\r\nx\r\n"))
    );
}

#[test]
fn json_path_test() {
    assert_matches!(
        json_path("JsonPath=resources\r\n"),
        IResult::Done("\r\n", WithElement::JsonPath("resources"))
    );
}

#[test]
fn omit_except_test() {
    assert_matches!(
        omit_except("OmitExcept=static,asyncstatic\r\n"),
        IResult::Done("\r\n", WithElement::OmitExcept("static,asyncstatic"))
    );
}

#[test]
fn attribute_test() {
    assert_matches!(
        attribute("[Obsolete]=Use Media.Upload and Statuses.Update.\r\n"),
        IResult::Done("\r\n", WithElement::Attribute("Obsolete", "Use Media.Upload and Statuses.Update."))
    );
}

#[test]
fn param_test() {
    assert_matches!(
        param("required int required_number\r\n"),
        IResult::Done("\r\n", Param {
            kind: ParamKind::Required,
            type_name_pairs: ref tn,
            when: None,
        })
        if &tn[..] == &[TypeNamePair { param_type: "int", name: "required_number" }] 
    );

    assert_matches!(
        param("optional string optional_string\r\n"),
        IResult::Done("\r\n", Param {
            kind: ParamKind::Optional,
            type_name_pairs: ref tn,
            when: None,
        })
        if &tn[..] == &[TypeNamePair { param_type: "string", name: "optional_string" }]
    );

    assert_matches!(
        param("either FileInfo media when FILEINFO\r\n"),
        IResult::Done("\r\n", Param {
            kind: ParamKind::Either(0),
            type_name_pairs: ref tn,
            when: Some("FILEINFO"),
        })
        if &tn[..] == &[TypeNamePair { param_type: "FileInfo", name: "media" }]
    );

    assert_matches!(
        param("either\r\n"),
        IResult::Done("\r\n", Param {
            kind: ParamKind::Either(0),
            type_name_pairs: ref tn,
            when: None,
        })
        if tn.is_empty()
    );

    assert_matches!(
        param("either string slug, string owner_screen_name\r\n"),
        IResult::Done("\r\n", Param {
            kind: ParamKind::Either(0),
            type_name_pairs: ref tn,
            when: None,
        })
        if &tn[..] == &[
            TypeNamePair { param_type: "string", name: "slug" },
            TypeNamePair { param_type: "string", name: "owner_screen_name" },
        ]
    );

    assert_matches!(
        param("either[1] int id_2\r\n"),
        IResult::Done("\r\n", Param {
            kind: ParamKind::Either(1),
            type_name_pairs: ref tn,
            when: None,
        })
        if &tn[..] == &[TypeNamePair { param_type: "int", name: "id_2" }]
    );
}

#[test]
fn params_test() {
    match params(r#"params
    {
        required string id
        required IEnumerable<CollectionEntryChange> changes
    } "#)
    {
        IResult::Done(" ", EndpointElement::Params(ref params)) => {
            assert_eq!(params.len(), 2);
            assert_matches!(
                params[0],
                Param { kind: ParamKind::Required, type_name_pairs: ref tn, when: None }
                if &tn[..] == &[TypeNamePair { param_type: "string", name: "id" }]
            );
            assert_matches!(
                params[1],
                Param { kind: ParamKind::Required, type_name_pairs: ref tn, when: None }
                if &tn[..] == &[TypeNamePair { param_type: "IEnumerable<CollectionEntryChange>", name: "changes" }]
            );
        },
        x => panic!("{:?}", x)
    }

    // root_test
    assert_matches!(
        params(r#"params
    {
        either
        either string resources
        either IEnumerable<string> resources
    } "#),
        IResult::Done(" ", EndpointElement::Params(_))
    );
}

#[test]
fn text_endpoint_element_test() {
    assert_matches!(
        text_endpoint_element("description\r\n{\r\nDescription of the endpoint.\r\n}\r\n"),
        IResult::Done("\r\n", EndpointElement::Description("\r\nDescription of the endpoint.\r\n"))
    );

    assert_matches!(
        text_endpoint_element("returns\r\n{\r\nDescription of returning value.\r\n}\r\n"),
        IResult::Done("\r\n", EndpointElement::Returns("\r\nDescription of returning value.\r\n"))
    );

    assert_matches!(
        text_endpoint_element("pe // optional\r\n{\r\ncustom.MethodBody(\"for params Expression<>[] overload\");\r\n}\r\n"),
        IResult::Done("\r\n", EndpointElement::Other("pe", "\r\ncustom.MethodBody(\"for params Expression<>[] overload\");\r\n"))
    );
}

#[test]
fn endpoint_header_test() {
    assert_matches!(
        endpoint_header("endpoint Dictionary<Dictionary<string,RateLimit>> RateLimitStatus : Get application/rate_limit_status\r\n"),
        IResult::Done("\r\n", EndpointHeader {
            return_type: "Dictionary<Dictionary<string,RateLimit>>",
            name: "RateLimitStatus",
            endpoint_type: EndpointType::Get("application/rate_limit_status"),
        })
    );

    assert_matches!(
        endpoint_header("endpoint void RemoveProfileBanner : Post account/remove_profile_banner\r\n"),
        IResult::Done("\r\n", EndpointHeader {
            return_type: "void",
            name: "RemoveProfileBanner",
            endpoint_type: EndpointType::Post("account/remove_profile_banner"),
        })
    );

    assert_matches!(
        endpoint_header("endpoint UploadInitCommandResult UploadInitCommand : Impl\r\n"),
        IResult::Done("\r\n", EndpointHeader {
            return_type: "UploadInitCommandResult",
            name: "UploadInitCommand",
            endpoint_type: EndpointType::Impl,
        })
    );
}

#[test]
fn root_test() {
    const application_api: &'static str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../clientgen/CoreTweet/ApiTemplates/application.api"));

    match root(application_api) {
        IResult::Done("", re) => {
            assert_eq!(re.len(), 3);
            assert_matches!(re[0], RootElement::Namespace("Application"));
            assert_matches!(re[1], RootElement::Description("Provides a set of methods for the wrapper of GET application."));
            match re[2] {
                RootElement::Endpoint { ref header, ref elements } => {
                    assert_matches!(header, &EndpointHeader {
                        return_type: "Dictionary<Dictionary<string,RateLimit>>",
                        name: "RateLimitStatus",
                        endpoint_type: EndpointType::Get("application/rate_limit_status"),
                    });

                    assert_eq!(elements.len(), 4);
                    assert_matches!(
                        elements[0],
                        EndpointElement::With(ref x)
                        if &x[..] == &[WithElement::JsonPath("resources")]
                    );
                    assert_matches!(
                        elements[1],
                        EndpointElement::Description(x)
                        if x.trim() == "Returns the current rate limits for methods belonging to the specified resource families."
                    );
                    assert_matches!(
                        elements[2],
                        EndpointElement::Returns(x)
                        if x.trim() == "The dictionary."
                    );
                    match elements[3] {
                        EndpointElement::Params(ref params) => {
                            assert_eq!(params.len(), 3);
                            assert_matches!(
                                params[0],
                                Param { kind: ParamKind::Either(0), type_name_pairs: ref tn, when: None }
                                if tn.len() == 0
                            );
                            assert_matches!(
                                params[1],
                                Param { kind: ParamKind::Either(0), type_name_pairs: ref tn, when: None }
                                if &tn[..] == &[TypeNamePair { param_type: "string", name: "resources" }]
                            );
                            assert_matches!(
                                params[2],
                                Param { kind: ParamKind::Either(0), type_name_pairs: ref tn, when: None }
                                if &tn[..] == &[TypeNamePair { param_type: "IEnumerable<string>", name: "resources" }]
                            );
                        },
                        ref x => panic!("elements[3] = {:?}", x)
                    }
                },
                ref x => panic!("re[2] = {:?}", x)
            }
        },
        x => panic!("{:?}", x)
    }
}
