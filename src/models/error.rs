use std;
use hyper::status::StatusCode;
use super::RateLimitStatus;

#[derive(Clone, Show, RustcDecodable)]
pub struct Error {
    pub code: i32,
    pub message: String
}

#[derive(Clone, Show)]
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

    fn detail(&self) -> Option<String> {
        let s = match self.errors {
            Some(ref x) => {
                let ref e = x[0];
                format!("{}: {} {}", self.status, e.code, e.message)
            },
            None => format!("{}: {}", self.status, self.raw_response)
        };
        Some(s)
    }
}
