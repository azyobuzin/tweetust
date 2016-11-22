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

pub mod application_only_authenticator;
pub mod oauth_authenticator;

pub enum RequestContent<'a> {
    None,
    WwwForm(Cow<'a, [(Cow<'a, str>, Cow<'a, str>)]>),
    MultipartFormData(&'a [(Cow<'a, str>, ParameterValue<'a>)]),
    Stream(StreamContent<'a>),
}

impl<'a> RequestContent<'a> {
    pub fn from_name_value_pairs(pairs: &'a [(Cow<'a, str>, ParameterValue<'a>)]) -> RequestContent<'a> {
        if pairs.is_empty() {
             RequestContent::None
        } else if pairs.iter()
            .any(|&(_, ref v)| match *v {
                ParameterValue::Text(_) => false,
                ParameterValue::File(_) => true,
            })
        {
            RequestContent::MultipartFormData(pairs)
        } else {
            RequestContent::WwwForm(Cow::Owned(
                pairs.iter()
                    .map(|&(ref key, ref val)| match *val {
                        ParameterValue::Text(ref val) => (Cow::Borrowed(key.as_ref()), Cow::Borrowed(val.as_ref())),
                        ParameterValue::File(_) => unreachable!(),
                    })
                    .collect()
            ))
        }
    }
}

pub enum ParameterValue<'a> {
    Text(Cow<'a, str>),
    File(&'a mut Read),
}

pub struct StreamContent<'a> {
    pub content_type: hyper::mime::Mime,
    pub content_length: Option<u64>,
    pub content: &'a mut Read,
}

pub struct Request<'a> {
    pub method: Method,
    pub url: Url,
    pub content: RequestContent<'a>,
}

impl<'a> Request<'a> {
    pub fn new(method: Method, url: &str, mut content: RequestContent<'a>) -> Result<Request<'a>, TwitterError> {
        let mut request_url = try!(Url::parse(url));

        match method  {
            Get | Delete | Head => {
                if let RequestContent::WwwForm(ref params) = content {
                    let mut query =  request_url.query_pairs_mut();
                    for &(ref key, ref val) in params.as_ref() {
                        query.append_pair(key.as_ref(), val.as_ref());
                    }
                } else {
                    return Err(TwitterError::InvalidRequest);
                }

                content = RequestContent::None;
            }
            _ => ()
        }

        Ok(Request { method: method, url: request_url, content: content })
    }
}

pub trait Authenticator {
    type Scheme: header::Scheme + Any;

    // TODO: remove
    fn send_request<'a>(&self, method: Method, url: &str, content: RequestContent<'a>)
        -> hyper::Result<Response>;

    // TODO: remove
    fn request_twitter<'a>(&self, method: Method, url: &str, content: RequestContent<'a>)
        -> TwitterResult<()>
    {
        read_to_twitter_result(try!(self.send_request(method, url, content)))
    }

    fn create_authorization_header(&self, request: &Request) -> Option<Self::Scheme>;
}

pub trait HttpHandler {
    fn send_request<A: Authenticator>(&self, request: Request, auth: &A) -> TwitterResult<()>;
}

#[derive(Debug, Default)]
pub struct DefaultHttpHandler {
    http_client: hyper::Client,
}

impl DefaultHttpHandler {
    pub fn new() -> DefaultHttpHandler {
        Default::default()
    }
}

impl HttpHandler for DefaultHttpHandler {
    fn send_request<A: Authenticator>(&self, request: Request, auth: &A) -> TwitterResult<()> {
        let scheme = auth.create_authorization_header(&request);
        let body;
        let mut req = self.http_client.request(request.method, request.url);

        if let Some(s) = scheme {
            req = req.header(header::Authorization(s));
        }

        match request.content {
            RequestContent::None => (),
            RequestContent::WwwForm(ref params) => {
                body = create_query(
                    params.as_ref().iter()
                        .map(|&(ref key, ref val)| (Cow::Borrowed(key.as_ref()), Cow::Borrowed(val.as_ref())))
                );
                req = req.body(&body[..])
                    .header(header::ContentType(mime::Mime(
                        mime::TopLevel::Application,
                        mime::SubLevel::WwwFormUrlEncoded,
                        Vec::new()
                    )));
            }
            RequestContent::MultipartFormData(_) => unimplemented!(),
            RequestContent::Stream(s) => {
                req =
                    req.body(
                        match s.content_length {
                            Some(len) => hyper::client::Body::SizedBody(s.content, len),
                            None => hyper::client::Body::ChunkedBody(s.content)
                        }
                    )
                    .header(header::ContentType(s.content_type));
            }
        }

        read_to_twitter_result(try!(req.send()))
    }
}

// TODO: remove
fn is_multipart<'a>(params: &[(Cow<'a, str>, ParameterValue<'a>)]) -> bool {
    params.iter().any(|x| match *x {
        (_, ParameterValue::Text(..)) => false,
        (_, ParameterValue::File(..)) => true
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

// TODO: remove
pub fn send_request<'a, S>(method: Method, url: &Url, content: RequestContent<'a>, authorization: S) -> hyper::Result<Response>
    where S: header::Scheme + Any
{
    let mut request_url = url.clone();

    let client = hyper::Client::new();
    let body;
    let mut req = client.request(method, request_url);

    match content {
        RequestContent::None => (),
        RequestContent::WwwForm(ref params) => {
            body = create_query(
                params.as_ref().iter()
                    .map(|&(ref key, ref val)| (Cow::Borrowed(key.as_ref()), Cow::Borrowed(val.as_ref())))
            );
            req = req.body(&body[..])
                .header(header::ContentType(mime::Mime(
                    mime::TopLevel::Application,
                    mime::SubLevel::WwwFormUrlEncoded,
                    Vec::new()
                )));
        }
        RequestContent::MultipartFormData(_) => unimplemented!(),
        RequestContent::Stream(s) => {
            req =
                req.body(
                    match s.content_length {
                        Some(len) => hyper::client::Body::SizedBody(s.content, len),
                        None => hyper::client::Body::ChunkedBody(s.content)
                    }
                )
                .header(header::ContentType(s.content_type));
        }
    }

    req.header(header::Authorization(authorization)).send()
}

include!(concat!(env!("OUT_DIR"), "/conn/internal_error_response.rs"));

/// Parses the rate limit headers and returns TwitterResult.
pub fn read_to_twitter_result(mut res: Response) -> TwitterResult<()> {
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
}

// TODO: remove
pub fn request_twitter<'a, S>(method: Method, url: Url, content: RequestContent<'a>, authorization: S) -> TwitterResult<()>
    where S: header::Scheme + Any
{
    read_to_twitter_result(try!(send_request(method, &url, content, authorization)))
}

pub fn parse_json<T: serde::de::Deserialize>(s: &str) -> serde_json::Result<T> {
    serde_json::from_str(s)
}
