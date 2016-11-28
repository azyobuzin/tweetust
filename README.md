# tweetust
[![crates.io](https://img.shields.io/crates/v/tweetust.svg)](https://crates.io/crates/tweetust)
[![Build Status](https://travis-ci.org/azyobuzin/tweetust.svg?branch=master)](https://travis-ci.org/azyobuzin/tweetust)

Twitter API wrapper for Rust.

# Roadmap
- [x] Use serde
- [x] Use Cow to store string
- [x] Update oauthcli
- [x] &mut builders
- [x] Change the way to generate code: be pure Rust
- [ ] Parse Tweet.source (in serde_json?)
- [ ] media API
- [ ] collections API
- [ ] Streaming

# メモ
## API定義オーバーライド
- 同じ名前のがあったらマージしていく的な方向でやっていきたい
- ~~マージするのはブロック単位（description とか params とか）で良い？~~
- エンドポイント単位でええやんな
- API を削除する方法（要検討）
