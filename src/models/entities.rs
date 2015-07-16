#[derive(Clone, Debug, RustcDecodable)]
pub struct Entities {
    pub hashtags: Vec<Symbol>,
    pub symbols: Vec<Symbol>,
    pub media: Option<Vec<Medium>>,
    pub urls: Vec<Url>,
    pub user_mentions: Vec<UserMention>
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct ExtendedEntities {
    pub media: Vec<Medium>
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct UserEntities {
    pub url: Option<UserEntitiesField>,
    pub description: UserEntitiesField
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct UserEntitiesField {
    pub urls: Vec<Url>
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Symbol {
    pub indices: Vec<i32>,
    pub text: String
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Medium {
    pub display_url: String,
    pub expanded_url: String,
    pub id: i64,
    pub indices: Vec<i32>,
    pub media_url: String,
    pub media_url_https: String,
    pub sizes: Sizes,
    pub source_status_id: Option<i64>,
    pub type_: String,
    pub url: String,
    pub video_info: Option<VideoInfo>
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Size {
    pub h: u32,
    pub resize: String,
    pub w: u32
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Sizes {
    pub thumb: Option<Size>,
    pub large: Option<Size>,
    pub medium: Option<Size>,
    pub small: Option<Size>
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct VideoInfo {
    pub aspect_ratio: Vec<u32>,
    pub duration_millis: Option<u32>,
    pub variants: Vec<Variant>
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Variant {
    pub bitrate: Option<u32>,
    pub content_type: String,
    pub url: String
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Url {
    pub display_url: String,
    pub expanded_url: String,
    pub indices: Vec<i32>,
    pub url: String
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct UserMention {
    pub id: i64,
    pub indices: Vec<i32>,
    pub name: String,
    pub screen_name: String
}
