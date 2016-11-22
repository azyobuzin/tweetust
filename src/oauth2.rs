//! The functions to get and invalidate your access token for
//! [Application-only authentication](https://dev.twitter.com/oauth/application-only).

use std::borrow::Cow;
use hyper::Post;
use hyper::header::Basic;
use url::{percent_encoding, Url};
use ::{ApplicationOnlyAuthenticator, TwitterResult};
use conn::{ParameterValue, RequestContent, request_twitter};

include!(concat!(env!("OUT_DIR"), "/oauth2_models.rs"));

impl TokenResponse {
    pub fn to_authenticator<'a>(self) -> ApplicationOnlyAuthenticator<'a> {
        ApplicationOnlyAuthenticator::new(self.access_token)
    }
}

#[derive(Clone, Debug)]
pub struct TokenRequestBuilder<'a> {
    consumer_key: Cow<'a, str>,
    consumer_secret: Cow<'a, str>,
    grant_type: Cow<'a, str>
}

impl<'a> TokenRequestBuilder<'a> {
    pub fn grant_type<T: Into<Cow<'a, str>>>(&mut self, val: T) -> &mut Self {
        self.grant_type = val.into();
        self
    }

    pub fn execute(&self) -> TwitterResult<TokenResponse> {
        let res = try!(request_twitter(
            Post,
            Url::parse("https://api.twitter.com/oauth2/token").unwrap(),
            RequestContent::WwwForm(Cow::Borrowed(&[
                (Cow::Borrowed("grant_type"), Cow::Borrowed(self.grant_type.as_ref()))
            ])),
            Basic {
                username: self.consumer_key.as_ref().to_owned(),
                password: Some(self.consumer_secret.as_ref().to_owned())
            }
        ));
        res.parse_to_object()
    }
}

pub fn token<'a, CK, CS>(consumer_key: CK, consumer_secret: CS) -> TokenRequestBuilder<'a>
    where CK: Into<Cow<'a, str>>, CS: Into<Cow<'a, str>>
{
    TokenRequestBuilder {
        consumer_key: consumer_key.into(),
        consumer_secret: consumer_secret.into(),
        grant_type: Cow::Borrowed("client_credentials")
    }
}

#[derive(Clone, Debug)]
pub struct InvalidateTokenRequestBuilder<'a> {
    consumer_key: Cow<'a, str>,
    consumer_secret: Cow<'a, str>,
    access_token: Cow<'a, str>
}

impl<'a> InvalidateTokenRequestBuilder<'a> {
    pub fn execute(&self) -> TwitterResult<InvalidateTokenResponse> {
        let access_token = percent_encoding::percent_decode(self.access_token.as_ref().as_bytes());
        let res = try!(request_twitter(
            Post,
            Url::parse("https://api.twitter.com/oauth2/invalidate_token").unwrap(),
            RequestContent::WwwForm(Cow::Borrowed(&[
                (Cow::Borrowed("access_token"), access_token.decode_utf8_lossy())
            ])),
            Basic {
                username: self.consumer_key.as_ref().to_owned(),
                password: Some(self.consumer_secret.as_ref().to_owned())
            }
        ));
        res.parse_to_object()
    }
}

pub fn invalidate_token<'a, CK, CS, T>(consumer_key: CK, consumer_secret: CS, access_token: T)
    -> InvalidateTokenRequestBuilder<'a>
    where CK: Into<Cow<'a, str>>, CS: Into<Cow<'a, str>>, T: Into<Cow<'a, str>>
{
    InvalidateTokenRequestBuilder {
        consumer_key: consumer_key.into(),
        consumer_secret: consumer_secret.into(),
        access_token: access_token.into()
    }
}
