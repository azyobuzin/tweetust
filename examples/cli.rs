//! The test Twitter client app

extern crate tweetust;

use std::fs;
use std::io;
use std::io::prelude::*;
use tweetust::*;

type Client<'a> = TwitterClient<OAuthAuthenticator<'a>>;

fn main() {
    let client = create_client();
    // TODO
}

static CONFIG_FILE: &'static str = "test_client_config.txt";
static CONSUMER_KEY: &'static str = "wDvwfgeq3mJO6GKTNXnOQvIf3";
static CONSUMER_SECRET: &'static str = "om5lZdHf9dbyQUEIdwtiz0HqeC83O5JQUV3Dc9Amk0HO7FB7Rs";

fn create_client<'a>() -> Client<'a> {
    fs::File::open(CONFIG_FILE)
        .and_then(load_from_file)
        .unwrap_or_else(|_| authorize())
}

fn load_from_file<'a>(file: fs::File) -> io::Result<Client<'a>> {
    let mut lines = io::BufReader::new(file).lines();
    let mut read = || lines.next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "eof"))
        .and_then(|x| x);

    let token = try!(read());
    let token_secret = try!(read());

    Ok(TwitterClient::new(OAuthAuthenticator::new(CONSUMER_KEY, CONSUMER_SECRET, token, token_secret)))
}

fn authorize<'a>() -> Client<'a> {
    let req_token = oauth::request_token(CONSUMER_KEY, CONSUMER_SECRET, "oob").execute().unwrap().object;

    {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write!(handle, "Go to https://api.twitter.com/oauth/authorize?oauth_token={}\nPut PIN: ", req_token.oauth_token).unwrap();
        handle.flush().unwrap();
    }

    let mut pin = String::new();
    io::stdin().read_line(&mut pin).unwrap();

    let access_token = req_token.access_token(pin).execute().unwrap().object;

    {
        let mut file = fs::File::create(CONFIG_FILE).unwrap();
        write!(file, "{}\n{}\n", access_token.oauth_token, access_token.oauth_token_secret).unwrap();
    }

    TwitterClient::new(access_token.to_authenticator())
}
