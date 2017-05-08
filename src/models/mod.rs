use std;
use std::fmt;
use std::collections::HashMap;
use std::ops::Deref;
use chrono;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeTuple;
use ::{parse_json, TwitterError, TwitterResult};

#[derive(Clone, Debug)]
pub struct TwitterResponse<T> {
    pub object: T,
    pub raw_response: String,
    pub rate_limit: Option<RateLimitStatus>,
}

#[derive(Clone, Debug)]
pub struct RawResponse {
    pub raw_response: String,
    pub rate_limit: Option<RateLimitStatus>,
}

impl RawResponse {
    pub fn parse_to_object<T: de::DeserializeOwned>(self) -> TwitterResult<T> {
        match parse_json(&self.raw_response) {
            Ok(x) => Ok(TwitterResponse {
                object: x,
                raw_response: self.raw_response,
                rate_limit: self.rate_limit,
            }),
            Err(x) => Err(TwitterError::ParseResponse(Some(x), self)),
        }
    }

    pub fn into_twitter_response(self) -> TwitterResponse<()> {
        TwitterResponse {
            object: (),
            raw_response: self.raw_response,
            rate_limit: self.rate_limit,
        }
    }
}

// https://serde.rs/enum-str.html
macro_rules! enum_str {
    ($name:ident { $($variant:ident($str:expr), )* }) => {
        #[derive(Clone, Debug, Eq, PartialEq, Hash)]
        pub enum $name {
            $($variant,)*
            Other(String)
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer,
            {
                // Serialize the enum as a string.
                serializer.serialize_str(match *self {
                    $( $name::$variant => $str, )*
                    $name::Other(ref x) => x.as_ref(),
                })
            }
        }

        impl<'x> ::serde::Deserialize<'x> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer<'x>,
            {
                struct Visitor;

                impl<'x> ::serde::de::Visitor<'x> for Visitor {
                    type Value = $name;

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                        where E: ::serde::de::Error,
                    {
                        match value {
                            $( $str => Ok($name::$variant), )*
                            x => Ok($name::Other(x.to_owned())),
                        }
                    }

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "a string")
                    }
                }

                // Deserialize the enum from a string.
                deserializer.deserialize_str(Visitor)
            }
        }
    }
}

include!("cursor.rs");
include!("direct_messages.rs");
include!("entities.rs");
include!("error.rs");
include!("friendships.rs");
include!("geo.rs");
include!("helps.rs");
include!("lists.rs");
include!("media.rs");
include!("places.rs");
include!("rate_limit.rs");
include!("search.rs");
include!("settings.rs");
include!("tweets.rs");
include!("users.rs");

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TweetMode {
    /// Compatibility mode
    Compat,
    /// Extended mode
    Extended
}

static CREATED_AT_FORMAT: &'static str = "%a %b %d %T %z %Y";

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct CreatedAt(pub chrono::DateTime<chrono::FixedOffset>);

impl Deref for CreatedAt {
    type Target = chrono::DateTime<chrono::FixedOffset>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for CreatedAt {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.format(CREATED_AT_FORMAT), f)
    }
}

impl Serialize for CreatedAt {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'x> Deserialize<'x> for CreatedAt {
    fn deserialize<D: Deserializer<'x>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'x> de::Visitor<'x> for Visitor {
            type Value = CreatedAt;

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match chrono::DateTime::parse_from_str(v, CREATED_AT_FORMAT) {
                    Ok(x) => Ok(CreatedAt(x)),
                    Err(_) => Err(E::invalid_value(de::Unexpected::Str(v), &self))
                }
            }

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a date/time string")
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
