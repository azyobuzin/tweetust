use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub struct RateLimitStatus {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64
}

#[derive(Clone, Debug)]
pub struct TwitterResponse<T> {
    pub object: T,
    pub raw_response: Rc<String>,
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

#[derive(Clone, Debug, RustcDecodable)]
pub struct CursorIds {
    pub previous_cursor: i64,
    pub next_cursor: i64,
    pub ids: Vec<u64>
}

pub mod error;
pub mod direct_messages;
pub mod entities;
pub mod friendships;
pub mod places;
pub mod search;
pub mod tweets;
pub mod users;
