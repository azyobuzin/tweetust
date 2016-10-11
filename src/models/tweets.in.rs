#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tweet {
    pub contributors: Option<Vec<Contributor>>,
    pub coordinates: Option<Coordinates>,
    pub created_at: String,
    pub current_user_retweet: Option<CurrentUserRetweet>,
    pub entities: Option<Box<Entities>>,
    pub extended_entities: Option<ExtendedEntities>,
    pub favorite_count: Option<u32>,
    pub favorited: Option<bool>,
    pub filter_level: Option<String>,
    pub id: i64,
    pub in_reply_to_screen_name: Option<String>,
    pub in_reply_to_status_id: Option<i64>,
    pub in_reply_to_user_id: Option<i64>,
    pub lang: Option<String>,
    pub place: Option<Place>,
    pub possibly_sensitive: Option<bool>,
    //pub scopes: Option<BTreeMap<String, json::Json>>,
    pub retweet_count: u32,
    pub retweeted: Option<bool>,
    pub retweeted_status: Option<Box<Tweet>>,
    pub source: String,
    pub text: String,
    pub user: Option<Box<User>>,
    pub withheld_copyright: Option<bool>,
    pub withheld_in_countries: Option<Vec<String>>,
    pub withheld_scope: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contributor {
    pub id: i64,
    pub screen_name: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Coordinates {
    pub coordinates: Vec<f64>,
    #[serde(rename = "type")]
    pub coordinates_type: String
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CurrentUserRetweet {
    pub id: i64
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LookupMap {
    pub id: HashMap<String, Option<Tweet>>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OEmbed {
    pub cache_age: String,
    pub url: String,
    pub provider_url: String,
    pub provider_name: String,
    pub author_name: String,
    pub version: String,
    pub author_url: String,
    pub type_: String,
    pub html: String,
    pub height: Option<i32>,
    pub width: Option<i32>
}
