use std;
use std::collections::HashMap;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use ::{conn, TwitterError, TwitterResult};

#[derive(Clone, Debug)]
pub struct RateLimitStatus {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64
}

#[derive(Clone, Debug)]
pub struct TwitterResponse<T> {
    pub object: T,
    pub raw_response: String,
    pub rate_limit: Option<RateLimitStatus>
}

impl TwitterResponse<()> {
    pub fn parse_to_object<T: ::serde::de::Deserialize>(self) -> TwitterResult<T> {
        match conn::parse_json(&self.raw_response) {
            Ok(x) => Ok(TwitterResponse {
                object: x,
                raw_response: self.raw_response,
                rate_limit: self.rate_limit
            }),
            Err(x) => Err(TwitterError::JsonError(x, self))
        }
    }
}

// https://serde.rs/enum-str.html
macro_rules! enum_str {
    ($name:ident { $($variant:ident($str:expr), )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub enum $name {
            $($variant,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::Serializer,
            {
                // Serialize the enum as a string.
                serializer.serialize_str(match *self {
                    $( $name::$variant => $str, )*
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
                            _ => Err(E::invalid_value(
                                &format!("unknown {} variant: {}",
                                stringify!($name), value))),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TweetMode {
    /// Compatibility mode
    Compat,
    /// Extended mode
    Extended
}

