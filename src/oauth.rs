//! The functions to get your access token for
//! [3-legged OAuth](https://dev.twitter.com/oauth/3-legged) or
//! [PIN-based OAuth](https://dev.twitter.com/oauth/pin-based).

use std::borrow::Cow;
use hyper::{Post, Url};
use oauthcli::{OAuthAuthorizationHeaderBuilder, SignatureMethod};
use url::form_urlencoded;
use ::{OAuthAuthenticator, TwitterError, TwitterResult};
use conn::{ParameterValue, RequestContent, request_twitter};
use models::TwitterResponse;

#[derive(Clone, Debug)]
pub struct RequestTokenResponse {
    consumer_key: String,
    consumer_secret: String,
    pub oauth_token: String,
    pub oauth_token_secret: String,
    pub oauth_callback_confirmed: bool
}

impl RequestTokenResponse {
    pub fn access_token<'a, V: Into<Cow<'a, str>>>(&'a self, oauth_verifier: V) -> AccessTokenRequestBuilder<'a> {
        access_token(
            &self.consumer_key[..],
            &self.consumer_secret[..],
            &self.oauth_token[..],
            &self.oauth_token_secret[..],
            oauth_verifier
        )
    }
}

#[derive(Clone, Debug)]
pub struct RequestTokenRequestBuilder<'a> {
    consumer_key: Cow<'a, str>,
    consumer_secret: Cow<'a, str>,
    oauth_callback: Cow<'a, str>,
    x_auth_access_type: Option<Cow<'a, str>>
}

impl<'a> RequestTokenRequestBuilder<'a> {
    pub fn x_auth_access_type<T: Into<Cow<'a, str>>>(&mut self, val: T) -> &mut Self {
        self.x_auth_access_type = Some(val.into());
        self
    }

    pub fn execute(&self) -> TwitterResult<RequestTokenResponse> {
        let request_token_url = Url::parse("https://api.twitter.com/oauth/request_token").unwrap();
        let mut params = Vec::new();
        if let Some(ref x) = self.x_auth_access_type {
            params.push((Cow::Borrowed("x_auth_access_type"), Cow::Borrowed(x.as_ref())))
        }

        let authorization =
            OAuthAuthorizationHeaderBuilder::new(
                "POST",
                &request_token_url,
                self.consumer_key.as_ref(),
                self.consumer_secret.as_ref(),
                SignatureMethod::HmacSha1
            )
            .callback(self.oauth_callback.as_ref())
            .request_parameters(params.iter()
                .map(|&(ref key, ref val)| (key.as_ref(), val.as_ref())))
            .finish_for_twitter();

        let res = try!(request_twitter(
            Post, request_token_url, RequestContent::WwwForm(Cow::Owned(params)), authorization));

        let (oauth_token, oauth_token_secret, oauth_callback_confirmed) = {
            let v = form_urlencoded::parse(res.raw_response.as_bytes()).collect::<Vec<_>>();
            (
                v.iter().find(|x| x.0 == "oauth_token").map(|&(_, ref val)| val.as_ref().to_owned()),
                v.iter().find(|x| x.0 == "oauth_token_secret").map(|&(_, ref val)| val.as_ref().to_owned()),
                v.iter().find(|x| x.0 == "oauth_callback_confirmed").and_then(|&(_, ref val)| val.parse().ok())
            )
        };

        match (oauth_token, oauth_token_secret) {
            (Some(oauth_token), Some(oauth_token_secret)) =>
                Ok(TwitterResponse {
                    object: RequestTokenResponse {
                        consumer_key: self.consumer_key.as_ref().to_owned(),
                        consumer_secret: self.consumer_secret.as_ref().to_owned(),
                        oauth_token: oauth_token,
                        oauth_token_secret: oauth_token_secret,
                        oauth_callback_confirmed: oauth_callback_confirmed.unwrap_or(false)
                    },
                    raw_response: res.raw_response,
                    rate_limit: res.rate_limit
                }),
            _ => Err(TwitterError::ParseError(res))
        }
    }
}

