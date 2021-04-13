use crate::config::Config;
use proc_macro2::{Span, TokenStream};
use syn::*;

pub(crate) fn get_orig_input_name(config: &Config) -> Ident {
    Ident::new(
        &("orig_".to_string() + &config.input_name),
        Span::call_site(),
    )
}

pub(crate) fn gen_fn_decl(
    generics: &Generics,
    extra_args: Option<&TokenStream>,
    config: &Config,
) -> TokenStream {
    let orig_input_name = get_orig_input_name(config);
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
    };
    // function declaration line
    if config.generic_errors {
        let ident_e = Ident::new("NomErr", Span::call_site());
        // extend where clause for generic parameters
        let dep: WherePredicate = parse_quote! {
            #ident_e: nom_derive::nom::error::ParseError<&#lft [u8]>
        };
        fn_where_clause.predicates.push(dep);
        // make sure generic parameters inplement Parse
        for param in generics.type_params() {
            let param_ident = &param.ident;
            let dep: WherePredicate = parse_quote! { #param_ident: Parse<&#lft [u8], #ident_e> };
            fn_where_clause.predicates.push(dep);
        }
        // let dep: WherePredicate = parse_quote! { #ident_e: std::fmt::Debug };
        // fn_where_clause.predicates.push(dep);
        quote! {
            pub fn parse<#lft, #ident_e>(#orig_input_name: &#lft [u8] #extra_args) -> nom::IResult<&#lft [u8], Self, #ident_e>
            #fn_where_clause
        }
    } else {
        // make sure generic parameters inplement Parse
        for param in generics.type_params() {
            let param_ident = &param.ident;
            let dep: WherePredicate = parse_quote! { #param_ident: Parse<&#lft [u8]> };
            fn_where_clause.predicates.push(dep);
        }
        quote! {
            pub fn parse<#lft>(#orig_input_name: &#lft [u8] #extra_args) -> nom::IResult<&#lft [u8], Self>
            #fn_where_clause
        }
    }
}
