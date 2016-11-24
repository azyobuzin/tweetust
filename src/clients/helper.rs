use std::borrow::Cow;
use std::fmt;
use std::fmt::Write;
use std::io;
use hyper::method::Method;
use ::TwitterResult;
use conn::*;
use models::TweetMode;

pub fn collection_paramter<I, D>(values: I) -> String
    where I: IntoIterator<Item=D>, D: fmt::Display
{
    let mut iter = values.into_iter();
    let mut dest = String::new();

    if let Some(v) = iter.next() {
        write!(dest, "{}", v).unwrap();

        while let Some(v) = iter.next() {
            write!(dest, ",{}", v).unwrap();
        }
    }

    dest
}

pub fn str_collection_parameter<I, S>(values: I) -> String
    where I: IntoIterator<Item=S>, S: AsRef<str>
{
    let mut iter = values.into_iter();
    let mut dest = String::new();

    if let Some(v) = iter.next() {
        dest.push_str(v.as_ref());

        while let Some(v) = iter.next() {
            dest.push(',');
            dest.push_str(v.as_ref());
        }
    }

    dest
}

pub fn execute_core<'a, A, H, U, R>(client: &super::TwitterClient<A, H>, method: Method,
    url: U, params: Vec<(Cow<'a, str>, ParameterValue<'a>)>) -> TwitterResult<R>
    where A: Authenticator, H: HttpHandler, U: AsRef<str>, R: ::serde::de::Deserialize
{
    let req = try!(Request::new(method, url.as_ref(), RequestContent::from_name_value_pairs(params)));
    try!(client.handler.send_request(req, &client.auth)).parse_to_object()
}

pub trait IntoParameterValue<'a> {
    fn into_parameter_value(self) -> ParameterValue<'a>;
}

impl<'a> IntoParameterValue<'a> for bool {
    fn into_parameter_value(self) -> ParameterValue<'a> {
        ParameterValue::Text(Cow::Borrowed(if self { "true" } else { "false" }))
    }
}

impl<'a> IntoParameterValue<'a> for TweetMode {
    fn into_parameter_value(self) -> ParameterValue<'a> {
        ParameterValue::Text(Cow::Borrowed(match self {
            TweetMode::Compat => "compat",
            TweetMode::Extended => "extended"
        }))
    }
}

impl<'a> IntoParameterValue<'a> for String {
    fn into_parameter_value(self) -> ParameterValue<'a> {
        ParameterValue::Text(Cow::Owned(self))
    }
}

impl<'a> IntoParameterValue<'a> for Cow<'a, str> {
    fn into_parameter_value(self) -> ParameterValue<'a> {
        ParameterValue::Text(self)
    }
}

macro_rules! to_string_parameter {
    ($t:ty) => (
        impl<'a> IntoParameterValue<'a> for $t {
            fn into_parameter_value(self) -> ParameterValue<'a> {
                ParameterValue::Text(Cow::Owned(self.to_string()))
            }
        }
    )
}

to_string_parameter!(i32);
to_string_parameter!(u32);
to_string_parameter!(i64);
to_string_parameter!(u64);
to_string_parameter!(f32);
to_string_parameter!(f64);

impl<'a> IntoParameterValue<'a> for Box<io::Read> {
    fn into_parameter_value(self) -> ParameterValue<'a> {
        ParameterValue::File(self)
    }
}
