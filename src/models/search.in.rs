#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub statuses: Vec<Tweet>,
    pub search_metadata: SearchMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub max_id: i64,
    pub since_id: i64,
    pub refresh_url: Option<String>,
    pub next_results: Option<String>,
    pub count: i32,
    pub completed_in: f32,
    pub query: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedSearch {
    pub created_at: CreatedAt,
    pub id: i64,
    pub name: String,
    pub query: String,
    // pub position: Option<?>
}
