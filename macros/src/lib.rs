//! # How to use client!
//! ```rust
//! client!(
//!     StatusesClient,
//!     [ // APIs
//!         (
//!             show, // method name,
//!             Get, // HTTP method
//!             "https://api.twitter.com/1.1/statuses/show/{}.json",
//!             // required parameters
//!             [id: String, ...],
//!             // optional parameters
//!             [trim_user: bool, ...],
//!             Status // return type
//!         ),
//!         ...
//!     ]
//! );
//! ```
//! If the API endpoint has `"{}"`, inserts the first argument there.

#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![allow(unstable)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use syntax::ast::{self, TokenTree};
use syntax::ext::base::{DummyResult, ExtCtxt, MacResult};
use syntax::ext::quote::rt::ToSource;
use syntax::codemap::Span;
use syntax::parse::token;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("client", expand_client);
}

struct ApiDef {
    method_name: String,
    http_method: String,
    url: String,
    required_params: Vec<ast::Arg>,
    optional_params: Vec<ast::Arg>,
    return_type: String
}

pub fn expand_client(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut iter = args.iter();
    let client_name;
    match iter.next() {
        Some(tt) => match tt {
            &ast::TtToken(_, token::Ident(ident, token::Plain)) =>
                client_name = token::get_ident(ident).to_string(),
            _ => {
                cx.span_err(sp, "the first argument is not identifier");
                return DummyResult::any(sp);
            }
        },
        None => {
            cx.span_err(sp, "no argument");
            return DummyResult::any(sp);
        }
    }

    let mut deftts;
    match iter.next() {
        Some(&ast::TtDelimited(span, ref delim)) => match delim.delim {
            token::Bracket => deftts = delim.tts.clone().into_iter(),
            _ => {
                cx.span_err(span, "the second argument is not []");
                return DummyResult::any(sp);
            }
        },
        _ => {
            cx.span_err(sp, "invalid second argument");
            return DummyResult::any(sp);
        }
    }

    if iter.next().is_some() {
        cx.span_err(sp, "too many arguments");
        return DummyResult::any(sp);
    }

    let mut defs = Vec::new();
    for tt in deftts {
        let defspan;
        let tts;
        match tt {
            ast::TtDelimited(span, ref delim) => match delim.delim {
                token::Paren => {
                    defspan = span;
                    tts = delim.tts.clone();
                },
                _ => {
                    cx.span_err(span, "an API definition must be surrounded by ()");
                    return DummyResult::any(sp);
                }
            },
            ast::TtToken(span, _) | ast::TtSequence(span, _) => {
                cx.span_err(span, "invalid API definition");
                return DummyResult::any(sp);
            }
        }

        if tts.len() != 6 {
            cx.span_err(defspan, "an API definition requires 6 arguments");
            return DummyResult::any(sp);
        }

        let method_name;
        match tts[0] {
            ast::TtToken(_, token::Ident(ident, token::Plain)) =>
                method_name = token::get_ident(ident).to_string(),
            ast::TtToken(span, _)
            | ast::TtDelimited(span, _)
            | ast::TtSequence(span, _) => {
                cx.span_err(span, "a method name must be a plain ident");
                return DummyResult::any(sp);
            }
        }

        let http_method = cx.new_parser_from_tts(&[tts[1].clone()])
            .parse_expr().to_source();

        let url = cx.new_parser_from_tts(&[tts[2].clone()])
            .parse_expr().to_source();

        let mut required_params = Vec::new();
        match tts[3] {
            ast::TtDelimited(span, ref delim) => match delim.delim {
                token::Bracket => {
                    for tt in delim.tts.clone().into_iter() {
                        required_params.push(
                            cx.new_parser_from_tts(&[tt]).parse_arg()
                        );
                    }
                },
                _ => {
                    cx.span_err(span, "required parameters must be surrounded by []");
                    return DummyResult::any(sp);
                }
            },
            ast::TtToken(span, _) | ast::TtSequence(span, _) => {
                cx.span_err(span, "invalid required parameters");
                return DummyResult::any(sp);
            }
        }

        let mut optional_params = Vec::new();
        match tts[4] {
            ast::TtDelimited(span, ref delim) => match delim.delim {
                token::Bracket => {
                    for tt in delim.tts.clone().into_iter() {
                        optional_params.push(
                            cx.new_parser_from_tts(&[tt]).parse_arg()
                        );
                    }
                },
                _ => {
                    cx.span_err(span, "optional parameters must be surrounded by []");
                    return DummyResult::any(sp);
                }
            },
            ast::TtToken(span, _) | ast::TtSequence(span, _) => {
                cx.span_err(span, "invalid optional parameters");
                return DummyResult::any(sp);
            }
        }

        let return_type = cx.new_parser_from_tts(&[tts[5].clone()])
            .parse_ty().to_source();

        defs.push(ApiDef {
            method_name: method_name,
            http_method: http_method,
            url: url,
            required_params: required_params,
            optional_params: optional_params,
            return_type: return_type
        });
    }

    unimplemented!();
}
