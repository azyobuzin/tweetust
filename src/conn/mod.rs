//! The low-level functions for connecting to Twitter with any authorization.
//! Usually, you will not use this module.

use std::string::ToString;
use hyper;
use hyper::{header, mime, HttpError, HttpResult, Get, Delete, Head};
use hyper::client::Response;
use hyper::method::Method;
use hyper::status::StatusClass;
use rustc_serialize::{json, Decodable};
use url::{form_urlencoded, Url};
use ::{TwitterError, TwitterResult};
use models::*;
use models::error::{Error, ErrorResponse};

pub mod application_only_authenticator;
pub mod oauth_authenticator;

pub enum Parameter<'a> {
    Value(&'a str, String),
    File(&'a str, &'a mut (Reader + 'a))
}

pub trait Authenticator: Clone {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter])
        -> HttpResult<Response>;
}

fn is_multipart(params: &[Parameter]) -> bool {
    params.iter().any(|x| match *x {
        Parameter::Value(_, _) => false,
        Parameter::File(_, _) => true
    })
}

pub fn send_request(method: Method, mut url: Url, params: &[Parameter],
    authorization: String) -> HttpResult<Response>
{
    let has_body = match method {
        Get | Delete | Head => false,
        _ => true
    };

    if !has_body {
        let query = match url.query_pairs() {
            Some(x) => x,
            None => Vec::new()
        };
        url.set_query_from_pairs(
            query.iter().map(|x| (x.0.as_slice(), x.1.as_slice())).chain(
                params.iter().map(|x| match x {
                    &Parameter::Value(key, ref val) => (key, val.as_slice()),
                    _ => panic!("the request whose method is GET, DELETE or HEAD has Parameter::File")
                })
            )
        );
    }

    let mut client = hyper::Client::new();
    let mut req = client.request(method, url);

    let body;

    if has_body {
        if is_multipart(params) {
            unimplemented!();
        } else {
            body = form_urlencoded::serialize(
                params.iter().map(|x| match x {
                    &Parameter::Value(key, ref val) => (key, val.as_slice()),
                    _ => unreachable!()
                })
            );
            req = req.body(body.as_slice())
                .header(header::ContentType(mime::Mime(
                    mime::TopLevel::Application,
                    mime::SubLevel::WwwFormUrlEncoded,
                    Vec::new()
                )));
        }
    }

    req.header(header::Authorization(authorization)).send()
}

#[derive(RustcDecodable)]
struct InternalErrorResponse {
    errors: Option<Vec<Error>>,
    error: Option<Vec<Error>>
}

pub fn read_to_twitter_result(source: HttpResult<Response>) -> TwitterResult<()> {
    match source {
        Ok(mut res) => {
            // Parse headers
            let limit = res.headers.get_raw("X-Rate-Limit-Limit")
                .and_then(|x| x.first())
                .and_then(|x| String::from_utf8_lossy(x.as_slice()).as_slice().parse());
            let remaining = res.headers.get_raw("X-Rate-Limit-Remaining")
                .and_then(|x| x.first())
                .and_then(|x| String::from_utf8_lossy(x.as_slice()).as_slice().parse());
            let reset = res.headers.get_raw("X-Rate-Limit-Reset")
                .and_then(|x| x.first())
                .and_then(|x| String::from_utf8_lossy(x.as_slice()).as_slice().parse());
            let rate_limit = limit.and(remaining).and(reset)
                .map(|_| RateLimitStatus {
                    limit: limit.unwrap(),
                    remaining: remaining.unwrap(),
                    reset: reset.unwrap()
                });

            match res.read_to_string() {
                Ok(body) => match res.status.class() {
                    // 2xx
                    StatusClass::Success => Ok(TwitterResponse {
                        object: (), raw_response: body, rate_limit: rate_limit
                    }),
                    _ => {
                        // Error response
                        let dec: json::DecodeResult<InternalErrorResponse> = json::decode(body.as_slice());
                        let errors = dec.ok().and_then(|x| x.errors.or(x.error));
                        Err(TwitterError::ErrorResponse(ErrorResponse {
                            status: res.status,
                            errors: errors,
                            raw_response: body,
                            rate_limit: rate_limit
                        }))
                    }
                },
                Err(e) => Err(TwitterError::HttpError(HttpError::HttpIoError(e)))
            }
        },
        Err(e) => Err(TwitterError::HttpError(e))
    }
}

pub trait ToParameter {
    fn to_parameter<'a>(self, key: &'a str) -> Parameter<'a>;
}

impl<T: ToString> ToParameter for T {
    fn to_parameter<'a>(self, key: &'a str) -> Parameter<'a> {
        Parameter::Value(key, self.to_string())
    }
}

/// Parse the JSON string to T with rustc-serialize.
/// As a stopgap measure, this function renames `type` to `type_`.
pub fn parse_json<T: Decodable>(s: &str) -> json::DecodeResult<T> {
    json::decode(s.replace("\"type\":", "\"type_\":").as_slice())
}
