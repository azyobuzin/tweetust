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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TextRange {
    pub start: i32,
    pub end: i32
}

impl Serialize for TextRange {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        let mut state = try!(serializer.serialize_seq(Some(2)));
        try!(serializer.serialize_seq_elt(&mut state, self.start));
        try!(serializer.serialize_seq_elt(&mut state, self.end));
        serializer.serialize_seq_end(state)
    }
}

impl Deserialize for TextRange {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        struct Visitor;
        impl de::Visitor for Visitor {
            type Value = TextRange;
            fn visit_seq<V: de::SeqVisitor>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> {
                let start: i32 = try!(visitor.visit().and_then(|x| x.ok_or_else(|| de::Error::invalid_length(0))));
                let end: i32 = try!(visitor.visit().and_then(|x| x.ok_or_else(|| de::Error::invalid_length(1))));
                try!(visitor.end());
                Ok(TextRange { start: start, end: end })
            }
        }

        deserializer.deserialize_seq_fixed_size(2, Visitor)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolEntity {
    pub indices: TextRange,
    pub text: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaEntity {
    pub ext_alt_text: Option<String>,
    pub display_url: String,
    pub expanded_url: String,
    pub id: i64,
    pub indices: TextRange,
    pub media_url: String,
    pub media_url_https: String,
    pub sizes: MediaSizes,
    pub source_status_id: Option<i64>,
    #[serde(rename = "type")]
    pub media_type: String,
    pub url: String,
    pub video_info: Option<Box<VideoInfo>>
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
    pub indices: TextRange,
    pub url: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserMentionEntity {
    pub id: i64,
    pub indices: TextRange,
    pub name: String,
    pub screen_name: String
}
