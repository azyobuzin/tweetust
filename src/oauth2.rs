//! The functions to get and invalidate your access token for
//! [Application-only authentication](https://dev.twitter.com/oauth/application-only).

use hyper::Post;
use rustc_serialize::base64::{self, ToBase64};
use url::{form_urlencoded, percent_encoding, Url};
use ::{ApplicationOnlyAuthenticator, TwitterError, TwitterResult};
use conn::{request_twitter, parse_json};
use conn::Parameter::Value;

fn percent_encode(input: &str) -> String {
    form_urlencoded::byte_serialize(input.as_bytes()).collect()
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

#[derive(Clone, Debug, RustcDecodable)]
pub struct TokenResponse {
    pub token_type: String,
    pub access_token: String
}

impl TokenResponse {
    pub fn to_authenticator(self) -> ApplicationOnlyAuthenticator {
        ApplicationOnlyAuthenticator(self.access_token)
    }
}

#[derive(Clone, Debug)]
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

    pub fn execute(&self) -> TwitterResult<TokenResponse> {
        let res = try!(request_twitter(
            Post,
            Url::parse("https://api.twitter.com/oauth2/token").unwrap(),
            &[Value("grant_type", self.grant_type.clone())],
            basic_authorization(
                &self.consumer_key[..], &self.consumer_secret[..])
        ));
        match parse_json(&res.raw_response[..]) {
            Ok(j) => Ok(res.object(j)),
            Err(e) => Err(TwitterError::JsonError(e, res))
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

#[derive(Clone, Debug, RustcDecodable)]
pub struct InvalidateTokenResponse {
    pub access_token: String
}

#[derive(Clone, Debug)]
pub struct InvalidateTokenRequestBuilder {
    consumer_key: String,
    consumer_secret: String,
    access_token: String
}

impl InvalidateTokenRequestBuilder {
    pub fn execute(&self) -> TwitterResult<InvalidateTokenResponse> {
        let res = try!(request_twitter(
            Post,
            Url::parse("https://api.twitter.com/oauth2/invalidate_token").unwrap(),
            &[Value(
                "access_token",
                String::from_utf8(percent_encoding::percent_decode(self.access_token.as_bytes()).collect()).unwrap()
            )],
            basic_authorization(
                &self.consumer_key[..], &self.consumer_secret[..])
        ));
        match parse_json(&res.raw_response[..]) {
            Ok(j) => Ok(res.object(j)),
            Err(e) => Err(TwitterError::JsonError(e, res))
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
