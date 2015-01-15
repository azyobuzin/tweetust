use std::str::FromStr;
use hyper::{Post, Url};
use oauthcli::{self, SignatureMethod};
use url::form_urlencoded;
use super::{OAuthAuthenticator, TwitterError, TwitterResult};
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
            Some(ref x) => params.push(Value("x_auth_access_type", x.as_slice())),
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
                &Value(key, val) => (key.to_string(), val.to_string()),
                _ => unreachable!()
            })
        );
        let result = send_request(Post, request_token_url.clone(), params.as_slice(), authorization);
        match read_to_twitter_result(result) {
            Ok(res) => {
                let v = form_urlencoded::parse(res.raw_response.as_bytes());
                let oauth_token = v.iter().find(|x| x.0 == "oauth_token");
                let oauth_token_secret = v.iter().find(|x| x.0 == "oauth_token_secret");
                let oauth_callback_confirmed = v.iter().find(|x| x.0 == "oauth_callback_confirmed");
                match oauth_token.and(oauth_token_secret).and(oauth_callback_confirmed) {
                    Some(_) => Ok(res.object(RequestTokenResult {
                        consumer_key: self.consumer_key.clone(),
                        consumer_secret: self.consumer_secret.clone(),
                        oauth_token: oauth_token.unwrap().1.clone(),
                        oauth_token_secret: oauth_token_secret.unwrap().1.clone(),
                        oauth_callback_confirmed: FromStr::from_str(
                            oauth_callback_confirmed.unwrap().1.as_slice()
                        ).unwrap_or(false)
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

pub fn access_token(consumer_key: &str, consumer_secret: &str,
    oauth_token: &str, oauth_token_secret: &str, oauth_verifier: &str)
    -> TwitterResult<AccessTokenResult>
{
    let access_token_url = Url::parse("https://api.twitter.com/oauth/access_token").unwrap();
    let authorization = oauthcli::authorization_header(
        "POST",
        access_token_url.clone(),
        None,
        consumer_key,
        consumer_secret,
        Some(oauth_token),
        Some(oauth_token_secret),
        SignatureMethod::HmacSha1,
        oauthcli::timestamp(),
        oauthcli::nonce(),
        None,
        Some(oauth_verifier),
        Vec::new().into_iter()
    );
    let result = send_request(Post, access_token_url.clone(), &[], authorization);
    match read_to_twitter_result(result) {
        Ok(res) => {
            let v = form_urlencoded::parse(res.raw_response.as_bytes());
            let oauth_token = v.iter().find(|x| x.0 == "oauth_token");
            let oauth_token_secret = v.iter().find(|x| x.0 == "oauth_token_secret");
            let user_id = v.iter().find(|x| x.0 == "user_id")
                .and_then(|x| FromStr::from_str(x.1.as_slice()));
            let screen_name = v.iter().find(|x| x.0 == "screen_name");
            match oauth_token.and(oauth_token_secret).and(user_id).and(screen_name) {
                Some(_) => Ok(res.object(AccessTokenResult {
                    consumer_key: consumer_key.to_string(),
                    consumer_secret: consumer_secret.to_string(),
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

impl RequestTokenResult {
    pub fn access_token(&self, oauth_verifier: &str) -> TwitterResult<AccessTokenResult> {
        access_token(
            self.consumer_key.as_slice(),
            self.consumer_secret.as_slice(),
            self.oauth_token.as_slice(),
            self.oauth_token_secret.as_slice(),
            oauth_verifier
        )
    }
}
