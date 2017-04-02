//! The test Twitter client app

#![feature(alloc)]

extern crate alloc;
extern crate tweetust;

use std::fmt;
use std::fs;
use std::io;
use std::io::prelude::*;
use tweetust::*;

type Client<'a> = TwitterClient<OAuthAuthenticator<'a>, DefaultHttpHandler<conn::DefaultHttpsConnector>>;

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
        ("account", CmdAccount::new()),
        ("statuses", CmdStatuses::new()),
        ("upload_video", CmdUploadVideo),
        ("rate_limit_status", CmdRateLimitStatus),
        ("quit", CmdQuit),
        ("exit", CmdQuit),
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

const CONFIG_FILE: &'static str = "test_client_config.txt";
const CONSUMER_KEY: &'static str = "wDvwfgeq3mJO6GKTNXnOQvIf3";
const CONSUMER_SECRET: &'static str = "om5lZdHf9dbyQUEIdwtiz0HqeC83O5JQUV3Dc9Amk0HO7FB7Rs";

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

    Ok(TwitterClient::new(
        OAuthAuthenticator::new(CONSUMER_KEY, CONSUMER_SECRET, token, token_secret),
        DefaultHttpHandler::with_https_connector().unwrap()
    ))
}

fn authorize<'a>() -> Client<'a> {
    let handler = DefaultHttpHandler::with_https_connector().unwrap();
    let req_token = oauth::request_token(CONSUMER_KEY, CONSUMER_SECRET, "oob")
        .execute(&handler).unwrap().object;

    write_and_flush(format_args!("Go to https://api.twitter.com/oauth/authorize?oauth_token={}\nPut PIN: ", req_token.oauth_token));

    let mut pin = String::with_capacity(7);
    io::stdin().read_line(&mut pin).unwrap();

    let access_token = req_token.access_token(pin.trim())
        .execute(&handler).unwrap().object;

    {
        let mut file = fs::File::create(CONFIG_FILE).unwrap();
        write!(file, "{}\n{}\n", access_token.oauth_token, access_token.oauth_token_secret).unwrap();
    }

    TwitterClient::new(access_token.to_authenticator(), handler)
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

struct CmdAccount { cmds: CommandMap }
impl CmdAccount {
    fn new() -> CmdAccount {
        CmdAccount {
            cmds: cmds_map! {
                ("profileimage", CmdAccountProfileImage),
            }
        }
    }
}
impl Command for CmdAccount {
    fn run(&self, reader: InputReader, client: &Client) {
        self.cmds.run(reader, client);
    }
}

struct CmdAccountProfileImage;
impl Command for CmdAccountProfileImage {
    fn run(&self, mut reader: InputReader, client: &Client) {
        if let Some(file_name) = reader.next() {
            handle(
                fs::File::open(file_name),
                |mut f| handle(
                    client.account()
                        .update_profile_image(&mut f)
                        .tweet_mode(models::TweetMode::Extended)
                        .execute(),
                    |x| println!("{:?}", x)
                )
            );
        } else {
            println!("Usage: account profileimage \"file path\"");
        }
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

struct CmdRateLimitStatus;
impl Command for CmdRateLimitStatus {
    fn run(&self, _: InputReader, client: &Client) {
        handle(
            client.application().rate_limit_status().execute(),
            |x| println!("{:?}", x)
        );
    }
}

struct CmdUploadVideo;
impl Command for CmdUploadVideo {
    fn run(&self, _: InputReader, client: &Client) {
        fn core(client: &Client) -> Result<(), Box<std::error::Error>> {
            write_and_flush(format_args!("File: "));            

            let mut video_file = {
                let mut file_name = String::new();
                io::stdin().read_line(&mut file_name).unwrap();
                fs::File::open(file_name.trim())?
            };

            let file_len = video_file.metadata()?.len();

            let init_res = client.media().upload_init_command(file_len, "video/mp4")
                .media_category("tweet_video")
                .execute()?;

            print!("\n{:?}\n\n", init_res);

            {
                const BUF_SIZE: usize = 5 * 1000 * 1000; // 5MB
                let mut buf = unsafe { alloc::raw_vec::RawVec::with_capacity(BUF_SIZE).into_box() };

                for segment_index in 0.. {
                    let read_bytes = video_file.read(&mut buf)?;
                    if read_bytes == 0 { break; }
                    let mut buf_reader = io::Cursor::new(&buf[..read_bytes]);
                    println!("Uploading {} bytes", read_bytes);
                    client.media().upload_append_command(init_res.object.media_id, segment_index)
                        .media(&mut buf_reader)
                        .execute()?;
                }
            }

            let finalize_res = client.media().upload_finalize_command(init_res.object.media_id).execute()?;
            println!("\n{:?}", finalize_res);

            if let Some(models::ProcessingInfo { mut check_after_secs, .. }) = finalize_res.object.processing_info {
                while let Some(x) = check_after_secs {
                    std::thread::sleep(std::time::Duration::from_secs(x as u64));

                    let status_res = client.media().upload_status_command(init_res.object.media_id).execute()?;
                    println!("\n{:?}", status_res);

                    check_after_secs = status_res.object.processing_info.check_after_secs;
                }
            }

            write_and_flush(format_args!("\nTweet: "));
            let mut status = String::new();
            io::stdin().read_line(&mut status).unwrap();

            println!(
                "\n{:?}",
                client.statuses()
                    .update(status)
                    .media_ids(Some(init_res.object.media_id))
                    .execute()?
            );

            Ok(())
        }

        handle(core(client), |_| ());
    }
}
