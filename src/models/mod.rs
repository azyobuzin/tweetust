use ::{conn, TwitterError, TwitterResult};

#[derive(Clone, Debug)]
pub struct RateLimitStatus {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64
}

#[derive(Clone, Debug)]
pub struct TwitterResponse<T> {
    pub object: T,
    pub raw_response: String,
    pub rate_limit: Option<RateLimitStatus>
}

impl TwitterResponse<()> {
    pub fn parse_to_object<T: ::serde::de::Deserialize>(self) -> TwitterResult<T> {
        match conn::parse_json(&self.raw_response) {
            Ok(x) => Ok(TwitterResponse {
                object: x,
                raw_response: self.raw_response,
                rate_limit: self.rate_limit
            }),
            Err(x) => Err(TwitterError::JsonError(x, self))
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/models/_models_list.rs"));
