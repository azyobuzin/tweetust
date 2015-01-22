//! # client!
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
//!
//! #paramenum!
//! Make Pascal Case enum members and implement fmt::String.
//!
//! ```ignore
//! paramenum!(EnumName { a, b, c });
//! ```

#![crate_type = "dylib"]
#![feature(plugin_registrar, quote)]
//#![allow(unstable, unused_must_use)]

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
use syntax::ptr::P;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("client", expand_client);
    reg.register_macro("paramenum", expand_paramenum);
}

fn to_pascal_case(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let mut is_next_upper = true;
    for c in s.chars() {
        if c == '_' {
            is_next_upper = true;
        } else {
            res.push(
                if is_next_upper {
                    is_next_upper = false;
                    c.to_uppercase()
                } else {
                    c
                }
            );
        }
    }
    res
}

fn expand_client(cx: &mut ExtCtxt, _: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut p = cx.new_parser_from_tts(args);

    let client_name = p.parse_ident();
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

            let method_name = p.parse_ident();
            let mut request_struct_name = to_pascal_case(method_name.as_str());
            request_struct_name.push_str("RequestBuilder");
            let request_struct_name = cx.ident_of(&request_struct_name[]);

            p.expect(&token::Comma);
            let http_method = p.parse_expr();

            p.expect(&token::Comma);
            let (url, _) = p.parse_str();

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
            let return_type = p.parse_ty();
            p.eat(&token::Comma);
            p.expect(&token::CloseDelim(token::Paren));

            let request_struct = {
                let request_struct_items = required_params.iter()
                    .map(|x| quote_tokens!(cx, $x,))
                    .chain(optional_params.iter().map(|ref x| {
                        let ref ty = x.ty;
                        let ref pat = x.pat;
                        quote_tokens!(cx, $pat: Option<$ty>,)
                    }))
                    .flat_map(|x| x.into_iter())
                    .collect::<Vec<_>>();
                quote_item!(cx,
                    pub struct $request_struct_name<T: ::conn::Authenticator> {
                        _auth: ::std::rc::Rc<T>,
                        $request_struct_items
                    }
                )
            };

            let request_impl = {
                let optional_params_fns = optional_params.iter().map(|ref x| {
                    let ref ty = x.ty;
                    let ref pat = x.pat;
                    if ty.to_source() == "String" {
                        quote_tokens!(cx, pub fn $pat(mut self, val: &str) -> Self {
                            self.$pat = Some(val.to_string());
                            self
                        })
                    } else {
                        quote_tokens!(cx, pub fn $pat(mut self, val: $ty) -> Self {
                            self.$pat = Some(val);
                            self
                        })
                    }
                }).flat_map(|x| x.into_iter()).collect::<Vec<_>>();
                let params_len = required_params.len() + optional_params.len();
                let need_format = url.get().contains("{}");
                let push = {
                    let mut reqparam_iter = required_params.iter();
                    if need_format { reqparam_iter.next(); }
                    reqparam_iter.map(|ref x| {
                        let ref pat = x.pat;
                        let n = pat.to_source();
                        let n = &n[];
                        quote_tokens!(cx, params.push(
                            ::conn::ToParameter::to_parameter(self.$pat.clone(), $n));)
                    }).chain(optional_params.iter().map(|ref x| {
                        let ref pat = x.pat;
                        let n = pat.to_source();
                        let n = &n[];
                        quote_tokens!(cx, match $pat {
                            Some(ref x) => params.push(::conn::ToParameter::to_parameter($x, $n)),
                            None => ()
                        })
                    })).flat_map(|x| x.into_iter()).collect::<Vec<_>>()
                };
                let url_format = {
                    let url = url.get();
                    if need_format {
                        let ref f = required_params[0].pat;
                        quote_tokens!(cx,
                            let url = format!($url, self.$f);
                            let url = &url[];
                        )
                    } else { quote_tokens!(cx, let url = $url;) }
                };
                let object =
                    if return_type.to_source().as_slice().contains("Box<") { quote_tokens!(cx, box j) }
                    else { quote_tokens!(cx, j) };
                quote_item!(cx,
                    impl<T: ::conn::Authenticator> $request_struct_name<T> {
                        $optional_params_fns

                        pub fn execute(&self) -> ::TwitterResult<$return_type> {
                            let mut params: Vec<::conn::Parameter> = Vec::with_capacity($params_len);
                            $push
                            $url_format
                            let res = try!(::conn::Authenticator::request_twitter(
                                &*self._auth, $http_method, url, params.as_slice()));
                            match ::conn::parse_json(res.raw_response.as_slice()) {
                                Ok(j) => Ok(res.object($object)),
                                Err(e) => Err(::TwitterError::JsonError(e, res))
                            }
                        }
                    }
                )
            };

            let client_fn = {
                let mut fn_args = Vec::new();
                let mut setters = Vec::new();
                for ref x in required_params.iter() {
                    let ref pat = x.pat;
                    if x.ty.to_source() == "String" {
                        fn_args.extend(quote_tokens!(cx, $pat: &str,).into_iter());
                        setters.extend(quote_tokens!(cx, $pat: $pat.to_string(),).into_iter());
                    } else {
                        fn_args.extend(quote_tokens!(cx, $x,).into_iter());
                        setters.extend(quote_tokens!(cx, $pat: $pat,).into_iter());
                    }
                }
                setters.extend(optional_params.iter().flat_map(|ref x| {
                    let ref pat = x.pat;
                    quote_tokens!(cx, $pat: None,).into_iter()
                }));
                quote_tokens!(cx, pub fn $method_name(&self, $fn_args) -> $request_struct_name {
                    $request_struct_name {
                        _auth: self.0.clone(),
                        $setters
                    }
                })
            };

            (request_struct, request_impl, client_fn)
        }
    );

    p.eat(&token::Comma);
    p.expect(&token::Eof);

    // client struct, client impl, API struct, API impl
    let mut items = Vec::with_capacity(defs.len() * 2 + 2);
    let mut client_fns = Vec::with_capacity(defs.len());
    for (request_struct, request_impl, client_fn) in defs.into_iter() {
        items.push(request_struct);
        items.push(request_impl);
        client_fns.push(client_fn);
    }

    items.push(quote_item!(cx,
        pub struct $client_name<T: ::conn::Authenticator>(pub ::std::rc::Rc<T>);
    ));

    items.push(quote_item!(cx,
        impl<T: ::conn::Authenticator> $client_name<T> { $client_fns }));

    MacItems::new(items.into_iter().map(|x| x.unwrap()))
}

fn expand_paramenum(cx: &mut ExtCtxt, _: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut p = cx.new_parser_from_tts(args);
    let name = p.parse_ident().to_string();
    p.expect(&token::OpenDelim(token::Brace));
    let items = p.parse_seq_to_end(
        &token::CloseDelim(token::Brace),
        SeqSep {
            sep: Some(token::Comma),
            trailing_sep_allowed: true
        },
        |p| {
            let i = p.parse_ident();
            (i.to_string(), to_pascal_case(i.as_str()))
        }
    );
    p.expect(&token::Eof);

    let mut e = format!("#[derive(Clone, Copy, Show)] pub enum {} {{\n    ", name);
    for x in items.iter() {
        write!(&mut e, "{}, ", x.1);
    }
    e.push_str("\n}");

    let mut i = format!("impl ::std::fmt::String for {} {{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {{
        f.write_str(match *self {{\n", name);
    for x in items.iter() {
        writeln!(&mut i, "          {}::{} => \"{}\",", name, x.1, x.0);
    }
    i.push_str("        })\n    }\n}");

    MacItems::new(vec![cx.parse_item(e), cx.parse_item(i)].into_iter())
}
