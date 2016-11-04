use std::borrow::Cow;
use hyper::{Get, Post};
use ::TwitterResult;
use conn::{Authenticator, Parameter};
use models::*;
use self::helper::*;

mod helper;

include!(concat!(env!("OUT_DIR"), "/clients.rs"));
