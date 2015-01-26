use std::fmt;
use hyper::Get;
use models::search::SearchResponse;

paramenum! ResultType { mixed, recent, popular }

#[derive(Clone, Copy, Debug)]
pub struct UntilDate {
    /// Years
    pub y: i32,
    /// Months
    pub m: i32,
    /// Days
    pub d: i32
}

impl UntilDate {
    pub fn new(y: i32, m: i32, d: i32) -> UntilDate {
        UntilDate { y: y, m: m, d: d }
    }
}

impl fmt::Display for UntilDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
}

client!(SearchClient, [
    (
        tweets, Get,
        "https://api.twitter.com/1.1/search/tweets.json",
        [q: String],
        [
            geocode: String, lang: String, locale: String,
            result_type: ResultType, count: i32, until: UntilDate,
            since_id: u64, max_id: u64, include_entities: bool
        ],
        SearchResponse
    )
]);
