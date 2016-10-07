use std::borrow::Cow;
use std::fmt;
use std::fmt::Write;
use hyper::method::Method;
use ::TwitterResult;
use conn::{Authenticator, Parameter};

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

    return dest;
}

pub fn str_collection_parameter<'a, I, S>(values: I) -> String
    where I: IntoIterator<Item=S>, S: Into<Cow<'a, str>>
{
    let mut iter = values.into_iter();
    let mut dest = String::new();

    if let Some(v) = iter.next() {
        dest.push_str(v.into().as_ref());

        while let Some(v) = iter.next() {
            dest.push(',');
            dest.push_str(v.into().as_ref());
        }
    }

    return dest;
}

pub fn bool_parameter<'a>(key: &'static str, val: &'a bool) -> Parameter<'a> {
    Parameter::Value(Cow::Borrowed(key), Cow::Borrowed(if *val { "true" } else { "false" }))
}

pub fn owned_str_parameter<'a>(key: &'static str, val: &'a str) -> Parameter<'a> {
    Parameter::Value(Cow::Borrowed(key), Cow::Borrowed(val))
}

pub fn cow_str_parameter<'a>(key: &'static str, val: &'a Cow<'a, str>) -> Parameter<'a> {
    Parameter::Value(Cow::Borrowed(key), Cow::Borrowed(val.as_ref()))
}

pub fn parameter<'a, T: ToString>(key: &'static str, val: &'a T) -> Parameter<'a> {
    Parameter::Value(Cow::Borrowed(key), Cow::Owned(val.to_string()))
}

pub fn execute_core<A, U, R>(auth: &A, method: Method, url: U, params: &[Parameter]) -> TwitterResult<R>
    where A: Authenticator, U: AsRef<str>, R: ::serde::de::Deserialize
{
    auth.request_twitter(method, url.as_ref(), params).and_then(|x| x.parse_to_object())
}
