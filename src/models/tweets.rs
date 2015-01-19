use std::collections::BTreeMap;
use super::entities::{Entities, ExtendedEntities};
use super::places::Place;
use super::users::User;

#[derive(Clone, Show, RustcDecodable)]
pub struct Tweet {
    pub contributors: Option<Vec<Contributor>>,
    pub coordinates: Option<Coordinates>,
    pub created_at: String,
    pub current_user_retweet: Option<CurrentUserRetweet>,
    pub entities: Option<Entities>,
    pub extended_entities: Option<ExtendedEntities>,
    pub favorite_count: Option<u32>,
    pub favorited: Option<bool>,
    pub filter_level: Option<String>,
    pub id: u64,
    pub in_reply_to_screen_name: Option<String>,
    pub in_reply_to_status_id: Option<u64>,
    pub in_reply_to_user_id: Option<u64>,
    pub lang: Option<String>,
    pub place: Option<Place>,
    pub possibly_sensitive: Option<bool>,
    //pub scopes: Option<BTreeMap<String, json::Json>>,
    pub retweet_count: u32,
    pub retweeted: Option<bool>,
    pub retweeted_status: Option<Box<Tweet>>,
    pub source: String,
    pub text: String,
    pub user: User,
    pub withheld_copyright: Option<bool>,
    pub withheld_in_countries: Option<Vec<String>>,
    pub withheld_scope: Option<String>
}

impl PartialEq for Tweet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Show, RustcDecodable)]
pub struct Contributor {
    pub id: u64,
    pub screen_name: String
}

#[derive(Clone, Show, RustcDecodable)]
pub struct Coordinates {
    pub coordinates: Vec<f64>,
    pub type_: String
}

#[derive(Clone, Copy, Show, RustcDecodable)]
pub struct CurrentUserRetweet {
    pub id: u64
}

#[derive(Clone, Show, RustcDecodable)]
pub struct LookupMap {
    pub id: BTreeMap<String, Option<Tweet>>
}
