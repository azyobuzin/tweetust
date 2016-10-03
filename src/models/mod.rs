use std::rc::Rc;

#[derive(Clone, Debug)]
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
            rate_limit: self.rate_limit.clone()
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/models/_models_list.rs"));
