use std::borrow::Cow;
use hyper::{Get, Post};
use super::TwitterClient;
use super::helper::*;
use ::TwitterResult;
use conn::*;
use models::*;

type Params<'a> = Vec<(Cow<'a, str>, ParameterValue<'a>)>;

pub fn account_settings<'a, A, H>(client: &TwitterClient<A, H>, params: Params<'a>) -> TwitterResult<AccountSettings>
    where A: Authenticator, H: HttpHandler
{
    let method = if params.is_empty() { Get } else { Post };
    execute_core(client, method, "https://api.twitter.com/1.1/account/settings.json", params)
}

pub fn media_upload<'a, A, H>(client: &TwitterClient<A, H>, params: Params<'a>) -> TwitterResult<MediaUploadResponse>
    where A: Authenticator, H: HttpHandler
{
    execute_core(client, Post, "https://upload.twitter.com/1.1/media/upload.json", params)
}
