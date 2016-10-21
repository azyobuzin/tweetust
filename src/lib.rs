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
//! It's easy for people who have leaned about Twitter, isn't it?

#![warn(unused_import_braces)]

extern crate chrono;
extern crate hyper;
extern crate oauthcli;
extern crate serde;
extern crate serde_json;
extern crate url;

use std::error::Error;
use std::fmt;
use models::TwitterResponse;
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
    HttpError(hyper::Error),
    JsonError(serde_json::Error, TwitterResponse<()>),
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

impl fmt::Display for TwitterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TwitterError::ErrorResponse(ref e) => fmt::Display::fmt(e, f),
            TwitterError::HttpError(ref e) => fmt::Display::fmt(e, f),
            TwitterError::JsonError(..) => fmt::Debug::fmt(self, f),
            TwitterError::ParseError(ref e) => write!(f, "ParseError: {:?}", e)
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
        TwitterError::HttpError(err)
    }
}

pub type TwitterResult<T> = Result<TwitterResponse<T>, TwitterError>;
