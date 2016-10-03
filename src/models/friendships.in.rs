pub mod friendships {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Connections {
        pub name: String,
        pub screen_name: String,
        pub id: i64,
        pub connections: Vec<String>
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Source {
        pub id: i64,
        pub screen_name: String,
        pub following: bool,
        pub followed_by: bool,
        pub following_received: Option<bool>,
        pub following_requested: Option<bool>,
        pub notifications_enabled: Option<bool>,
        pub can_dm: bool,
        pub blocking: Option<bool>,
        pub blocked_by: Option<bool>,
        pub muting: Option<bool>,
        pub want_retweets: Option<bool>,
        pub all_replies: Option<bool>,
        pub marked_spam: Option<bool>
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Target {
        pub id: i64,
        pub screen_name: String,
        pub following: bool,
        pub followed_by: bool,
        pub following_received: Option<bool>,
        pub following_requested: Option<bool>
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Relationship {
        pub target: Target,
        pub source: Source
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct FriendshipResponse {
        pub relationship: Relationship
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Friendship {
        pub id: i64,
        pub screen_name: String,
        pub name: String,
        pub connections: Vec<String>
    }
}
