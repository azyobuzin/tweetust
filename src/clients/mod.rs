use std::borrow::Cow;
use hyper::{Get, Post};
use ::TwitterResult;
use conn::{Authenticator, ParameterValue};
use models::*;
use self::helper::*;

mod helper;

include!(concat!(env!("OUT_DIR"), "/clients.rs"));
