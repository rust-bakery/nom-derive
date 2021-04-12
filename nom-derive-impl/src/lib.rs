//! # nom-derive
//!
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE-MIT)
//! [![Apache License 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)
//! [![docs.rs](https://docs.rs/nom-derive/badge.svg)](https://docs.rs/nom-derive)
//! [![Build Status](https://travis-ci.org/chifflier/nom-derive.svg?branch=master)](https://travis-ci.org/chifflier/nom-derive)
//! [![Crates.io Version](https://img.shields.io/crates/v/nom-derive.svg)](https://crates.io/crates/nom-derive)
//!
//! ## Overview
//!
//! nom-derive is a custom derive attribute, to derive [nom] parsers automatically from the structure definition.
//!
//! It is not meant to replace [nom], but to provide a quick and easy way to generate parsers for
//! structures, especially for simple structures. This crate aims at simplifying common cases.
//! In some cases, writing the parser manually will remain more efficient.
//!
//! - [API documentation](https://docs.rs/nom-derive)
//! - [Documentation of `Nom` attribute](derive.Nom.html). This is the main
//!   documentation for this crate, with all possible options and many examples.
//!
//! *Feedback welcome !*
//!
//! ## `#[derive(Nom)]`
//!
//! This crate exposes a single custom-derive macro `Nom` which
//! implements `parse` for the struct it is applied to.
//!
//! The goal of this project is that:
//!
//! * `derive(Nom)` should be enough for you to derive [nom] parsers for simple
//!   structures easily, without having to write it manually
//! * it allows overriding any parsing method by your own
//! * it allows using generated parsing functions along with handwritten parsers and
//!   combining them without efforts
//! * it remains as fast as nom
//!
//! `nom-derive` adds declarative parsing to `nom`. It also allows mixing with
//! procedural parsing easily, making writing parsers for byte-encoded formats
//! very easy.
//!
//! For example:
//!
//! ```ignore
//! use nom_derive::{Nom, Parse};
//!
//! #[derive(Nom)]
//! struct S {
//!   a: u32,
//!   b: u16,
//!   c: u16
//! }
//! ```
//!
//! This adds a static method `parse` to `S`, with the following signature:
//! ```rust,ignore
//! impl S {
//!     pub fn parse(i: &[u8]) -> nom::IResult(&[u8], S);
//! }
//! ```
//!
//! To parse input, just call `let res = S::parse(input);`.
//!
//! For extensive documentation of all attributes and examples, see the [Nom derive
//! attribute](derive.Nom.html) documentation.
//!
//! Many examples are provided, and more can be found in the [project
//! tests](https://github.com/rust-bakery/nom-derive/tree/master/tests).
//!
//! ## Combinators visibility
//!
//! All inferred parsers will generate code with absolute type path, so there is no need
//! to add `use` statements for them. However, if you use any combinator directly (or in a `Parse`
//! statement, for ex.), it has to be imported as usual.
//!
//! That is probably not going to change, since
//! * a proc_macro cannot export items other than functions tagged with `#[proc_macro_derive]`
//! * there are variants of combinators with the same names (complete/streaming, bits/bytes), so
//!   re-exporting them would create side-effects.
//!
//! ## Debug tips
//!
//! * If the generated parser does not compile, add `#[nom(DebugDerive)]` to the structure.
//!   It will dump the generated parser to `stderr`.
//! * If the generated parser fails at runtime, try adding `#[nom(Debug)]` to the structure or
//!   to fields. It wraps subparsers in `dbg_dmp` and will print the field name and input to
//!   `stderr` if the parser fails.
//!
//! [nom]: https://github.com/geal/nom

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::*;

mod config;
mod endian;
mod enums;
mod meta;
mod parsertree;
mod structs;

use crate::endian::set_object_endianness;
use crate::meta::attr::{MetaAttr, MetaAttrType};
use enums::impl_nom_enums;
use structs::{get_pre_post_exec, parse_struct};

