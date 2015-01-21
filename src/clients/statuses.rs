use hyper::{Get, Post};
use models::CursorIds;
use models::tweets::{LookupMap, OEmbed, Tweet};

paramenum!(Align { left, right, center, none });

client!(StatusesClient, [
    (
        mentions_timeline, Get,
        "https://api.twitter.com/1.1/statuses/mentions_timeline.json",
        [],
        [
            count: i32, since_id: u64, max_id: u64, trim_user: bool,
            contributor_details: bool, include_entities: bool
        ],
        Vec<Tweet>
    ),
    (
        user_timeline, Get,
        "https://api.twitter.com/1.1/statuses/user_timeline.json",
        [],
        [
            user_id: u64, screen_name: String, since_id: u64, count: i32,
            max_id: u64, trim_user: bool, exclude_replies: bool,
            contributor_details: bool, include_rts: bool
        ],
        Vec<Tweet>
    ),
    (
        home_timeline, Get,
        "https://api.twitter.com/1.1/statuses/home_timeline.json",
        [],
        [
            count: i32, since_id: u64, max_id: u64, trim_user: bool,
            exclude_replies: bool, contributor_details: bool,
            include_entities: bool
        ],
        Vec<Tweet>
    ),
    (
        retweets_of_me, Get,
        "https://api.twitter.com/1.1/statuses/retweets_of_me.json",
        [],
        [
            count: i32, since_id: u64, max_id: u64, trim_user: bool,
            include_entities: bool, include_user_entities: bool
        ],
        Vec<Tweet>
    ),
    (
        retweets, Get,
        "https://api.twitter.com/1.1/statuses/retweets/{}.json",
        [id: u64],
        [count: i32, trim_user: bool],
        Vec<Tweet>
    ),
    (
        show, Get,
        "https://api.twitter.com/1.1/statuses/show.json",
        [id: u64],
        [trim_user: bool, include_my_retweet: bool, include_entities: bool],
        Box<Tweet>
    ),
    (
        destroy, Post,
        "https://api.twitter.com/1.1/statuses/destroy/{}.json",
        [id: u64],
        [trim_user: bool],
        Box<Tweet>
    ),
    (
        update, Post,
        "https://api.twitter.com/1.1/statuses/update.json",
        [status: String],
        [
            in_reply_to_status_id: u64, possibly_sensitive: bool, lat: f64,
            long: f64, place_id: String, display_coordinates: bool,
            trim_user: bool, media_ids: String
        ],
        Box<Tweet>
    ),
    (
        retweet, Post,
        "https://api.twitter.com/1.1/statuses/retweet/{}.json",
        [id: u64],
        [trim_user: bool],
        Box<Tweet>
    ),
    (
        oembed, Get,
        "https://api.twitter.com/1.1/statuses/oembed.json",
        [],
        [
            id: u64, url: String, maxwidth: i32, hide_media: bool,
            hide_thread: bool, omit_script: bool, align: Align,
            related: String, lang: String
        ],
        Box<OEmbed>
    ),
    (
        retweeters_ids, Get,
        "https://api.twitter.com/1.1/statuses/retweeters/ids.json",
        [id: u64],
        [cursor: i64],
        CursorIds
    ),
    (
        lookup, Get,
        "https://api.twitter.com/1.1/statuses/lookup.json",
        [id: String],
        [include_entities: bool, trim_user: bool],
        Vec<Tweet>
    ),
    (
        lookup_map, Get,
        "https://api.twitter.com/1.1/statuses/lookup.json?map=true",
        [id: String],
        [include_entities: bool, trim_user: bool],
        LookupMap
    )
]);
