use std::borrow::Cow;
use oauthcli::{OAuthAuthorizationHeader, OAuthAuthorizationHeaderBuilder, SignatureMethod};
use super::*;

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
    type Scheme = OAuthAuthorizationHeader;

    fn create_authorization_header(&self, request: &Request) -> Option<Self::Scheme> {
        let mut builder = OAuthAuthorizationHeaderBuilder::new(
            request.method.as_ref(),
            &request.url,
            self.consumer_key.as_ref(),
            self.consumer_secret.as_ref(),
            SignatureMethod::HmacSha1
        );
        builder.token(self.access_token.as_ref(), self.access_token_secret.as_ref());

        if let RequestContent::WwwForm(ref params) = request.content {
            builder.request_parameters(
                params.as_ref().iter()
                    .map(|&(ref key, ref val)| (key.as_ref(), val.as_ref()))
            );
        }

        Some(builder.finish_for_twitter())
    }
}
