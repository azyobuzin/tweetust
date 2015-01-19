//! The functions to get your access token for
//! [3-legged OAuth](https://dev.twitter.com/oauth/3-legged) or
//! [PIN-based OAuth](https://dev.twitter.com/oauth/pin-based).

use hyper::{Post, Url};
use oauthcli::{self, SignatureMethod};
use url::form_urlencoded;
use ::{OAuthAuthenticator, TwitterError, TwitterResult};
use conn::{send_request, read_to_twitter_result};
use conn::Parameter::Value;

#[derive(Clone, Show)]
pub struct RequestTokenResult {
    consumer_key: String,
    consumer_secret: String,
    pub oauth_token: String,
    pub oauth_token_secret: String,
    pub oauth_callback_confirmed: bool
}

#[derive(Clone, Show)]
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

    pub fn execute(&self) -> TwitterResult<RequestTokenResult> {
        let request_token_url = Url::parse("https://api.twitter.com/oauth/request_token").unwrap();
        let mut params = Vec::new();
        match self.x_auth_access_type {
            Some(ref x) => params.push(Value("x_auth_access_type", x.clone())),
            None => ()
        }
        let authorization = oauthcli::authorization_header(
            "POST",
            request_token_url.clone(),
            None,
            self.consumer_key.as_slice(),
            self.consumer_secret.as_slice(),
            None,
            None,
            SignatureMethod::HmacSha1,
            oauthcli::timestamp(),
            oauthcli::nonce(),
            Some(self.oauth_callback.as_slice()),
            None,
            params.iter().map(|x| match x {
                &Value(key, ref val) => (key.to_string(), val.clone()),
                _ => unreachable!()
            })
        );
        let result = send_request(Post, request_token_url.clone(), params.as_slice(), authorization);
        match read_to_twitter_result(result) {
            Ok(res) => {
                let v = form_urlencoded::parse(res.raw_response.as_bytes());
                let oauth_token = v.iter().find(|x| x.0 == "oauth_token");
                let oauth_token_secret = v.iter().find(|x| x.0 == "oauth_token_secret");
                let oauth_callback_confirmed = v.iter()
                    .find(|x| x.0 == "oauth_callback_confirmed")
                    .and_then(|x| x.1.as_slice().parse());
                match oauth_token.and(oauth_token_secret) {
                    Some(_) => Ok(res.object(RequestTokenResult {
                        consumer_key: self.consumer_key.clone(),
                        consumer_secret: self.consumer_secret.clone(),
                        oauth_token: oauth_token.unwrap().1.clone(),
                        oauth_token_secret: oauth_token_secret.unwrap().1.clone(),
                        oauth_callback_confirmed: oauth_callback_confirmed.unwrap_or(false)
                    })),
                    None => Err(TwitterError::ParseError(res))
                }
            },
            Err(e) => Err(e)
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

#[derive(Clone, Show)]
pub struct AccessTokenResult {
    consumer_key: String,
    consumer_secret: String,
    pub oauth_token: String,
    pub oauth_token_secret: String,
    pub user_id: u64,
    pub screen_name: String
}

impl AccessTokenResult {
    pub fn to_authenticator(&self) -> OAuthAuthenticator {
        OAuthAuthenticator::new(
            self.consumer_key.as_slice(),
            self.consumer_secret.as_slice(),
            self.oauth_token.as_slice(),
            self.oauth_token_secret.as_slice()
        )
    }
}

#[derive(Clone, Show)]
pub struct AccessTokenRequestBuilder {
    consumer_key: String,
    consumer_secret: String,
    oauth_token: String,
    oauth_token_secret: String,
    oauth_verifier: String
}

impl AccessTokenRequestBuilder {
    pub fn execute(&self) -> TwitterResult<AccessTokenResult> {
        let access_token_url = Url::parse("https://api.twitter.com/oauth/access_token").unwrap();
        let authorization = oauthcli::authorization_header(
            "POST",
            access_token_url.clone(),
            None,
            self.consumer_key.as_slice(),
            self.consumer_secret.as_slice(),
            Some(self.oauth_token.as_slice()),
            Some(self.oauth_token_secret.as_slice()),
            SignatureMethod::HmacSha1,
            oauthcli::timestamp(),
            oauthcli::nonce(),
            None,
            Some(self.oauth_verifier.as_slice()),
            Vec::new().into_iter()
        );
        let result = send_request(Post, access_token_url.clone(), &[], authorization);
        match read_to_twitter_result(result) {
            Ok(res) => {
                let v = form_urlencoded::parse(res.raw_response.as_bytes());
                let oauth_token = v.iter().find(|x| x.0 == "oauth_token");
                let oauth_token_secret = v.iter().find(|x| x.0 == "oauth_token_secret");
                let user_id = v.iter().find(|x| x.0 == "user_id")
                    .and_then(|x| x.1.as_slice().parse());
                let screen_name = v.iter().find(|x| x.0 == "screen_name");
                match oauth_token.and(oauth_token_secret).and(user_id).and(screen_name) {
                    Some(_) => Ok(res.object(AccessTokenResult {
                        consumer_key: self.consumer_key.to_string(),
                        consumer_secret: self.consumer_secret.to_string(),
                        oauth_token: oauth_token.unwrap().1.clone(),
                        oauth_token_secret: oauth_token_secret.unwrap().1.clone(),
                        user_id: user_id.unwrap(),
                        screen_name: screen_name.unwrap().1.clone()
                    })),
                    None => Err(TwitterError::ParseError(res))
                }
            },
            Err(e) => Err(e)
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

impl RequestTokenResult {
    pub fn access_token(&self, oauth_verifier: &str) -> AccessTokenRequestBuilder {
        access_token(
            self.consumer_key.as_slice(),
            self.consumer_secret.as_slice(),
            self.oauth_token.as_slice(),
            self.oauth_token_secret.as_slice(),
            oauth_verifier
        )
    }
}
