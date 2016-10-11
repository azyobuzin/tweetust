#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entities {
    pub hashtags: Vec<SymbolEntity>,
    pub symbols: Vec<SymbolEntity>,
    pub media: Option<Vec<MediaEntity>>,
    pub urls: Vec<UrlEntity>,
    pub user_mentions: Vec<UserMentionEntity>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtendedEntities {
    pub media: Vec<MediaEntity>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserEntities {
    pub url: Option<UserEntitiesField>,
    pub description: UserEntitiesField
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserEntitiesField {
    pub urls: Vec<UrlEntity>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolEntity {
    pub indices: Vec<i32>,
    pub text: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaEntity {
    pub ext_alt_text: Option<String>,
    pub display_url: String,
    pub expanded_url: String,
    pub id: i64,
    pub indices: Vec<i32>,
    pub media_url: String,
    pub media_url_https: String,
    pub sizes: MediaSizes,
    pub source_status_id: Option<i64>,
    #[serde(rename = "type")]
    pub media_type: String,
    pub url: String,
    pub video_info: Option<VideoInfo>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaSize {
    pub h: u32,
    pub resize: String,
    pub w: u32
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaSizes {
    pub thumb: Option<MediaSize>,
    pub large: Option<MediaSize>,
    pub medium: Option<MediaSize>,
    pub small: Option<MediaSize>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub aspect_ratio: Vec<u32>,
    pub duration_millis: Option<u32>,
    pub variants: Vec<VideoVariant>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoVariant {
    pub bitrate: Option<u32>,
    pub content_type: String,
    pub url: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UrlEntity {
    pub display_url: String,
    pub expanded_url: String,
    pub indices: Vec<i32>,
    pub url: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserMentionEntity {
    pub id: i64,
    pub indices: Vec<i32>,
    pub name: String,
    pub screen_name: String
}
