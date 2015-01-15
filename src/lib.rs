#![allow(unstable)]
#![feature(plugin)]

extern crate hyper;
extern crate oauthcli;
extern crate "rustc-serialize" as rustc_serialize;
extern crate url;

#[plugin]
#[no_link]
extern crate tweetust_macros;

use std::error::{Error, FromError};
use rustc_serialize::json;
use models::TwitterResponse;

pub use auth::{access_token, request_token};
pub use clients::TwitterClient;
pub use conn::oauth_authenticator::OAuthAuthenticator;

pub mod auth;
pub mod clients;
pub mod conn;
pub mod models;

#[derive(Show)]
pub enum TwitterError {
    ErrorResponse(models::error::ErrorResponse),
    HttpError(hyper::HttpError),
    JsonError(json::DecoderError, TwitterResponse<()>),
    ParseError(TwitterResponse<()>)
}

impl Error for TwitterError {
    fn description(&self) -> &str {
        "an error occured in your request"
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TwitterError::ErrorResponse(ref e) => Some(e),
            TwitterError::HttpError(ref e) => Some(e),
            TwitterError::JsonError(ref e, _) => Some(e),
            TwitterError::ParseError(_) => None
        }
    }
}

impl FromError<hyper::HttpError> for TwitterError {
    fn from_error(err: hyper::HttpError) -> TwitterError {
        TwitterError::HttpError(err)
    }
}

pub type TwitterResult<T> = Result<TwitterResponse<T>, TwitterError>;
