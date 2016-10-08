//! The low-level functions for connecting to Twitter with any authorization.
//! Usually, you will not use this module.

use std::any::Any;
use std::borrow::Cow;
use std::fmt::Write;
use std::io::Read;
use hyper::{self, header, mime, Get, Delete, Head};
use hyper::client::Response;
use hyper::method::Method;
use hyper::status::StatusClass;
use oauthcli;
use url::{percent_encoding, Url};
use serde;
use serde_json;
use ::{TwitterError, TwitterResult};
use models::*;
use models::error::{Error, ErrorResponse};

pub mod application_only_authenticator;
pub mod oauth_authenticator;

pub enum Parameter<'a> {
    Value(Cow<'a, str>, Cow<'a, str>),
    File(Cow<'a, str>, &'a mut (Read + 'a))
}

pub trait Authenticator {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter])
        -> hyper::Result<Response>;

    fn request_twitter(&self, method: Method, url: &str, params: &[Parameter])
        -> TwitterResult<()>
    {
        read_to_twitter_result(self.send_request(method, url, params))
    }
}

fn is_multipart(params: &[Parameter]) -> bool {
    params.iter().any(|x| match *x {
        Parameter::Value(..) => false,
        Parameter::File(..) => true
    })
}

fn create_query<'a, I>(pairs: I) -> String
    where I: Iterator<Item=(Cow<'a, str>, Cow<'a, str>)>
{
    let es = oauthcli::OAUTH_ENCODE_SET;
    let mut s = String::new();
    for (key, val) in pairs {
        if s.len() > 0 {
            s.push('&');
        }
        write!(
            &mut s,
            "{}={}",
            percent_encoding::utf8_percent_encode(&key, es),
            percent_encoding::utf8_percent_encode(&val, es)
        ).unwrap();
    }
    s
}

pub fn send_request<S>(method: Method, url: &Url, params: &[Parameter], authorization: S) -> hyper::Result<Response>
    where S: header::Scheme + Any
{
    // TODO: refactor

    let mut request_url = url.clone();

    let has_body = match method {
        Get | Delete | Head => false,
        _ => true
    };

    if !has_body {
        let query = create_query(
            url.query_pairs().chain(
                params.iter().map(|x| match x {
                    &Parameter::Value(ref key, ref val) => (Cow::Borrowed(key.as_ref()), Cow::Borrowed(val.as_ref())),
                    _ => panic!("the request whose method is GET, DELETE or HEAD has Parameter::File")
                })
            )
        );
        request_url.set_query(Some(&query));
    }

    let client = hyper::Client::new();
    let body;
    let mut req = client.request(method, request_url);

    if has_body {
        if is_multipart(params) {
            unimplemented!();
        } else {
            body = create_query(
                params.iter().map(|x| match x {
                    &Parameter::Value(ref key, ref val) => (Cow::Borrowed(key.as_ref()), Cow::Borrowed(val.as_ref())),
                    _ => unreachable!()
                })
            );
            req = req.body(&body[..])
                .header(header::ContentType(mime::Mime(
                    mime::TopLevel::Application,
                    mime::SubLevel::WwwFormUrlEncoded,
                    Vec::new()
                )));
        }
    }

    req.header(header::Authorization(authorization)).send()
}

include!(concat!(env!("OUT_DIR"), "/conn/internal_error_response.rs"));

fn read_to_twitter_result(source: hyper::Result<Response>) -> TwitterResult<()> {
    match source {
        Ok(mut res) => {
            // Parse headers
            let limit = res.headers.get_raw("X-Rate-Limit-Limit")
                .and_then(|x| x.first())
                .and_then(|x| (&String::from_utf8_lossy(&x[..])[..]).parse().ok());
            let remaining = res.headers.get_raw("X-Rate-Limit-Remaining")
                .and_then(|x| x.first())
                .and_then(|x| (&String::from_utf8_lossy(&x[..])[..]).parse().ok());
            let reset = res.headers.get_raw("X-Rate-Limit-Reset")
                .and_then(|x| x.first())
                .and_then(|x| (&String::from_utf8_lossy(&x[..])[..]).parse().ok());
            let rate_limit = limit.and(remaining).and(reset)
                .map(|_| RateLimitStatus {
                    limit: limit.unwrap(),
                    remaining: remaining.unwrap(),
                    reset: reset.unwrap()
                });

            let mut body = String::new();
            match res.read_to_string(&mut body) {
                Ok(_) => match res.status.class() {
                    // 2xx
                    StatusClass::Success => Ok(TwitterResponse {
                        object: (), raw_response: body, rate_limit: rate_limit
                    }),
                    _ => {
                        // Error response
                        let dec = parse_json::<InternalErrorResponse>(&body);
                        let errors = dec.ok().and_then(|x| x.errors.or(x.error));
                        Err(TwitterError::ErrorResponse(ErrorResponse {
                            status: res.status,
                            errors: errors,
                            raw_response: body,
                            rate_limit: rate_limit
                        }))
                    }
                },
                Err(e) => Err(TwitterError::HttpError(hyper::Error::Io(e)))
            }
        },
        Err(e) => Err(TwitterError::HttpError(e))
    }
}

pub fn request_twitter<S>(method: Method, url: Url, params: &[Parameter], authorization: S) -> TwitterResult<()>
    where S: header::Scheme + Any
{
    read_to_twitter_result(send_request(method, &url, params, authorization))
}

pub fn parse_json<T: serde::de::Deserialize>(s: &str) -> serde_json::Result<T> {
    serde_json::from_str(s)
}
