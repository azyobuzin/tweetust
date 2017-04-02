use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use hyper::header;
use oauthcli::{OAuthAuthorizationHeader, OAuthAuthorizationHeaderBuilder, ParseOAuthAuthorizationHeaderError, SignatureMethod};
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
    type Scheme = OAuthAuthorizationScheme;

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

        let h = builder.finish_for_twitter();
        Some(OAuthAuthorizationScheme(h))
    }
}

/// hyper's Authorization header implementation
#[derive(Debug, Clone)]
pub struct OAuthAuthorizationScheme(pub OAuthAuthorizationHeader);

impl header::Scheme for OAuthAuthorizationScheme {
    fn scheme() -> Option<&'static str> {
        Some("OAuth")
    }

    fn fmt_scheme(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.auth_param())
    }
}

impl FromStr for OAuthAuthorizationScheme {
    type Err = ParseOAuthAuthorizationHeaderError;

    fn from_str(s: &str) -> Result<OAuthAuthorizationScheme, ParseOAuthAuthorizationHeaderError> {
        OAuthAuthorizationHeader::from_str(s)
            .map(OAuthAuthorizationScheme)
    }
}
