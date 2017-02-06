//! Tweetust is a simple wrapper for Twitter API.
//!
//! # Getting started
//! This is a Twitter API wrapper, so you must lean Twitter API.
//! [Visit the official document](https://dev.twitter.com/).
//!
//! After getting the API key, let's start using tweetust.
//!
//! # How to get the access token
//! See [oauth::request_token function](oauth/fn.request_token.html).
//! After getting the access token, you can use [to_authenticator function](oauth/struct.AccessTokenResponse.html#method.to_authenticator)
//! to make [OAuthAuthenticator](conn/oauth_authenticator/struct.OAuthAuthenticator.html).
//!
//! # How to create OAuthAuthenticator with an access token string
//! See [OAuthAuthenticator::new](conn/oauth_authenticator/struct.OAuthAuthenticator.html#method.new).
//!
//! # The first tweeting
//! When you created OAuthAuthenticator and set to `auth` variable, you can tweet in a minute.
//!
//! ```no_run
//! use tweetust::*;
//!
//! let auth = OAuthAuthenticator::new("API Key", "API Secret", "Access Token", "Access Token Secret");
//!
//! let your_tweet =
//!   TwitterClient::new(auth)
//!     .statuses()
//!     .update("My First Tweet!")
//!     .execute();
//! ```
//!
//! It's easy for those who have leaned about Twitter, isn't it?

#![warn(unused_import_braces)]

pub extern crate chrono;
pub extern crate hyper;
extern crate multipart;
extern crate oauthcli;
extern crate serde;
extern crate serde_json;
extern crate url;

use std::error::Error;
use std::fmt;
use std::io;
use models::{RawResponse, TwitterResponse};
use models::ErrorResponse;

pub use clients::TwitterClient;
pub use conn::application_only_authenticator::ApplicationOnlyAuthenticator;
pub use conn::oauth_authenticator::OAuthAuthenticator;

pub mod clients;
pub mod conn;
pub mod models;
pub mod oauth;
pub mod oauth2;

#[derive(Debug)]
pub enum TwitterError {
    ErrorResponse(ErrorResponse),
    Url(url::ParseError),
    InvalidRequest,
    Io(io::Error),
    Http(hyper::Error),
    ParseResponse(Option<serde_json::Error>, RawResponse),
}

impl Error for TwitterError {
    fn description(&self) -> &str {
        "an error occured in your request"
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TwitterError::ErrorResponse(ref e) => Some(e),
            TwitterError::Url(ref e) => Some(e),
            TwitterError::InvalidRequest => None,
            TwitterError::Io(ref e) => Some(e),
            TwitterError::Http(ref e) => Some(e),
            TwitterError::ParseResponse(Some(ref e), _) => Some(e),
            TwitterError::ParseResponse(None, _) => None,
        }
    }
}

impl fmt::Display for TwitterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TwitterError::ErrorResponse(ref e) => fmt::Display::fmt(e, f),
            TwitterError::Url(ref e) => fmt::Display::fmt(e, f),
            TwitterError::InvalidRequest => f.write_str("invalid request"),
            TwitterError::Io(ref e) => fmt::Display::fmt(e, f),
            TwitterError::Http(ref e) => fmt::Display::fmt(e, f),
            TwitterError::ParseResponse(_, ref res) => write!(f, "invalid response body: {}", res.raw_response),
        }
    }
}

impl From<ErrorResponse> for TwitterError {
    fn from(err: ErrorResponse) -> TwitterError {
        TwitterError::ErrorResponse(err)
    }
}

impl From<hyper::Error> for TwitterError {
    fn from(err: hyper::Error) -> TwitterError {
        TwitterError::Http(err)
    }
}

impl From<url::ParseError> for TwitterError {
    fn from(err: url::ParseError) -> TwitterError {
        TwitterError::Url(err)
    }
}

impl From<io::Error> for TwitterError {
    fn from(err: io::Error) -> TwitterError {
        TwitterError::Io(err)
    }
}

pub type TwitterResult<T> = Result<TwitterResponse<T>, TwitterError>;

fn parse_json<T: serde::de::Deserialize>(s: &str) -> serde_json::Result<T> {
    serde_json::from_str(s)
}
