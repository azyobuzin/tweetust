use std::borrow::Cow;
use std::fmt;
use std::fmt::Write;
use hyper::method::Method;
use ::TwitterResult;
use conn::{Authenticator, Parameter};
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

pub fn execute_core<A, U, R>(auth: &A, method: Method, url: U, params: &[Parameter]) -> TwitterResult<R>
    where A: Authenticator, U: AsRef<str>, R: ::serde::de::Deserialize
{
    auth.request_twitter(method, url.as_ref(), params).and_then(|x| x.parse_to_object())
}

pub trait ToParameter<'a> {
    fn to_parameter(&'a self, key: &'static str) -> Parameter<'a>;
}

impl<'a> ToParameter<'a> for bool {
    fn to_parameter(&'a self, key: &'static str) -> Parameter<'a> {
        Parameter::Value(Cow::Borrowed(key), Cow::Borrowed(if *self { "true" } else { "false" }))
    }
}

impl<'a> ToParameter<'a> for TweetMode {
    fn to_parameter(&'a self, key: &'static str) -> Parameter<'a> {
        Parameter::Value(Cow::Borrowed(key), Cow::Borrowed(match *self {
            TweetMode::Compat => "compat",
            TweetMode::Extended => "extended"
        }))
    }
}

impl<'a> ToParameter<'a> for str {
    fn to_parameter(&'a self, key: &'static str) -> Parameter<'a> {
        Parameter::Value(Cow::Borrowed(key), Cow::Borrowed(self))
    }
}

impl<'a, 'b> ToParameter<'a> for Cow<'b, str> {
    fn to_parameter(&'a self, key: &'static str) -> Parameter<'a> {
        Parameter::Value(Cow::Borrowed(key), Cow::Borrowed(self.as_ref()))
    }
}

macro_rules! to_string_parameter {
    ($t:ty) => (
        impl<'a> ToParameter<'a> for $t {
            fn to_parameter(&'a self, key: &'static str) -> Parameter<'a> {
                Parameter::Value(Cow::Borrowed(key), Cow::Owned(self.to_string()))
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
