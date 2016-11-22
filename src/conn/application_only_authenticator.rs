use std::borrow::Cow;
use hyper::header::Bearer;
use super::*;

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
    type Scheme = Bearer;

    fn create_authorization_header(&self, _: &Request) -> Option<Self::Scheme> {
        Some(Bearer { token: self.access_token.as_ref().to_owned() })
    }
}
