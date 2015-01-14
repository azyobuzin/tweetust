use std::str::FromStr;
use hyper::{Post, Url};
use oauthcli::{self, SignatureMethod};
use url::form_urlencoded;
use super::{TwitterError, TwitterResult};
use conn::{send_request, read_to_twitter_result};
use conn::Parameter::Value;

#[derive(Clone, Show)]
pub struct RequestTokenResult {
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
            }).collect::<Vec<_>>().iter()
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
