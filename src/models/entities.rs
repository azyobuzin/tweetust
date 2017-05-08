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
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tup = try!(serializer.serialize_tuple(2));
        try!(tup.serialize_element(&self.start));
        try!(tup.serialize_element(&self.end));
        tup.end()
    }
}

impl<'x> Deserialize<'x> for TextRange {
    fn deserialize<D: Deserializer<'x>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'x> de::Visitor<'x> for Visitor {
            type Value = TextRange;

            fn visit_seq<A: de::SeqAccess<'x>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let start: i32 = try!(access.next_element().and_then(|x| x.ok_or_else(|| de::Error::invalid_length(0, &self))));
                let end: i32 = try!(access.next_element().and_then(|x| x.ok_or_else(|| de::Error::invalid_length(1, &self))));
                Ok(TextRange { start: start, end: end })
            }

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "an array of two integers")
            }
        }

        deserializer.deserialize_tuple(2, Visitor)
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
    pub display_url: Option<String>,
    pub expanded_url: Option<String>,
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
