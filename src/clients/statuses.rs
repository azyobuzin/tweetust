use hyper::{Get, Post};
use super::super::{conn, TwitterError, TwitterResult};
use super::super::models::status::Status;

client!(StatusesClient, [
    (
        mentions_timeline, Get,
        "https://api.twitter.com/1.1/statuses/mentions_timeline.json",
        [],
        [
            count: i32, since_id: u64, max_id: u64, trim_user: bool,
            contributor_details: bool, include_entities: bool
        ],
        Vec<Status>
    )
]);
