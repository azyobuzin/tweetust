use super::tweets::Tweet;

#[derive(Clone, Show, RustcDecodable)]
pub struct SearchResponse {
    pub statuses: Vec<Tweet>,
    pub search_metadata: SearchMetadata
}

#[derive(Clone, Show, RustcDecodable)]
pub struct SearchMetadata {
    pub max_id: u64,
    pub since_id: u64,
    pub refresh_url: Option<String>,
    pub next_results: Option<String>,
    pub count: i32,
    pub completed_in: f32,
    pub query: String
}
