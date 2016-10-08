use std::borrow::Cow;
use hyper;
use hyper::client::Response;
use hyper::method::Method;
use oauthcli::{OAuthAuthorizationHeaderBuilder, SignatureMethod};
use url::Url;
use super::{Authenticator, is_multipart, Parameter, send_request};

/// OAuth 1.0 wrapper
#[derive(Clone, Debug)]
pub struct OAuthAuthenticator<'a> {
    pub consumer_key: Cow<'a, str>,
    pub consumer_secret: Cow<'a, str>,
    pub access_token: Cow<'a, str>,
    pub access_token_secret: Cow<'a, str>
}

impl<'a> OAuthAuthenticator<'a> {
    pub fn new<CK, CS, T, TS>(consumer_key: CK, consumer_secret: CS,
        access_token: T, access_token_secret: TS) -> OAuthAuthenticator<'a>
        where CK: Into<Cow<'a, str>>, CS: Into<Cow<'a, str>>, T: Into<Cow<'a, str>>, TS: Into<Cow<'a, str>>
    {
        OAuthAuthenticator {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
            access_token: access_token.into(),
            access_token_secret: access_token_secret.into()
        }
    }
}

impl<'a> Authenticator for OAuthAuthenticator<'a> {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter]) -> hyper::Result<Response> {
        match Url::parse(url) {
            Ok(ref u) => {
                let authorization = {
                    let mut builder = OAuthAuthorizationHeaderBuilder::new(
                        method.as_ref(),
                        u,
                        self.consumer_key.as_ref(),
                        self.consumer_secret.as_ref(),
                        SignatureMethod::HmacSha1
                    );
                    builder.token(self.access_token.as_ref(), self.access_token_secret.as_ref());

                    if !is_multipart(params) {
                        builder.request_parameters(params.iter().map(|x| match x {
                            &Parameter::Value(ref key, ref val) => (key.as_ref(), val.as_ref()),
                            _ => unreachable!()
                        }));
                    }

                    builder.finish_for_twitter()
                };

                send_request(method, u, params, authorization)
            },
            Err(e) => Err(hyper::Error::Uri(e))
        }
    }
}
