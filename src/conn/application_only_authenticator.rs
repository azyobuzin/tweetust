use hyper;
use hyper::client::Response;
use hyper::method::Method;
use url::Url;
use super::{Authenticator, Parameter, send_request};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationOnlyAuthenticator(pub String);

impl Authenticator for ApplicationOnlyAuthenticator {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter]) -> hyper::Result<Response> {
        match Url::parse(url) {
            Ok(u) => send_request(method, u, params, format!("Bearer {}", self.0)),
            Err(e) => Err(hyper::Error::Uri(e))
        }
    }
}
