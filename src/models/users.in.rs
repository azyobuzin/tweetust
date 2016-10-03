pub mod users {
    use super::entities::UserEntities;
    use super::tweets::Tweet;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct User {
        pub contributors_enabled: bool,
        pub created_at: String,
        pub default_profile: bool,
        pub default_profile_image: bool,
        pub description: Option<String>,
        pub entities: Option<UserEntities>,
        pub favourites_count: u32,
        pub follow_request_sent: Option<bool>,
        pub followers_count: u32,
        pub friends_count: u32,
        pub geo_enabled: bool,
        pub id: i64,
        pub is_translator: bool,
        pub lang: String,
        pub listed_count: u32,
        pub location: Option<String>,
        pub name: String,
        pub profile_background_color: String,
        pub profile_background_image_url: String,
        pub profile_background_image_url_https: String,
        pub profile_background_tile: bool,
        pub profile_banner_url: Option<String>,
        pub profile_image_url: String,
        pub profile_image_url_https: String,
        pub profile_link_color: String,
        pub profile_sidebar_border_color: String,
        pub profile_sidebar_fill_color: String,
        pub profile_text_color: String,
        pub profile_use_background_image: bool,
        pub protected: bool,
        pub screen_name: String,
        pub show_all_inline_media: Option<bool>,
        pub status: Option<Box<Tweet>>,
        pub statuses_count: u32,
        pub time_zone: Option<String>,
        pub url: Option<String>,
        pub utc_offset: Option<i32>,
        pub verified: bool,
        pub withheld_in_countries: Option<String>,
        pub withheld_scope: Option<String>,
        pub muting: Option<bool>
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CursorUsers {
        pub previous_cursor: i64,
        pub next_cursor: i64,
        pub users: Vec<User>
    }
}
