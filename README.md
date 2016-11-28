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
## execute の処理内容
通常の API → 今まで通り
Impl API → params: Vec<_> の作成までやって、実際の実装をたたく

## API定義オーバーライド
- 同じ名前のがあったらマージしていく的な方向でやっていきたい
- マージするのはブロック単位（description とか params とか）で良い？
- 戻り値の型をオーバーライドしたいのもある
- API を削除する方法（要検討）
- Impl まだできてないのをスルーする方法（属性追加？）
