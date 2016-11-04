# tweetust
[![crates.io](https://img.shields.io/crates/v/tweetust.svg)](https://crates.io/crates/tweetust)
[![Build Status](https://travis-ci.org/azyobuzin/tweetust.svg?branch=master)](https://travis-ci.org/azyobuzin/tweetust)

Twitter API wrapper for Rust.

# How to build from source code
1. Build clientgen/clientgen.csproj in MSBuild or XBuild
2. Run the built clientgen
3. `cargo build`

# Roadmap
- [x] Use serde
- [x] Use Cow to store string
- [x] Update oauthcli
- [x] &mut builders
- [x] Change the way to generate code: be pure Rust
- [ ] Parse created_at and Tweet.source (in serde_json?)
- [ ] Support new APIs
- [ ] Streaming
