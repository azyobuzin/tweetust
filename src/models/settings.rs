#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountSettings {
    pub allow_contributor_request: String,
    pub allow_dms_from: String,
    pub allow_dm_groups_from: String,
    pub always_use_https: bool,
    pub discoverable_by_email: bool,
    pub discoverable_by_mobile_phone: bool,
    pub display_sensitive_media: bool,
    pub geo_enabled: bool,
    pub language: String,
    pub protected: bool,
    pub screen_name: String,
    pub sleep_time: SleepTime,
    pub smart_mute: bool,
    pub time_zone: TimeZone,
    pub translator_type: String,
    pub trend_location: Vec<TrendPlace>,
    pub use_cookie_personalization: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SleepTime {
    pub enabled: bool,
    pub end_time: Option<i32>,
    pub start_time: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeZone {
    pub name: String,
    pub tzinfo_name: String,
    pub utc_offset: i32,
}
