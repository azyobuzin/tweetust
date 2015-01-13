use hyper::{header, HttpError, HttpResult};
use hyper::client::Client;
use hyper::client::response::Response;
use hyper::method::Method;
use hyper::mime;
use oauthcli::{self, SignatureMethod};
use url::{form_urlencoded, Url};

pub enum Parameter<'a> {
    Value(&'a str, &'a str),
    File(&'a str, &'a [u8])
}

pub trait Authenticator {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter])
        -> HttpResult<Response>;
}

fn is_multipart(params: &[Parameter]) -> bool {
    params.iter().any(|x| match x {
        &Parameter::Value(_, _) => false,
        &Parameter::File(_, _) => true
    })
}

fn send_request_internal(method: Method, mut url: Url, params: &[Parameter],
    authorization: String) -> HttpResult<Response>
{
    let has_body = match method {
        Method::Get | Method::Delete | Method::Head => false,
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
                    &Parameter::Value(key, val) => (key, val),
                    _ => panic!("The request whose method is GET, DELETE or HEAD has Parameter::File")
                })
            )
        );
    }

    let mut client = Client::new();
    let mut req = client.request(method, url);

    let body;

    if has_body {
        if is_multipart(params) {
            unimplemented!();
        } else {
            body = form_urlencoded::serialize(
                params.iter().map(|x| match x {
                    &Parameter::Value(ref key, val) => (key.as_slice(), val),
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

#[derive(Clone, Show)]
pub struct OAuthAuthenticator {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_token_secret: String
}

impl OAuthAuthenticator {
    pub fn new(consumer_key: &str, consumer_secret: &str,
        access_token: &str, access_token_secret: &str) -> OAuthAuthenticator
    {
        OAuthAuthenticator {
            consumer_key: consumer_key.to_string(),
            consumer_secret: consumer_secret.to_string(),
            access_token: access_token.to_string(),
            access_token_secret: access_token_secret.to_string()
        }
    }
}

impl Authenticator for OAuthAuthenticator {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter]) -> HttpResult<Response> {
        match Url::parse(url) {
            Ok(u) => {
                let multipart = is_multipart(params);
                let mut auth_params = Vec::<(String, String)>::new();
                if !multipart {
                    auth_params.extend(params.iter().map(|x| match x {
                        &Parameter::Value(key, val) => (key.to_string(), val.to_string()),
                        _ => unreachable!()
                    }));
                }

                let authorization = oauthcli::authorization_header(
                    method.to_string().as_slice(),
                    u.clone(),
                    None,
                    self.consumer_key.as_slice(),
                    self.consumer_secret.as_slice(),
                    Some(self.access_token.as_slice()),
                    Some(self.access_token_secret.as_slice()),
                    SignatureMethod::HmacSha1,
                    oauthcli::timestamp(),
                    oauthcli::nonce(),
                    None,
                    None,
                    auth_params.iter()
                );
                send_request_internal(method, u.clone(), params, authorization)
            },
            Err(e) => Err(HttpError::HttpUriError(e))
        }
    }
}
