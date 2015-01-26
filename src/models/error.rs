use std::{self, fmt};
use hyper::status::StatusCode;
use super::RateLimitStatus;

#[derive(Clone, Debug, RustcDecodable)]
pub struct Error {
    pub code: i32,
    pub message: String
}

#[derive(Clone, Debug)]
pub struct ErrorResponse {
    pub status: StatusCode,
    pub errors: Option<Vec<Error>>,
    pub raw_response: String,
    pub rate_limit: Option<RateLimitStatus>
}

impl std::error::Error for ErrorResponse {
    fn description(&self) -> &str {
        "the server returned an error response"
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.errors {
            Some(ref x) => {
                let ref e = x[0];
                write!(f, "{}: {} {}", self.status, e.code, e.message)
            },
            None => write!(f, "{}: {}", self.status, self.raw_response)
        }
    }
}
