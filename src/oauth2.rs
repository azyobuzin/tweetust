//! The functions to get and invalidate your access token for
//! [Application-only authentication](https://dev.twitter.com/oauth/application-only).

use std::borrow::Cow;
use hyper::Post;
use hyper::header::Basic;
use url::percent_encoding;
use ::{ApplicationOnlyAuthenticator, TwitterResult};
use conn::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token_type: String,
    pub access_token: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvalidateTokenResponse {
    pub access_token: String
}


impl TokenResponse {
    pub fn to_authenticator<'a>(self) -> ApplicationOnlyAuthenticator<'a> {
        ApplicationOnlyAuthenticator::new(self.access_token)
    }
}

struct CkCsBasicAuthenticator<'a> {
    consumer_key: &'a str,
    consumer_secret: &'a str,
}

impl<'a> CkCsBasicAuthenticator<'a> {
    fn new(consumer_key: &'a str, consumer_secret: &'a str) -> CkCsBasicAuthenticator<'a> {
        CkCsBasicAuthenticator {
            consumer_key: consumer_key,
            consumer_secret: consumer_secret,
        }
    }
}

impl<'a> Authenticator for CkCsBasicAuthenticator<'a> {
    type Scheme = Basic;

    fn create_authorization_header(&self, _: &Request) -> Option<Self::Scheme> {
        Some(Basic {
            username: self.consumer_key.to_owned(),
            password: Some(self.consumer_secret.to_owned())
        })
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

    pub fn execute<H: HttpHandler>(&self, handler: &H) -> TwitterResult<TokenResponse> {
        let params = [(Cow::Borrowed("grant_type"), Cow::Borrowed(self.grant_type.as_ref()))];

        let req = try!(Request::new(
            Post,
            "https://api.twitter.com/oauth2/token",
            RequestContent::WwwForm(Cow::Borrowed(&params))
        ));

        try!(handler.send_request(
            req,
            &CkCsBasicAuthenticator::new(&self.consumer_key, &self.consumer_secret)
        )).parse_to_object()
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
    pub fn execute<H: HttpHandler>(&self, handler: &H) -> TwitterResult<InvalidateTokenResponse> {
        let access_token = percent_encoding::percent_decode(self.access_token.as_ref().as_bytes());
        let params = [(Cow::Borrowed("access_token"), access_token.decode_utf8_lossy())];

        let req = try!(Request::new(
            Post,
            "https://api.twitter.com/oauth2/invalidate_token",
            RequestContent::WwwForm(Cow::Borrowed(&params))
        ));

        try!(handler.send_request(
            req,
            &CkCsBasicAuthenticator::new(&self.consumer_key, &self.consumer_secret)
        )).parse_to_object()
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