pub fn request_token<'a, CK, CS, CB>(consumer_key: CK, consumer_secret: CS, oauth_callback: CB) -> RequestTokenRequestBuilder<'a>
    where CK: Into<Cow<'a, str>>, CS: Into<Cow<'a, str>>, CB: Into<Cow<'a, str>>
{
    RequestTokenRequestBuilder {
        consumer_key: consumer_key.into(),
        consumer_secret: consumer_secret.into(),
        oauth_callback: oauth_callback.into(),
        x_auth_access_type: None
    }
}

#[derive(Clone, Debug)]
pub struct AccessTokenResponse {
    consumer_key: String,
    consumer_secret: String,
    pub oauth_token: String,
    pub oauth_token_secret: String,
    pub user_id: i64,
    pub screen_name: String
}

impl AccessTokenResponse {
    pub fn to_authenticator<'a>(self) -> OAuthAuthenticator<'a> {
        OAuthAuthenticator::new(
            self.consumer_key,
            self.consumer_secret,
            self.oauth_token,
            self.oauth_token_secret
        )
    }
}

#[derive(Clone, Debug)]
pub struct AccessTokenRequestBuilder<'a> {
    consumer_key: Cow<'a, str>,
    consumer_secret: Cow<'a, str>,
    oauth_token: Cow<'a, str>,
    oauth_token_secret: Cow<'a, str>,
    oauth_verifier: Cow<'a, str>
}

impl<'a> AccessTokenRequestBuilder<'a> {
    pub fn execute(&self) -> TwitterResult<AccessTokenResponse> {
        let access_token_url = Url::parse("https://api.twitter.com/oauth/access_token").unwrap();

        let authorization =
            OAuthAuthorizationHeaderBuilder::new(
                "POST",
                &access_token_url,
                self.consumer_key.as_ref(),
                self.consumer_secret.as_ref(),
                SignatureMethod::HmacSha1
            )
            .token(self.oauth_token.as_ref(), self.oauth_token_secret.as_ref())
            .verifier(self.oauth_verifier.as_ref())
            .finish_for_twitter();

        let res = try!(request_twitter(
            Post, access_token_url, RequestContent::None, authorization));

        let t = {
            let v = form_urlencoded::parse(res.raw_response.as_bytes()).collect::<Vec<_>>();
            (
                v.iter().find(|x| x.0 == "oauth_token").map(|&(_, ref val)| val.as_ref().to_owned()),
                v.iter().find(|x| x.0 == "oauth_token_secret").map(|&(_, ref val)| val.as_ref().to_owned()),
                v.iter().find(|x| x.0 == "user_id").and_then(|&(_, ref val)| val.parse().ok()),
                v.iter().find(|x| x.0 == "screen_name").map(|&(_, ref val)| val.as_ref().to_owned())
            )
        };

        match t {
            (Some(oauth_token), Some(oauth_token_secret), Some(user_id), Some(screen_name)) =>
                Ok(TwitterResponse {
                    object: AccessTokenResponse {
                        consumer_key: self.consumer_key.as_ref().to_owned(),
                        consumer_secret: self.consumer_secret.as_ref().to_owned(),
                        oauth_token: oauth_token,
                        oauth_token_secret: oauth_token_secret,
                        user_id: user_id,
                        screen_name: screen_name
                    },
                    raw_response: res.raw_response,
                    rate_limit: res.rate_limit
                }),
            _ => Err(TwitterError::ParseError(res))
        }
    }
}

pub fn access_token<'a, CK, CS, T, TS, V>(consumer_key: CK, consumer_secret: CS,
    oauth_token: T, oauth_token_secret: TS, oauth_verifier: V)
    -> AccessTokenRequestBuilder<'a>
    where CK: Into<Cow<'a, str>>, CS: Into<Cow<'a, str>>, T: Into<Cow<'a, str>>, TS: Into<Cow<'a, str>>, V: Into<Cow<'a, str>>
{
    AccessTokenRequestBuilder {
        consumer_key: consumer_key.into(),
        consumer_secret: consumer_secret.into(),
        oauth_token: oauth_token.into(),
        oauth_token_secret: oauth_token_secret.into(),
        oauth_verifier: oauth_verifier.into()
    }
}
