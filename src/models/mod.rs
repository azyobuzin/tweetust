use std;
use std::fmt;
use std::collections::HashMap;
use std::ops::Deref;
use chrono;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
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
    pub fn parse_to_object<T: de::Deserialize>(self) -> TwitterResult<T> {
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
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::Serializer,
            {
                // Serialize the enum as a string.
                serializer.serialize_str(match *self {
                    $( $name::$variant => $str, )*
                    $name::Other(ref x) => x.as_ref(),
                })
            }
        }

        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer,
            {
                struct Visitor;

                impl ::serde::de::Visitor for Visitor {
                    type Value = $name;

                    fn visit_str<E>(&mut self, value: &str) -> Result<$name, E>
                        where E: ::serde::de::Error,
                    {
                        match value {
                            $( $str => Ok($name::$variant), )*
                            x => Ok($name::Other(x.to_owned())),
                        }
                    }
                }

                // Deserialize the enum from a string.
                deserializer.deserialize_str(Visitor)
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/models/_models_list.rs"));

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
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl Deserialize for CreatedAt {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        struct Visitor;
        impl de::Visitor for Visitor {
            type Value = CreatedAt;
            fn visit_str<E: de::Error>(&mut self, v: &str) -> Result<Self::Value, E> {
                match chrono::DateTime::parse_from_str(v, CREATED_AT_FORMAT) {
                    Ok(x) => Ok(CreatedAt(x)),
                    Err(x) => Err(E::invalid_value(&x.to_string()))
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
