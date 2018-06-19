#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub contributors_enabled: bool,
    pub created_at: CreatedAt,
    pub default_profile: bool,
    pub default_profile_image: bool,
    pub description: Option<String>,
    pub email: Option<String>,
    pub entities: Option<UserEntities>,
    pub favourites_count: u32,
    pub follow_request_sent: Option<bool>,
    pub followers_count: u32,
    pub friends_count: u32,
    pub has_extended_profile: Option<bool>,
    pub geo_enabled: bool,
    pub id: i64,
    pub is_translator: bool,
    pub is_translation_enabled: Option<bool>,
    pub lang: String,
    pub listed_count: u32,
    pub location: Option<String>,
    pub muting: Option<bool>,
    pub name: String,
    pub needs_phone_verification: Option<bool>,
    pub profile_background_color: String,
    pub profile_background_image_url: Option<String>,
    pub profile_background_image_url_https: Option<String>,
    pub profile_background_tile: bool,
    pub profile_banner_url: Option<String>,
    pub profile_image_url: String,
    pub profile_image_url_https: String,
    pub profile_link_color: String,
    pub profile_location: Option<Place>,
    pub profile_sidebar_border_color: String,
    pub profile_sidebar_fill_color: String,
    pub profile_text_color: String,
    pub profile_use_background_image: bool,
    pub protected: bool,
    pub screen_name: String,
    pub show_all_inline_media: Option<bool>,
    pub status: Option<Box<Tweet>>,
    pub statuses_count: u32,
    pub suspended: Option<bool>,
    pub time_zone: Option<String>,
    pub translator_type: Option<String>,
    pub url: Option<String>,
    pub utc_offset: Option<i32>,
    pub verified: bool,
    pub withheld_in_countries: Option<String>,
    pub withheld_scope: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorUsers {
    pub previous_cursor: i64,
    pub next_cursor: i64,
    pub users: Vec<User>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserCategory {
    pub name: String,
    pub slug: String,
    pub size: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestedUsers {
    pub name: String,
    pub slug: String,
    pub size: u32,
    pub users: Vec<User>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileBannerSizes {
    pub sizes: HashMap<String, ProfileBannerSize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileBannerSize {
    pub w: u32,
    pub h: u32,
    pub url: String,
}
