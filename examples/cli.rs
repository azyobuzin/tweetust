//! The test Twitter client app

extern crate tweetust;

use std::fmt;
use std::fs;
use std::io;
use std::io::prelude::*;
use tweetust::*;

type Client<'a> = TwitterClient<OAuthAuthenticator<'a>, conn::DefaultHttpHandler>;

macro_rules! cmds_map {
    ($(($name:expr, $cmd:expr),)*) => {{
        let mut cmds = CommandMap::new();
        $(cmds.add($name, Box::new($cmd));)*
        cmds
    }}
}

fn main() {
    let client = create_client();

    let cmds = cmds_map! {
        ("statuses", CmdStatuses::new()),
        ("quit", CmdQuit),
    };

    let mut buf = String::new();

    loop {
        write_and_flush(format_args!("> "));
        buf.clear();
        io::stdin().read_line(&mut buf).unwrap();
        cmds.run(InputReader::new(&buf), &client);
        print!("\n");
    }
}

fn write_and_flush(fmt: std::fmt::Arguments) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_fmt(fmt).unwrap();
    handle.flush().unwrap();
}

static CONFIG_FILE: &'static str = "test_client_config.txt";
static CONSUMER_KEY: &'static str = "wDvwfgeq3mJO6GKTNXnOQvIf3";
static CONSUMER_SECRET: &'static str = "om5lZdHf9dbyQUEIdwtiz0HqeC83O5JQUV3Dc9Amk0HO7FB7Rs";

fn create_client<'a>() -> Client<'a> {
    load_config_file().unwrap_or_else(|_| authorize())
}

fn load_config_file<'a>() -> io::Result<Client<'a>> {
    let file = try!(fs::File::open(CONFIG_FILE));
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

    write_and_flush(format_args!("Go to https://api.twitter.com/oauth/authorize?oauth_token={}\nPut PIN: ", req_token.oauth_token));

    let mut pin = String::with_capacity(7);
    io::stdin().read_line(&mut pin).unwrap();

    let access_token = req_token.access_token(pin.trim()).execute().unwrap().object;

    {
        let mut file = fs::File::create(CONFIG_FILE).unwrap();
        write!(file, "{}\n{}\n", access_token.oauth_token, access_token.oauth_token_secret).unwrap();
    }

    TwitterClient::new(access_token.to_authenticator())
}

struct InputReader<'a> {
    input: &'a str
}

impl<'a> InputReader<'a> {
    fn new(input: &'a str) -> InputReader<'a> {
        InputReader { input: input.trim_left() }
    }
}

impl<'a> std::iter::Iterator for InputReader<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let input = self.input;

        let (result, new_input) = match input.chars().next() {
            None => (None, input),
            Some(d @ '"') | Some(d @ '\'') => {
                let s = &input[1..];
                match s.find(d) {
                    Some(i) => (Some(&s[..i]), s[i + 1..].trim_left()),
                    None => (Some(s), &input[0..0])
                }
            },
            Some(_) => match input.find(|c: char| c.is_whitespace()) {
                Some(i) => (Some(&input[..i]), input[i + 1..].trim_left()),
                None => (Some(input), &input[0..0])
            }
        };

        self.input = new_input;
        result
    }
}

trait Command {
    fn run(&self, reader: InputReader, client: &Client);
}

struct CommandMap {
    map: Vec<(&'static str, Box<Command>)>
}

impl CommandMap {
    fn new() -> CommandMap {
        CommandMap { map: Vec::new() }
    }

    fn add(&mut self, name: &'static str, cmd: Box<Command>) {
        self.map.push((name, cmd));
    }

    fn run(&self, mut reader: InputReader, client: &Client) {
        let cmd = reader.next().unwrap_or("");
        if let Some(&(_, ref cmd)) = self.map.iter().filter(|&&(name, _)| name == cmd).nth(0) {
            cmd.run(reader, client);
        } else {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            writeln!(handle, "Available commands").unwrap();
            for &(name, _) in self.map.iter() {
                writeln!(handle, "  {}", name).unwrap();
            }
        }
    }
}

struct CmdQuit;
impl Command for CmdQuit {
    fn run(&self, _: InputReader, _: &Client) {
        std::process::exit(0);
    }
}

fn handle<O, E, F>(result: Result<O, E>, action: F)
    where E: fmt::Debug, F: FnOnce(O)
{
    match result {
        Ok(x) => action(x),
        Err(x) => println!("Error\n{:?}", x)
    }
}

struct CmdStatuses { cmds: CommandMap }
impl CmdStatuses {
    fn new() -> CmdStatuses {
        CmdStatuses {
            cmds: cmds_map! {
                ("update", CmdStatusesUpdate),
                ("show", CmdStatusesShow),
            }
        }
    }
}
impl Command for CmdStatuses {
    fn run(&self, reader: InputReader, client: &Client) {
        self.cmds.run(reader, client);
    }
}

struct CmdStatusesUpdate;
impl Command for CmdStatusesUpdate {
    fn run(&self, mut reader: InputReader, client: &Client) {
        if let Some(status) = reader.next() {
            handle(
                client.statuses()
                    .update(status)
                    .tweet_mode(models::TweetMode::Extended)
                    .execute(),
                |x| println!("{:?}", x)
            );
        } else {
            println!("Usage: statuses update \"tweet\"");
        }
    }
}

struct CmdStatusesShow;
impl Command for CmdStatusesShow {
    fn run(&self, mut reader: InputReader, client: &Client) {
        if let Some(id) = reader.next().and_then(|x| x.parse().ok()) {
            handle(
                client.statuses()
                    .show(id)
                    .tweet_mode(models::TweetMode::Extended)
                    .execute(),
                |x| println!("{:?}", x)
            );
        } else {
            println!("Usage: statuses show id");
        }
    }
}
