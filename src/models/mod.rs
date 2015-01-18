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

#[derive(Clone, Show, RustcDecodable)]
pub struct CursorIds {
    pub previous_cursor: i64,
    pub next_cursor: i64,
    pub ids: Vec<u64>
}

pub mod error;
pub mod entities;
pub mod places;
pub mod tweets;
pub mod users;
