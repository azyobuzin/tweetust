//! # How to use client!
//! TODO: write me

#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![allow(unstable)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use syntax::ast::{TokenTree, TtToken};
use syntax::ext::base::{DummyResult, ExtCtxt, MacResult};
use syntax::codemap::Span;
use syntax::parse::token;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("client", expand_client);
}

fn expand_client(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut iter = args.iter();
    let client_name;
    match iter.next() {
        Some(tt) => match tt {
            &TokenTree::TtToken(span, ref token) => match token {
                &token::Ident(ident, _) => client_name = token::get_ident(ident).to_string(),
                _ => {
                    cx.span_err(span, "the first argument is not identifier");
                    return DummyResult::any(sp);
                }
            },
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
    
    unimplemented!();
}
