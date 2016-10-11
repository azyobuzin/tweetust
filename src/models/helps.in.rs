#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub characters_reserved_per_media: u32,
    pub dm_text_character_limit: u32,
    pub max_media_per_upload: u32,
    pub non_username_paths: Vec<String>,
    pub photo_size_limit: u32,
    pub short_url_length: u32,
    pub short_url_length_https: u32,
    pub photo_sizes: MediaSizes
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub status: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivacyResponse {
    pub privacy: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TosResponse {
    pub tos: String
}
