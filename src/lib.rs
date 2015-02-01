//! Tweetust is the simple wrapper for Twitter API.
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
//! ```ignore
//! // extern crate tweetust; or use tweetust;
//! let your_tweet =
//!   tweetust::TwitterClient::new(&auth)
//!     .statuses()
//!     .update("My First Tweet!")
//!     .execute();
//! ```
//! It's easy for people who have leaned about Twitter, isn't it?

#![allow(unused_must_use)]
#![warn(unused_import_braces, unused_typecasts)]
#![feature(box_syntax, collections, core, io, plugin)]

extern crate hyper;
extern crate oauthcli;
extern crate "rustc-serialize" as rustc_serialize;
extern crate url;

#[plugin]
#[no_link]
extern crate tweetust_macros;

use std::error::{Error, FromError};
use std::fmt;
use rustc_serialize::json;
use models::TwitterResponse;
use models::error::ErrorResponse;

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

impl FromError<ErrorResponse> for TwitterError {
    fn from_error(err: ErrorResponse) -> TwitterError {
        TwitterError::ErrorResponse(err)
    }
}

impl FromError<hyper::HttpError> for TwitterError {
    fn from_error(err: hyper::HttpError) -> TwitterError {
        TwitterError::HttpError(err)
    }
}

pub type TwitterResult<T> = Result<TwitterResponse<T>, TwitterError>;
