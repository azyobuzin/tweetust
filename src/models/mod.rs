pub mod error;

#[derive(Clone, Copy, Show)]
pub struct RateLimitStatus {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64
}

#[derive(Clone, Show)]
pub struct TwitterResponse<T> {
    pub object: T,
    pub raw_response: String,
    pub rate_limit: Option<RateLimitStatus>
}

impl TwitterResponse<()> {
    pub fn object<T>(&self, val: T) -> TwitterResponse<T> {
        TwitterResponse {
            object: val,
            raw_response: self.raw_response.clone(),
            rate_limit: self.rate_limit
        }
    }
}
