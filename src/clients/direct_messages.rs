use hyper::{Get, Post};
use models::direct_messages::DirectMessage;

client!(DirectMessagesClient, [
    (
        sent, Get,
        "https://api.twitter.com/1.1/direct_messages/sent.json",
        [],
        [
            since_id: u64, max_id: u64, count: i32, page: i32,
            include_entities: bool
        ],
        Vec<DirectMessage>
    ),
    (
        show, Get,
        "https://api.twitter.com/1.1/direct_messages/show.json",
        [id: u64],
        [],
        DirectMessage
    ),
    (
        direct_messages, Get,
        "https://api.twitter.com/1.1/direct_messages.json",
        [],
        [
            since_id: u64, max_id: u64, count: i32, include_entities: bool,
            skip_status: bool
        ],
        Vec<DirectMessage>
    ),
    (
        destroy, Post,
        "https://api.twitter.com/1.1/direct_messages/destroy.json",
        [id: u64],
        [include_entities: bool],
        DirectMessage
    ),
    (
        new, Post,
        "https://api.twitter.com/1.1/direct_messages/new.json",
        [text: String],
        [user_id: u64, screen_name: String],
        DirectMessage
    )
]);
