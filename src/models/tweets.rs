#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tweet {
    pub contributors: Option<Vec<Contributor>>,
    pub coordinates: Option<TweetCoordinates>,
    pub created_at: CreatedAt,
    pub current_user_retweet: Option<CurrentUserRetweet>,
    pub display_text_range: Option<TextRange>,
    pub entities: Option<Box<Entities>>,
    pub extended_entities: Option<ExtendedEntities>,
    pub extended_tweet: Option<Box<CompatExtendedTweet>>,
    pub favorite_count: Option<u32>,
    pub favorited: Option<bool>,
    pub filter_level: Option<FilterLevel>,
    pub full_text: Option<String>,
    pub id: i64,
    pub in_reply_to_screen_name: Option<String>,
    pub in_reply_to_status_id: Option<i64>,
    pub in_reply_to_user_id: Option<i64>,
    pub is_quoted_status: Option<bool>,
    pub lang: Option<String>,
    pub place: Option<Box<Place>>,
    pub possibly_sensitive: Option<bool>,
    pub possibly_sensitive_appealable: Option<bool>,
    pub quoted_status_id: Option<i64>,
    pub quoted_status: Option<Box<Tweet>>,
    //pub scopes: Option<BTreeMap<String, json::Json>>,
    pub retweet_count: u32,
    pub retweeted: Option<bool>,
    pub retweeted_status: Option<Box<Tweet>>,
    pub source: String,
    pub text: Option<String>,
    pub truncated: Option<bool>,
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
pub struct TweetCoordinates {
    pub coordinates: Coordinates,
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
    #[serde(rename = "type")]
    pub content_type: String,
    pub html: String,
    pub height: Option<i32>,
    pub width: Option<i32>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompatExtendedTweet {
    pub full_text: String,
    pub display_text_range: TextRange,
    pub entities: Entities,
    pub extended_entities: Option<ExtendedEntities>,
}

enum_str!(FilterLevel {
    None("none"),
    Low("low"),
    Medium("medium"),
});
