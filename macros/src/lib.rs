//! # How to use client!
//! ```ignore
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
#![allow(unstable, unused_must_use)]

extern crate rustc;
extern crate syntax;

use std::fmt::Writer;
use rustc::plugin::Registry;
use syntax::ast::{self, TokenTree};
use syntax::ext::base::{ExtCtxt, MacItems, MacResult};
use syntax::ext::quote::rt::{ExtParseUtils, ToSource};
use syntax::codemap::Span;
use syntax::parse::common::SeqSep;
use syntax::parse::token;

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

pub fn expand_client(cx: &mut ExtCtxt, _: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut p = cx.new_parser_from_tts(args);

    let client_name = p.parse_ident().to_string();
    p.expect(&token::Comma);
    p.expect(&token::OpenDelim(token::Bracket));

    let defs = p.parse_seq_to_end(
        &token::CloseDelim(token::Bracket),
        SeqSep {
            sep: Some(token::Comma),
            trailing_sep_allowed: true
        },
        |p| {
            p.expect(&token::OpenDelim(token::Paren));

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
            p.expect(&token::CloseDelim(token::Paren));

            ApiDef {
                method_name: method_name,
                request_struct_name: request_struct_name,
                http_method: http_method,
                url: url,
                required_params: required_params,
                optional_params: optional_params,
                return_type: return_type
            }
        }
    );

    // client struct, client impl, API struct, API impl
    let mut items = Vec::with_capacity(defs.len() * 2 + 2);

    items.push(cx.parse_item(format!(
        "pub struct {}<T: ::conn::Authenticator>(pub ::std::rc::Rc<T>);",
        client_name
    )));

    let mut client_impl = format!(
        "impl<T: ::conn::Authenticator> {}<T> {{\n",
        client_name
    );
    for ref def in defs.iter() {
        write!(&mut client_impl, "pub fn {}(&self, ", def.method_name);
        for ref p in def.required_params.iter() {
            if p.ty.to_source() == "String" {
                write!(&mut client_impl, "{}: &str, ", p.pat.to_source());
            } else {
                write!(&mut client_impl, "{}, ", p.to_source());
            }
        }
        writeln!(&mut client_impl,
            ") -> {0}<T> {{\n{0} {{\n_auth: self.0.clone(),", def.request_struct_name);
        for ref p in def.required_params.iter() {
            write!(&mut client_impl, "{0}: {0}", p.pat.to_source());
            client_impl.push_str(
                if p.ty.to_source() == "String" { ".to_string(),\n" }
                else { ",\n" }
            );
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
            "pub struct {}<T: ::conn::Authenticator> {{\n_auth: ::std::rc::Rc<T>,\n",
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
            "impl<T: ::conn::Authenticator> {}<T> {{\n",
            def.request_struct_name
        );
        for ref p in def.optional_params.iter() {
            let ty = p.ty.to_source();
            let is_str = ty == "String";
            writeln!(&mut request_impl, "pub fn {0}(mut self, val: {1}) -> Self {{
self.{0} = Some(val{2});\nself\n}}",
                p.pat.to_source(),
                if is_str { "&str" } else { ty.as_slice() },
                if is_str { ".to_string()" } else { "" }
            );
        }
        writeln!(&mut request_impl, "pub fn execute(&self) -> ::TwitterResult<{}> {{
let mut params: Vec<::conn::Parameter> = Vec::with_capacity({});",
            def.return_type, def.required_params.len() + def.optional_params.len()
        );
        let need_format = def.url.as_slice().contains("{}");
        let mut reqparam_iter = def.required_params.iter();
        if need_format { reqparam_iter.next(); }
        for ref p in reqparam_iter {
            writeln!(&mut request_impl,
                "params.push(::conn::ToParameter::to_parameter(self.{0}.clone(), \"{0}\"));",
                p.pat.to_source()
            );
        }
        for ref p in def.optional_params.iter() {
            writeln!(&mut request_impl, "match self.{0} {{
    Some(ref x) => params.push(::conn::ToParameter::to_parameter(x, \"{0}\")),
    None => ()\n}}",
                p.pat.to_source()
            );
        }
        request_impl.push_str("let url = ");
        if need_format {
            write!(&mut request_impl, "format!({}, self.{})",
                def.url, def.required_params[0].pat.to_source());
        } else {
            request_impl.push_str(def.url.as_slice());
        }
        write!(&mut request_impl, ";
let res = try!(::conn::Authenticator::request_twitter(
    &*self._auth, {}, url{}, params.as_slice()));
match ::conn::parse_json(res.raw_response.as_slice()) {{
    Ok(j) => Ok(res.object({}j)),
    Err(e) => Err(::TwitterError::JsonError(e, res))
}}\n}}\n}}",
            def.http_method,
            if need_format { ".as_slice()" } else { "" },
            if def.return_type.as_slice().contains("Box<") { "box " } else { "" }
        );
        items.push(cx.parse_item(request_impl));
    }

    MacItems::new(items.into_iter())
}
