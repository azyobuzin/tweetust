use hyper::{Get, Post};
use models::CursorIds;
use models::friendships::{Connections, FriendshipResponse};
use models::users::User;

client!(FriendshipsClient, [
    (
        no_retweets_ids, Get,
        "https://api.twitter.com/1.1/friendships/no_retweets/ids.json",
        [], [],
        Vec<u64>
    ),
    (
        incoming, Get,
        "https://api.twitter.com/1.1/friendships/incoming.json",
        [],
        [cursor: i64],
        CursorIds
    ),
    (
        outgoing, Get,
        "https://api.twitter.com/1.1/friendships/outgoing.json",
        [],
        [cursor: i64],
        CursorIds
    ),
    (
        create, Post,
        "https://api.twitter.com/1.1/friendships/create.json",
        [],
        [screen_name: String, user_id: u64, follow: bool],
        User
    ),
    (
        destroy, Post,
        "https://api.twitter.com/1.1/friendships/destroy.json",
        [],
        [screen_name: String, user_id: u64],
        User
    ),
    (
        update, Post,
        "https://api.twitter.com/1.1/friendships/update.json",
        [],
        [screen_name: String, user_id: u64, device: bool, retweets: bool],
        FriendshipResponse
    ),
    (
        show, Get,
        "https://api.twitter.com/1.1/friendships/show.json",
        [],
        [
            source_id: u64, source_screen_name: String,
            target_id: u64, target_screen_name: String
        ],
        FriendshipResponse
    ),
    (
        lookup, Get,
        "https://api.twitter.com/1.1/friendships/lookup.json",
        [],
        [screen_name: Vec<String>, user_id: Vec<u64>],
        Vec<Connections>
    )
]);
