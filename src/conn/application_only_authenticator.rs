use std::borrow::Cow;
use hyper;
use hyper::client::Response;
use hyper::header::Bearer;
use hyper::method::Method;
use url::Url;
use super::{Authenticator, Parameter, send_request};

#[derive(Clone, Debug)]
pub struct ApplicationOnlyAuthenticator<'a> {
    pub access_token: Cow<'a, str>
}

impl<'a> ApplicationOnlyAuthenticator<'a> {
    pub fn new<T: Into<Cow<'a, str>>>(access_token: T) -> ApplicationOnlyAuthenticator<'a> {
        ApplicationOnlyAuthenticator { access_token: access_token.into() }
    }
}

impl<'a> Authenticator for ApplicationOnlyAuthenticator<'a> {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter]) -> hyper::Result<Response> {
        match Url::parse(url) {
            Ok(ref u) => send_request(method, u, params, Bearer { token: self.access_token.as_ref().to_owned() }),
            Err(e) => Err(hyper::Error::Uri(e))
        }
    }
}
