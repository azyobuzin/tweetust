#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoResponse {
    pub result: GeoResult,
    pub query: GeoQuery,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoResult {
    pub places: Vec<Place>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoQuery {
    pub url: String,
    #[serde(rename = "type")]
    pub query_type: String,
    pub params: GeoQueryParams,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoQueryParams {
    pub accuracy: f64,
    pub granularity: String,
    pub query: Option<String>,
    pub autocomplete: Option<bool>,
    pub trim_place: Option<bool>,
    pub coordinates: TweetCoordinates,
}
