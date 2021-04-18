//! # nom-derive-impl
//!
//! ## Overview
//!
//! nom-derive is a custom derive attribute, to derive `nom` parsers automatically from the structure definition.
//!
//! This crate is not meant to be used directly.
//! See [`nom-derive`](https://docs.rs/nom-derive) crate for documentation.

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
mod gen;
mod meta;
mod parsertree;
mod structs;

use crate::config::Config;
use crate::endian::{set_object_endianness, ParserEndianness};
use crate::gen::*;
use crate::meta::attr::{MetaAttr, MetaAttrType};
use enums::impl_nom_enums;
use structs::{get_pre_post_exec, parse_struct};

/// The `Nom` derive automatically generates a `parse` function for the structure
/// using [nom] parsers. It will try to infer parsers for primitive of known
/// types, but also allows you to specify parsers using custom attributes.
///
/// Deriving parsers supports `struct` and `enum` types.
///
/// The documentation of the `Nom` custom derive attribute and all possible options
/// can be found in the [nom-derive documentation](https://docs.rs/nom-derive).
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
    match impl_nom(&ast, false, ParserEndianness::Unspecified) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(NomBE, attributes(nom))]
pub fn nom_be(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build and return the generated impl
    match impl_nom(&ast, false, ParserEndianness::BigEndian) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(NomLE, attributes(nom))]
pub fn nom_le(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build and return the generated impl
    match impl_nom(&ast, false, ParserEndianness::LittleEndian) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

pub(crate) fn get_extra_args(meta_list: &[MetaAttr]) -> Option<&proc_macro2::TokenStream> {
    meta_list
        .iter()
        .find(|m| m.attr_type == MetaAttrType::ExtraArgs)
        .and_then(MetaAttr::arg)
}

fn impl_nom(
    ast: &syn::DeriveInput,
    debug_derive: bool,
    endianness: ParserEndianness,
) -> Result<TokenStream> {
    // eprintln!("ast: {:#?}", ast);
    let struct_name = ast.ident.to_string();
    // parse top-level attributes and prepare tokens for each field parser
    let meta = meta::parse_nom_top_level_attribute(&ast.attrs)?;
    // eprintln!("top-level meta: {:?}", meta);
    let mut config = Config::from_meta_list(struct_name, &meta).expect("Could not build config");
    config.debug_derive |= debug_derive;
    set_object_endianness(ast.ident.span(), endianness, &meta, &mut config)?;
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
    let orig_input_name = get_orig_input_name(&config);
    let extra_args = get_extra_args(&meta);
    let fn_body = quote! {
        let #input_name = #orig_input_name;
        #tl_pre
        #(#pre let (#input_name, #idents) = #parser_tokens (#input_name) ?; #post)*
        let struct_def = #struct_def;
        #tl_post
        Ok((#input_name, struct_def))
    };
    let fn_decl = gen_fn_decl(generics, extra_args, &config);
    // extract impl parameters from struct
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
    match impl_nom(&ast, true, ParserEndianness::Unspecified) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}
