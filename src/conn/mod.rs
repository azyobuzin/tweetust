//! The low-level functions for connecting to Twitter with any authorization.
//! Usually, you will not use this module.

use std::any::Any;
use std::borrow::Cow;
use std::io::{self, Read};
use hyper::{self, header, mime, Get, Delete, Head};
use hyper::client::Response;
use hyper::method::Method;
use hyper::status::StatusClass;
use oauthcli;
use url::{percent_encoding, Url};
use ::{parse_json, TwitterError};
use models::*;

#[cfg(feature = "hyper-native-tls")]
use hyper_native_tls::NativeTlsClient;
#[cfg(feature = "hyper-native-tls")]
use hyper_native_tls::native_tls;

pub mod application_only_authenticator;
pub mod oauth_authenticator;
mod hyper_multipart;

pub enum RequestContent<'a> {
    None,
    WwwForm(Cow<'a, [(Cow<'a, str>, Cow<'a, str>)]>),
    MultipartFormData(Vec<(Cow<'a, str>, ParameterValue<'a>)>),
    Stream(StreamContent<'a>),
}

impl<'a> RequestContent<'a> {
    pub fn from_name_value_pairs(pairs: Vec<(Cow<'a, str>, ParameterValue<'a>)>) -> RequestContent<'a> {
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
                pairs.into_iter()
                    .map(|(key, val)| match val {
                        ParameterValue::Text(val) => (key, val),
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
                match content {
                    RequestContent::WwwForm(ref params) => {
                        let mut query =  request_url.query_pairs_mut();
                        for &(ref key, ref val) in params.as_ref() {
                            query.append_pair(key.as_ref(), val.as_ref());
                        }
                    }
                    RequestContent::None => (),
                    _ => return Err(TwitterError::InvalidRequest)
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
    fn create_authorization_header(&self, request: &Request) -> Option<Self::Scheme>;
}

pub trait HttpHandler {
    /// Sends a request and parses the response.
    fn send_request<A: Authenticator>(&self, request: Request, auth: &A) -> Result<RawResponse, TwitterError>;
}

impl<'a, H: HttpHandler + ?Sized> HttpHandler for &'a H {
    fn send_request<A: Authenticator>(&self, request: Request, auth: &A) -> Result<RawResponse, TwitterError> {
        (**self).send_request(request, auth)
    }
}

pub type DefaultHttpsConnector = hyper::client::Pool<hyper::net::HttpsConnector<NativeTlsClient>>;

pub struct DefaultHttpHandler<C: hyper::net::NetworkConnector> {
    connector: C,
}

impl<C: hyper::net::NetworkConnector> DefaultHttpHandler<C> {
    pub fn new(connector: C) -> DefaultHttpHandler<C> {
        DefaultHttpHandler {
            connector: connector,
        }
    }
}

#[cfg(feature = "hyper-native-tls")]
impl DefaultHttpHandler<DefaultHttpsConnector> {
    pub fn with_https_connector() -> native_tls::Result<Self> {
        NativeTlsClient::new()
            .map(|x| DefaultHttpHandler::new(
                hyper::client::Pool::with_connector(
                    Default::default(),
                    hyper::net::HttpsConnector::new(x)
                )
            ))
    }
}

impl<C: hyper::net::NetworkConnector> HttpHandler for DefaultHttpHandler<C> {
    fn send_request<A: Authenticator>(&self, request: Request, auth: &A) -> Result<RawResponse, TwitterError> {
        use std::io::Write;

        let scheme = auth.create_authorization_header(&request);
        let body;
        let mut req = hyper::client::Request::with_connector(request.method, request.url, &self.connector)?;

        if let Some(s) = scheme {
            req.headers_mut().set(header::Authorization(s));
        }

        let res = match request.content {
            RequestContent::None => req.start()?.send(),
            RequestContent::WwwForm(ref params) => {
                body = create_query(
                    params.as_ref().iter()
                        .map(|&(ref key, ref val)| (Cow::Borrowed(key.as_ref()), Cow::Borrowed(val.as_ref())))
                );
                {
                    let mut headers = req.headers_mut();
                    headers.set(header::ContentLength(body.len() as u64));
                    headers.set(header::ContentType(mime::Mime(
                        mime::TopLevel::Application,
                        mime::SubLevel::WwwFormUrlEncoded,
                        Vec::new()
                    )));
                }
                let mut req = req.start()?;
                req.write_all(body.as_bytes())?;
                req.send()
            }
            RequestContent::MultipartFormData(params) => {
                let mut multipart = hyper_multipart::create_multipart_client(req)?;
                for (key, val) in params {
                    match val {
                        ParameterValue::Text(x) => multipart.write_text(key, x),
                        ParameterValue::File(mut x) => multipart.write_stream(key, &mut x, Some("file"), None),
                    }?;
                }
                multipart.send()
            }
            RequestContent::Stream(s) => {
                {
                    let mut headers = req.headers_mut();
                    headers.set(header::ContentType(s.content_type));
                    if let Some(len) = s.content_length {
                        headers.set(header::ContentLength(len));
                    }
                }
                let mut req = req.start()?;
                io::copy(s.content, &mut req)?;
                req.send()
            }
        };

        read_to_twitter_result(res?)
    }
}

fn create_query<'a, I>(pairs: I) -> String
    where I: Iterator<Item=(Cow<'a, str>, Cow<'a, str>)>
{
    use std::fmt::Write;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InternalErrorResponse {
    errors: Option<Vec<Error>>,
    error: Option<Vec<Error>>
}

/// Parses the rate limit headers and returns.
pub fn read_to_twitter_result(mut res: Response) -> Result<RawResponse, TwitterError> {
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
    try!(res.read_to_string(&mut body));

    match res.status.class() {
        // 2xx
        StatusClass::Success => Ok(RawResponse {
            raw_response: body,
            rate_limit: rate_limit,
        }),
        _ => {
            // Error response
            let dec = parse_json::<InternalErrorResponse>(&body);
            let errors = dec.ok().and_then(|x| x.errors.or(x.error));
            Err(TwitterError::ErrorResponse(ErrorResponse {
                status: res.status,
                errors: errors,
                raw_response: body,
                rate_limit: rate_limit,
            }))
        }
    }
}
