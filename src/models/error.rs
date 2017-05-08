#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Error {
    pub code: i32,
    pub message: String
}

#[derive(Clone, Debug)]
pub struct ErrorResponse {
    pub status: ::hyper::status::StatusCode,
    pub errors: Option<Vec<Error>>,
    pub raw_response: String,
    pub rate_limit: Option<RateLimitStatus>
}

impl std::error::Error for ErrorResponse {
    fn description(&self) -> &str {
        "the server returned an error response"
    }
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.errors {
            Some(ref x) => {
                let ref e = x[0];
                write!(f, "{}: {} {}", self.status, e.code, e.message)
            },
            None => write!(f, "{}: {}", self.status, self.raw_response)
        }
    }
}
