use std::borrow::Cow;
use std::io::Read;
use hyper::{Get, Post};
use ::TwitterResult;
use conn::*;
use models::*;
use self::helper::*;

mod helper;
mod impls;

include!(concat!(env!("OUT_DIR"), "/clients.rs"));
