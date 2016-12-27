#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64,
}

impl RateLimitStatus {
    pub fn reset_date_time(&self) -> chrono::DateTime<chrono::UTC> {
        use chrono::TimeZone;
        chrono::UTC.timestamp(self.reset, 0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateLimitStatusResponse {
    pub rate_limit_context: RateLimitContext,
    pub resources: HashMap<String, HashMap<String, RateLimitStatus>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateLimitContext {
    pub access_token: String,
}
