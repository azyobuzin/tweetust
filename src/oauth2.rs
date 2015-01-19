//! The functions to get and invalidate your access token for
//! [Application-only authentication](https://dev.twitter.com/oauth/application-only).

use hyper::Post;
use rustc_serialize::base64::{self, ToBase64};
use url::{percent_encoding, Url};
use ::{ApplicationOnlyAuthenticator, TwitterError, TwitterResult};
use conn::{send_request, read_to_twitter_result, parse_json};
use conn::Parameter::Value;

#[inline]
fn percent_encode(input: &str) -> String {
    percent_encoding::utf8_percent_encode(
        input,
        percent_encoding::FORM_URLENCODED_ENCODE_SET
    )
}

fn basic_authorization(consumer_key: &str, consumer_secret: &str) -> String {
    format!(
        "Basic {}",
        format!("{}:{}", percent_encode(consumer_key),
            percent_encode(consumer_secret))
            .as_bytes().to_base64(base64::Config {
                char_set: base64::CharacterSet::Standard,
                newline: base64::Newline::LF,
                pad: true,
                line_length: None
            })
    )
}

#[derive(Clone, Show, RustcDecodable)]
pub struct TokenResult {
    pub token_type: String,
    pub access_token: String
}

impl TokenResult {
    pub fn to_authenticator(self) -> ApplicationOnlyAuthenticator {
        ApplicationOnlyAuthenticator(self.access_token)
    }
}

#[derive(Clone, Show)]
pub struct TokenRequestBuilder {
    consumer_key: String,
    consumer_secret: String,
    grant_type: String
}

impl TokenRequestBuilder {
    pub fn grant_type(mut self, val: &str) -> TokenRequestBuilder {
        self.grant_type = val.to_string();
        self
    }

    pub fn execute(&self) -> TwitterResult<TokenResult> {
        let result = send_request(
            Post,
            Url::parse("https://api.twitter.com/oauth2/token").unwrap(),
            &[Value("grant_type", self.grant_type.clone())],
            basic_authorization(
                self.consumer_key.as_slice(), self.consumer_secret.as_slice())
        );
        match read_to_twitter_result(result) {
            Ok(res) => match parse_json(res.raw_response.as_slice()) {
                Ok(j) => Ok(res.object(j)),
                Err(e) => Err(TwitterError::JsonError(e, res))
            },
            Err(e) => Err(e)
        }
    }
}

pub fn token(consumer_key: &str, consumer_secret: &str) -> TokenRequestBuilder {
    TokenRequestBuilder {
        consumer_key: consumer_key.to_string(),
        consumer_secret: consumer_secret.to_string(),
        grant_type: "client_credentials".to_string()
    }
}

#[derive(Clone, Show, RustcDecodable)]
pub struct InvalidateTokenResult {
    pub access_token: String
}

#[derive(Clone, Show)]
pub struct InvalidateTokenRequestBuilder {
    consumer_key: String,
    consumer_secret: String,
    access_token: String
}

impl InvalidateTokenRequestBuilder {
    pub fn execute(&self) -> TwitterResult<InvalidateTokenResult> {
        let result = send_request(
            Post,
            Url::parse("https://api.twitter.com/oauth2/invalidate_token").unwrap(),
            &[Value("access_token", percent_encoding::lossy_utf8_percent_decode(
                self.access_token.as_bytes()))],
            basic_authorization(
                self.consumer_key.as_slice(), self.consumer_secret.as_slice())
        );
        match read_to_twitter_result(result) {
            Ok(res) => match parse_json(res.raw_response.as_slice()) {
                Ok(j) => Ok(res.object(j)),
                Err(e) => Err(TwitterError::JsonError(e, res))
            },
            Err(e) => Err(e)
        }
    }
}

pub fn invalidate_token(consumer_key: &str, consumer_secret: &str, access_token: &str)
    -> InvalidateTokenRequestBuilder
{
    InvalidateTokenRequestBuilder {
        consumer_key: consumer_key.to_string(),
        consumer_secret: consumer_secret.to_string(),
        access_token: access_token.to_string()
    }
}
