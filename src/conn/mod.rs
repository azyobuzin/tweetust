//! The low-level functions for connecting to Twitter with any authorization.
//! Usually, you will not use this module.

use std::fmt::{self, Write};
use std::io::Read;
use std::rc::Rc;
use std::string::ToString;
use hyper::{self, header, mime, Get, Delete, Head};
use hyper::client::Response;
use hyper::method::Method;
use hyper::status::StatusClass;
use oauthcli;
use rustc_serialize::{json, Decodable};
use url::{percent_encoding, Url};
use ::{TwitterError, TwitterResult};
use models::*;
use models::error::{Error, ErrorResponse};

pub mod application_only_authenticator;
pub mod oauth_authenticator;

pub enum Parameter<'a> {
    Value(&'a str, String),
    File(&'a str, &'a mut (Read + 'a))
}

impl<'a> Parameter<'a> {
    pub fn key_value<T: ToString>(key: &'a str, value: T) -> Parameter<'a> {
        Parameter::Value(key, value.to_string())
    }

    pub fn from_vec<T: fmt::Display>(key: &'a str, value: Vec<T>)  -> Parameter<'a> {
        let mut val = String::new();
        for elm in value.into_iter() {
            if val.len() > 0 {
                val.push(',');
            }
            write!(&mut val, "{}", elm).unwrap();
        }

        Parameter::Value(key, val)
    }
}

pub trait Authenticator: Clone {
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
    where I: Iterator<Item=(&'a str, &'a str)>
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
            percent_encoding::utf8_percent_encode(key, es),
            percent_encoding::utf8_percent_encode(val, es)
        ).unwrap();
    }
    s
}

pub fn send_request(method: Method, url: Url, params: &[Parameter],
    authorization: String) -> hyper::Result<Response>
{
    let mut request_url = url.clone();

    let has_body = match method {
        Get | Delete | Head => false,
        _ => true
    };

    if !has_body {
        let query_pairs = url.query_pairs().collect::<Vec<_>>();
        let query = create_query(
            query_pairs.iter().map(|x| (&x.0[..], &x.1[..])).chain(
                params.iter().map(|x| match x {
                    &Parameter::Value(key, ref val) => (key, &val[..]),
                    _ => panic!("the request whose method is GET, DELETE or HEAD has Parameter::File")
                })
            )
        );
        request_url.set_query(Some(&query[..]));
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
                    &Parameter::Value(key, ref val) => (key, &val[..]),
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

    let req = req.header(header::Authorization(authorization));
    req.send()
}

#[derive(Debug, RustcDecodable)]
struct InternalErrorResponse {
    errors: Option<Vec<Error>>,
    error: Option<Vec<Error>>
}

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
                        object: (), raw_response: Rc::new(body), rate_limit: rate_limit
                    }),
                    _ => {
                        // Error response
                        let dec: json::DecodeResult<InternalErrorResponse> = json::decode(&body[..]);
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

pub fn request_twitter(method: Method, url: Url, params: &[Parameter],
    authorization: String) -> TwitterResult<()>
{
    read_to_twitter_result(send_request(method, url, params, authorization))
}

/// Parse the JSON string to T with rustc-serialize.
/// As a stopgap measure, this function renames `type` to `type_`.
pub fn parse_json<T: Decodable>(s: &str) -> json::DecodeResult<T> {
    json::decode(&s.replace("\"type\":", "\"type_\":")[..])
}
