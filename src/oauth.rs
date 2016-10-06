//! The functions to get your access token for
//! [3-legged OAuth](https://dev.twitter.com/oauth/3-legged) or
//! [PIN-based OAuth](https://dev.twitter.com/oauth/pin-based).

use std::borrow::Cow;
use hyper::{Post, Url};
use oauthcli::{self, SignatureMethod};
use url::form_urlencoded;
use ::{OAuthAuthenticator, TwitterError, TwitterResult};
use conn::request_twitter;
use conn::Parameter::Value;

#[derive(Clone, Debug)]
pub struct RequestTokenResponse {
    consumer_key: String,
    consumer_secret: String,
    pub oauth_token: String,
    pub oauth_token_secret: String,
    pub oauth_callback_confirmed: bool
}

impl RequestTokenResponse {
    pub fn access_token(&self, oauth_verifier: &str) -> AccessTokenRequestBuilder {
        access_token(
            &self.consumer_key[..],
            &self.consumer_secret[..],
            &self.oauth_token[..],
            &self.oauth_token_secret[..],
            oauth_verifier
        )
    }
}

// TODO: Cow
#[derive(Clone, Debug)]
pub struct RequestTokenRequestBuilder {
    consumer_key: String,
    consumer_secret: String,
    oauth_callback: String,
    x_auth_access_type: Option<String>
}

impl RequestTokenRequestBuilder {
    pub fn x_auth_access_type(mut self, val: &str) -> RequestTokenRequestBuilder {
        self.x_auth_access_type = Some(val.to_string());
        self
    }

    pub fn execute(&self) -> TwitterResult<RequestTokenResponse> {
        let request_token_url = Url::parse("https://api.twitter.com/oauth/request_token").unwrap();
        let mut params = Vec::new();
        if let Some(ref x) = self.x_auth_access_type {
            // TODO: RequestTokenRequestBuilderのCow化
            params.push(Value(Cow::Borrowed("x_auth_access_type"), Cow::Owned(x.clone())))
        }
        // TODO: Builder使う
        let authorization = oauthcli::authorization_header(
            "POST",
            request_token_url.clone(),
            None,
            &self.consumer_key[..],
            &self.consumer_secret[..],
            None,
            None,
            SignatureMethod::HmacSha1,
            &oauthcli::timestamp()[..],
            &oauthcli::nonce()[..],
            Some(&self.oauth_callback[..]),
            None,
            params.iter().map(|x| match x {
                &Value(ref key, ref val) => (key.to_string(), val.to_string()),
                _ => unreachable!()
            })
        );
        let res = try!(request_twitter(
            Post, request_token_url.clone(), &params[..], authorization));
        let v = form_urlencoded::parse(res.raw_response.as_bytes()).collect::<Vec<_>>();
        let oauth_token = v.iter().find(|x| x.0 == "oauth_token");
        let oauth_token_secret = v.iter().find(|x| x.0 == "oauth_token_secret");
        let oauth_callback_confirmed = v.iter()
            .find(|x| x.0 == "oauth_callback_confirmed")
            .and_then(|x| (&x.1[..]).parse().ok());
        match oauth_token.and(oauth_token_secret) {
            Some(_) => Ok(res.object(RequestTokenResponse {
                consumer_key: self.consumer_key.clone(),
                consumer_secret: self.consumer_secret.clone(),
                oauth_token: oauth_token.unwrap().1.clone().to_string(),
                oauth_token_secret: oauth_token_secret.unwrap().1.clone().to_string(),
                oauth_callback_confirmed: oauth_callback_confirmed.unwrap_or(false)
            })),
            None => Err(TwitterError::ParseError(res.clone()))
        }
    }
}

pub fn request_token(consumer_key: &str, consumer_secret: &str, oauth_callback: &str)
    -> RequestTokenRequestBuilder
{
    RequestTokenRequestBuilder {
        consumer_key: consumer_key.to_string(),
        consumer_secret: consumer_secret.to_string(),
        oauth_callback: oauth_callback.to_string(),
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

// TODO: Cow
#[derive(Clone, Debug)]
pub struct AccessTokenRequestBuilder {
    consumer_key: String,
    consumer_secret: String,
    oauth_token: String,
    oauth_token_secret: String,
    oauth_verifier: String
}

impl AccessTokenRequestBuilder {
    pub fn execute(&self) -> TwitterResult<AccessTokenResponse> {
        let access_token_url = Url::parse("https://api.twitter.com/oauth/access_token").unwrap();
        let authorization = oauthcli::authorization_header(
            "POST",
            access_token_url.clone(),
            None,
            &self.consumer_key[..],
            &self.consumer_secret[..],
            Some(&self.oauth_token[..]),
            Some(&self.oauth_token_secret[..]),
            SignatureMethod::HmacSha1,
            &oauthcli::timestamp()[..],
            &oauthcli::nonce()[..],
            None,
            Some(&self.oauth_verifier[..]),
            Vec::new().into_iter()
        );
        let res = try!(request_twitter(
            Post, access_token_url.clone(), &[], authorization));
        let v = form_urlencoded::parse(res.raw_response.as_bytes()).collect::<Vec<_>>();
        let oauth_token = v.iter().find(|x| x.0 == "oauth_token");
        let oauth_token_secret = v.iter().find(|x| x.0 == "oauth_token_secret");
        let user_id = v.iter().find(|x| x.0 == "user_id")
            .and_then(|x| (&x.1[..]).parse().ok());
        let screen_name = v.iter().find(|x| x.0 == "screen_name");
        match oauth_token.and(oauth_token_secret).and(user_id).and(screen_name) {
            Some(_) => Ok(res.object(AccessTokenResponse {
                consumer_key: self.consumer_key.to_string(),
                consumer_secret: self.consumer_secret.to_string(),
                oauth_token: oauth_token.unwrap().1.clone().to_string(),
                oauth_token_secret: oauth_token_secret.unwrap().1.clone().to_string(),
                user_id: user_id.unwrap(),
                screen_name: screen_name.unwrap().1.clone().to_string()
            })),
            None => Err(TwitterError::ParseError(res.clone()))
        }
    }
}

pub fn access_token(consumer_key: &str, consumer_secret: &str,
    oauth_token: &str, oauth_token_secret: &str, oauth_verifier: &str)
    -> AccessTokenRequestBuilder
{
    AccessTokenRequestBuilder {
        consumer_key: consumer_key.to_string(),
        consumer_secret: consumer_secret.to_string(),
        oauth_token: oauth_token.to_string(),
        oauth_token_secret: oauth_token_secret.to_string(),
        oauth_verifier: oauth_verifier.to_string()
    }
}
