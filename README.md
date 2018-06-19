# tweetust
[![crates.io](https://img.shields.io/crates/v/tweetust.svg)](https://crates.io/crates/tweetust)
[![Build Status](https://travis-ci.org/azyobuzin/tweetust.svg?branch=master)](https://travis-ci.org/azyobuzin/tweetust)

Twitter API wrapper for Rust.

# Roadmap
- [ ] Parse Tweet.source (in serde_json?)
- [x] media API
    - [ ] media/metadata/create
- [ ] collections API
- [ ] Streaming
- [ ] `jsonmap` element in API definition files
    - direct_messages/events
- [ ] Support hyper 0.11 (I wonder it is very hard...)

# How to build

This project generates code using the [CoreTweet API Templates](https://github.com/CoreTweet/CoreTweet/tree/master/ApiTemplates).
CoreTweet is included as a submodule so:

```
git clone --recursive git@github.com:azyobuzin/tweetust.git
```

...before `cargo build`.

The tests use `#![feature(alloc)]` which is only available on the nightly channel:

```
rustup default nightly
cargo test
```
