extern crate tweetust;
use std::io::stdin;

fn main() {
    let req_token = tweetust::oauth::request_token("wDvwfgeq3mJO6GKTNXnOQvIf3", "om5lZdHf9dbyQUEIdwtiz0HqeC83O5JQUV3Dc9Amk0HO7FB7Rs", "oob")
        .execute().unwrap().object;
    println!("Go to https://api.twitter.com/oauth/authorize?oauth_token={}", req_token.oauth_token);

    println!("Put PIN");
    let mut pin = String::new();
    stdin().read_line(&mut pin).unwrap();

    let access_token = req_token.access_token(&pin[..]).execute().unwrap().object;
    let client = tweetust::TwitterClient::new(access_token.to_authenticator());

    println!("Put your tweet");
    let mut status = String::new();
    stdin().read_line(&mut status).unwrap();

    let response = client.statuses().update(&status[..]).execute().unwrap();
    println!("{:?}", response);
}