/// The `Nom` derive automatically generates a `parse` function for the structure
/// using [nom] parsers. It will try to infer parsers for primitive of known
/// types, but also allows you to specify parsers using custom attributes.
///
/// Deriving parsers supports `struct` and `enum` types.
///
/// Many examples are provided, and more can be found in the [project
/// tests](https://github.com/rust-bakery/nom-derive/tree/master/tests).
///
/// [nom]: https://github.com/Geal/nom
#[proc_macro_derive(Nom, attributes(nom))]
pub fn nom(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build and return the generated impl
    match impl_nom(&ast, false) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

fn get_extra_args(meta_list: &[MetaAttr]) -> Option<&proc_macro2::TokenStream> {
    meta_list
        .iter()
        .find(|m| m.attr_type == MetaAttrType::ExtraArgs)
        .and_then(MetaAttr::arg)
}

fn impl_nom(ast: &syn::DeriveInput, debug_derive: bool) -> Result<TokenStream> {
    use crate::config::Config;
    // eprintln!("ast: {:#?}", ast);
    let struct_name = ast.ident.to_string();
    // parse top-level attributes and prepare tokens for each field parser
    let meta = meta::parse_nom_top_level_attribute(&ast.attrs)?;
    // eprintln!("top-level meta: {:?}", meta);
    let mut config = Config::from_meta_list(struct_name, &meta).expect("Could not build config");
    config.debug_derive |= debug_derive;
    set_object_endianness(ast.ident.span(), &meta, &mut config)?;
    let (tl_pre, tl_post) = get_pre_post_exec(&meta, &config);
    // enums are handled differently
    let s = match &ast.data {
        syn::Data::Enum(_) => {
            return impl_nom_enums(ast, &mut config);
        }
        syn::Data::Struct(ref s) => parse_struct(s, &mut config)?,
        syn::Data::Union(_) => panic!("Unions not supported"),
    };
    // prepare tokens
    let generics = &ast.generics;
    let name = &ast.ident;
    let (idents, parser_tokens): (Vec<_>, Vec<_>) = s
        .parsers
        .iter()
        .map(|sp| {
            let id = syn::Ident::new(&sp.name, Span::call_site());
            (id, &sp.item)
        })
        .unzip();
    let (pre, post): (Vec<_>, Vec<_>) = s
        .parsers
        .iter()
        .map(|sp| (sp.pre_exec.as_ref(), sp.post_exec.as_ref()))
        .unzip();
    let idents2 = idents.clone();
    // Code generation
    let struct_def = match (s.empty, s.unnamed) {
        (true, _) => quote! { ( #name ) },
        (_, true) => quote! { ( #name ( #(#idents2),* ) ) },
        (_, false) => quote! { ( #name { #(#idents2),* } ) },
    };
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    let orig_input_name = syn::Ident::new(
        &("orig_".to_string() + &config.input_name),
        Span::call_site(),
    );
    let extra_args = get_extra_args(&meta);
    let fn_body = quote! {
        let #input_name = #orig_input_name;
        #tl_pre
        #(#pre let (#input_name, #idents) = #parser_tokens (#input_name) ?; #post)*
        let struct_def = #struct_def;
        #tl_post
        Ok((#input_name, struct_def))
    };
    // get lifetimes
    let lft = Lifetime::new("'nom", Span::call_site());
    let lfts: Vec<_> = generics.lifetimes().collect();
    let mut fn_where_clause = WhereClause {
        where_token: Token![where](Span::call_site()),
        predicates: punctuated::Punctuated::new(),
    };
    if !lfts.is_empty() {
        // input slice must outlive all lifetimes from Self
        let wh: WherePredicate = parse_quote! { #lft: #(#lfts)+* };
        fn_where_clause.predicates.push(wh);
        // for &l in &lfts {
        //     let rev_wh: WherePredicate = parse_quote! { #l: #lft };
        //     fn_where_clause.predicates.push(rev_wh);
        // }
    };
    // function declaration line
    let fn_decl = if config.generic_errors {
        let ident_e = Ident::new("E", Span::call_site());
        // extend where clause for generic parameters
        let dep: WherePredicate = parse_quote! {
            #ident_e: nom_derive::nom::error::ParseError<&#lft [u8]>
        };
        fn_where_clause.predicates.push(dep);
        // let dep: WherePredicate = parse_quote! { #ident_e: std::fmt::Debug };
        // fn_where_clause.predicates.push(dep);
        quote! {
            pub fn parse<#lft, #ident_e>(#orig_input_name: &#lft [u8] #extra_args) -> nom::IResult<&#lft [u8], Self, #ident_e>
            #fn_where_clause
        }
    } else {
        quote! {
            pub fn parse<#lft>(#orig_input_name: &#lft [u8] #extra_args) -> nom::IResult<&#lft [u8], Self>
            #fn_where_clause
        }
    };
    // extract impl parameters
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Generate impl
    let impl_tokens = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #fn_decl
            {
                #fn_body
            }
        }
    };
    if config.debug_derive {
        eprintln!("tokens:\n{}", impl_tokens);
    }
    Ok(impl_tokens.into())
}

/// This derive macro behaves exactly like [Nom derive](derive.Nom.html), except it
/// prints the generated parser on stderr.
/// This is helpful for debugging generated parsers.
#[deprecated(
    since = "0.6.0",
    note = "Please use the nom(DebugDerive) attribute instead"
)]
#[proc_macro_derive(NomDeriveDebug, attributes(nom))]
pub fn nom_derive_debug(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build and return the generated impl
    match impl_nom(&ast, true) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}
