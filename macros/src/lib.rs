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

use std::fmt::Writer;
use rustc::plugin::Registry;
use syntax::ast::{self, TokenTree};
use syntax::ext::base::{DummyResult, ExtCtxt, MacItems, MacResult};
use syntax::ext::quote::rt::{ExtParseUtils, ToSource};
use syntax::codemap::Span;
use syntax::parse::common::SeqSep;
use syntax::parse::token;
use syntax::parse::parser::Parser;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("client", expand_client);
}

struct ApiDef {
    method_name: String,
    request_struct_name: String,
    http_method: String,
    url: String,
    required_params: Vec<ast::Arg>,
    optional_params: Vec<ast::Arg>,
    return_type: String
}

pub fn expand_client(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let client_name;
    let deftts;
    match args {
        [
            ast::TtToken(_, token::Ident(client_name_id, token::Plain)),
            ast::TtToken(_, token::Comma),
            ast::TtDelimited(_, ref delim)
        ] if delim.delim == token::Bracket => {
            client_name = client_name_id.to_string();
            deftts = delim.tts.clone();
        }
        _ => {
            cx.span_err(sp, "invalid arguments");
            cx.span_note(sp, format!("{:?}", args).as_slice());
            return DummyResult::any(sp);
        }
    }

    let mut defs = Vec::with_capacity(deftts.len());
    for tt in deftts.into_iter() {
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

        let mut p = cx.new_parser_from_tts(tts.as_slice());

        let method_name = p.parse_ident().to_string();
        let mut request_struct_name = String::with_capacity(method_name.len() + 14);
        let mut is_next_upper = true;
        for c in method_name.as_slice().chars() {
            if c == '_' {
                is_next_upper = true;
            } else {
                request_struct_name.push(
                    if is_next_upper {
                        is_next_upper = false;
                        c.to_uppercase()
                    } else {
                        c
                    }
                );
            }
        }
        request_struct_name.push_str("RequestBuilder");

        p.expect(&token::Comma);
        let http_method = p.parse_expr().to_source();

        p.expect(&token::Comma);
        let url = p.parse_str().0.get().to_source();

        p.expect(&token::Comma);
        p.expect(&token::OpenDelim(token::Bracket));
        let required_params = p.parse_seq_to_end(
            &token::CloseDelim(token::Bracket),
            SeqSep {
                sep: Some(token::Comma),
                trailing_sep_allowed: true
            },
            |p| p.parse_arg()
        );

        p.expect(&token::Comma);
        p.expect(&token::OpenDelim(token::Bracket));
        let optional_params = p.parse_seq_to_end(
            &token::CloseDelim(token::Bracket),
            SeqSep {
                sep: Some(token::Comma),
                trailing_sep_allowed: true
            },
            |p| p.parse_arg()
        );

        p.expect(&token::Comma);
        let return_type = p.parse_ty().to_source();
        p.eat(&token::Comma);

        defs.push(ApiDef {
            method_name: method_name,
            request_struct_name: request_struct_name,
            http_method: http_method,
            url: url,
            required_params: required_params,
            optional_params: optional_params,
            return_type: return_type
        });
    }

    // client struct, client impl, API struct, API impl
    let mut items = Vec::with_capacity(defs.len() * 2 + 2);

    items.push(cx.parse_item(format!(
        "pub struct {}<T: conn::Authenticator>(pub ::std::rc::Rc<T>);",
        client_name
    )));

    let mut client_impl = format!(
        "impl<T: conn::Authenticator> {}<T> {{\n",
        client_name
    );
    for ref def in defs.iter() {
        write!(&mut client_impl, "pub fn {}(&self, ", def.method_name);
        for ref p in def.required_params.iter() {
            write!(&mut client_impl, "{}, ", p.to_source());
        }
        writeln!(&mut client_impl,
            ") -> {0}<T> {{\n{0} {{\n_auth: self.0.clone(),", def.request_struct_name);
        for ref p in def.required_params.iter() {
            writeln!(&mut client_impl, "{0}: {0},", p.pat.to_source());
        }
        for ref p in def.optional_params.iter() {
            writeln!(&mut client_impl, "{}: None,", p.pat.to_source());
        }
        client_impl.push_str("}\n}\n");
    }
    client_impl.push('}');
    items.push(cx.parse_item(client_impl));

    for ref def in defs.iter() {
        let mut request_struct = format!(
            "pub struct {}<T: conn::Authenticator> {{\n_auth: ::std::rc::Rc<T>,\n",
            def.request_struct_name
        );
        for ref p in def.required_params.iter() {
            writeln!(&mut request_struct, "{},", p.to_source());
        }
        for ref p in def.optional_params.iter() {
            writeln!(&mut request_struct, "{}: Option<{}>,",
                p.pat.to_source(), p.ty.to_source()
            );
        }
        request_struct.push('}');
        items.push(cx.parse_item(request_struct));

        let mut request_impl = format!(
            "impl<T: conn::Authenticator> {}<T> {{\n",
            def.request_struct_name
        );
        for ref p in def.optional_params.iter() {
            writeln!(&mut request_impl, "pub fn {0}(mut self, val: {1}) -> Self {{
self.{0} = Some(val);\nself\n}}",
                p.pat.to_source(),
                p.ty.to_source()
            );
        }
        writeln!(&mut request_impl, "pub fn execute(&self) -> TwitterResult<::std::boxed::Box<{}>> {{
let mut params: Vec<conn::Parameter> = Vec::with_capacity({});",
            def.return_type, def.required_params.len() + def.optional_params.len()
        );
        for ref p in def.required_params.iter() {
            writeln!(&mut request_impl,
                "params.push(conn::ToParameter::to_parameter(self.{0}, \"{0}\"));",
                p.pat.to_source()
            );
        }
        for ref p in def.optional_params.iter() {
            writeln!(&mut request_impl, "match self.{0} {{
    Some(x) => params.push(conn::ToParameter::to_parameter(x, \"{0}\")),
    None => ()\n}}",
                p.pat.to_source()
            );
        }
        write!(&mut request_impl, "let url = {};
let result = conn::read_to_twitter_result(
    conn::Authenticator::send_request(&*self._auth, {}, url, params.as_slice())
);
match result {{
    Ok(res) => match ::rustc_serialize::json::decode(res.raw_response.as_slice()) {{
        Ok(j) => Ok(res.object(box j)),
        Err(e) => Err(TwitterError::JsonError(e, res))
    }},
    Err(e) => Err(e)
}}\n}}\n}}",
            def.url,
            def.http_method
        );
        items.push(cx.parse_item(request_impl));
    }

    MacItems::new(items.into_iter())
}
