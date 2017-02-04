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

const MEDIA_UPLOAD_URL: &'static str = "https://upload.twitter.com/1.1/media/upload.json";

pub fn media_upload<'a, A, H>(client: &TwitterClient<A, H>, params: Params<'a>) -> TwitterResult<MediaUploadResponse>
    where A: Authenticator, H: HttpHandler
{
    execute_core(client, Post, MEDIA_UPLOAD_URL, params)
}

pub fn media_upload_init_command<'a, A, H>(client: &TwitterClient<A, H>, mut params: Params<'a>) -> TwitterResult<UploadInitCommandResponse>
    where A: Authenticator, H: HttpHandler
{
    params.push((Cow::Borrowed("command"), ParameterValue::Text(Cow::Borrowed("INIT"))));
    execute_core(client, Post, MEDIA_UPLOAD_URL, params)
}

pub fn media_upload_append_command<'a, A, H>(client: &TwitterClient<A, H>, mut params: Params<'a>) -> TwitterResult<()>
    where A: Authenticator, H: HttpHandler
{
    params.push((Cow::Borrowed("command"), ParameterValue::Text(Cow::Borrowed("APPEND"))));
    execute_core(client, Post, MEDIA_UPLOAD_URL, params)
}

pub fn media_upload_finalize_command<'a, A, H>(client: &TwitterClient<A, H>, mut params: Params<'a>) -> TwitterResult<UploadFinalizeCommandResponse>
    where A: Authenticator, H: HttpHandler
{
    params.push((Cow::Borrowed("command"), ParameterValue::Text(Cow::Borrowed("FINALIZE"))));
    execute_core(client, Post, MEDIA_UPLOAD_URL, params)
}

pub fn media_upload_status_command<'a, A, H>(client: &TwitterClient<A, H>, mut params: Params<'a>) -> TwitterResult<UploadFinalizeCommandResponse>
    where A: Authenticator, H: HttpHandler
{
    params.push((Cow::Borrowed("command"), ParameterValue::Text(Cow::Borrowed("STATUS"))));
    execute_core(client, Get, MEDIA_UPLOAD_URL, params)
}
