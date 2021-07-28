use crate::config::Config;
use crate::endian::*;
use crate::enums::impl_nom_enums;
use crate::meta;
use crate::meta::attr::{MetaAttr, MetaAttrType};
use crate::structs::gen_struct_impl;
use proc_macro2::{Span, TokenStream};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::*;

pub(crate) fn gen_fn_decl(
    endianness: ParserEndianness,
    extra_args: Option<&TokenStream>,
    config: &Config,
) -> TokenStream {
    let orig_input = Ident::new(config.orig_input_name(), Span::call_site());
    let parse = match endianness {
        ParserEndianness::BigEndian => "parse_be",
        ParserEndianness::LittleEndian => "parse_le",
        ParserEndianness::SetEndian => panic!("gen_fn_decl should never receive SetEndian"),
        ParserEndianness::Unspecified => "parse",
    };
    let parse = Ident::new(parse, Span::call_site());
    // get lifetimes
    let lft = Lifetime::new(&config.lifetime_name(), Span::call_site());
    let mut fn_where_clause = WhereClause {
        where_token: Token![where](Span::call_site()),
        predicates: punctuated::Punctuated::new(),
    };
    let mut fn_args: Punctuated<_, Token![,]> = Punctuated::new();
    let arg_input: FnArg = parse_quote!(#orig_input: &#lft [u8]);
    fn_args.push(arg_input);
    // check extra args
    if let Some(ts) = extra_args {
        let ts = ts.clone();
        let parser = Punctuated::<syn::FnArg, syn::Token![,]>::parse_separated_nonempty;
        let extra_args = parser.parse2(ts).expect("parse extra_args");
        for extra_arg in &extra_args {
            fn_args.push(extra_arg.clone());
        }
    };
    let special_case = extra_args.is_some() || config.selector().is_some();
    let mut scope = quote! {};
    if special_case {
        scope = quote! { pub }
    }
    // function declaration line
    if config.generic_errors {
        let ident_e = Ident::new(config.error_name(), Span::call_site());
        let mut fn_generics = None;
        if special_case {
            // special case: not implementing the Parse trait,
            // generic errors must be added to function, not struct
            //
            // extend where clause for generic parameters
            let dep: WherePredicate = parse_quote! {
                #ident_e: nom_derive::nom::error::ParseError<&#lft [u8]>
            };
            fn_where_clause.predicates.push(dep);
            let dep: WherePredicate = parse_quote! { #ident_e: std::fmt::Debug };
            fn_where_clause.predicates.push(dep);
            // add error type to function generics
            fn_generics = Some(quote!(<#ident_e>));
        }
        quote! {
           #scope fn #parse#fn_generics(#fn_args) -> nom::IResult<&#lft [u8], Self, #ident_e>
            #fn_where_clause
        }
    } else {
        quote! {
           #scope fn #parse(#fn_args) -> nom::IResult<&#lft [u8], Self>
            #fn_where_clause
        }
    }
}

pub(crate) fn get_extra_args(meta_list: &[MetaAttr]) -> Option<&proc_macro2::TokenStream> {
    meta_list
        .iter()
        .find(|m| m.attr_type == MetaAttrType::ExtraArgs)
        .and_then(MetaAttr::arg)
}

pub(crate) fn gen_impl(
    ast: &syn::DeriveInput,
    debug_derive: bool,
    endianness: ParserEndianness,
) -> Result<TokenStream> {
    // eprintln!("ast: {:#?}", ast);
    let struct_name = ast.ident.to_string();
    // parse top-level attributes and prepare tokens for each field parser
    let meta = meta::parse_nom_top_level_attribute(&ast.attrs)?;
    // eprintln!("top-level meta: {:?}", meta);
    let mut config = Config::from_meta_list(struct_name, &meta)?;
    config.debug_derive |= debug_derive;
    let orig_input = Ident::new(config.orig_input_name(), Span::call_site());
    let lft = Lifetime::new(config.lifetime_name(), Span::call_site());
    let ident_e = Ident::new(config.error_name(), Span::call_site());
    set_object_endianness(ast.ident.span(), endianness, &meta, &mut config)?;
    let extra_args = get_extra_args(&meta);
    // build function args
    let mut fn_args: Punctuated<_, Token![,]> = Punctuated::new();
    let arg_input: FnArg = parse_quote!(#orig_input: &#lft [u8]);
    fn_args.push(arg_input);
    // build call_args from extra_args
    let mut call_args: Punctuated<_, Token![,]> = Punctuated::new();
    // check selector
    call_args.push(orig_input);
    match (config.selector(), config.selector_name()) {
        (Some(sel_type), Some(s)) => {
            let selector = Ident::new(s, Span::call_site());
            let sel_type = Ident::new(sel_type, Span::call_site());
            let selector_arg: FnArg = parse_quote!(#selector: #sel_type);
            fn_args.push(selector_arg);
            call_args.push(selector);
        }
        (None, None) => (),
        (_, _) => panic!("Impossible configuration for 'selector'"),
    }
    // check extra args
    if let Some(ts) = extra_args {
        let ts = ts.clone();
        let parser = Punctuated::<syn::FnArg, syn::Token![,]>::parse_separated_nonempty;
        let extra_args = parser.parse2(ts).expect("parse extra_args");
        for extra_arg in &extra_args {
            fn_args.push(extra_arg.clone());
            match extra_arg {
                syn::FnArg::Receiver(_) => panic!("self should not be used in extra_args"),
                syn::FnArg::Typed(t) => {
                    if let syn::Pat::Ident(pat_ident) = t.pat.as_ref() {
                        call_args.push(pat_ident.ident.clone());
                    } else {
                        panic!("unexpected pattern in extra_args");
                    }
                }
            }
        }
    };

    // XXX if Error type is not used (not a parse Trait), it should be moved to the function

    // generate trait:
    //   get forced endianness, if any
    //   if forced endianness(ex: be): gen only parse_be, and make other points to parse_be
    //   if !forced: gen parse_le and parse_be
    let endianness = get_object_endianness(&config);
    // big-endian
    let tokens_parse_be = if endianness != ParserEndianness::LittleEndian {
        match &ast.data {
            syn::Data::Enum(_) => {
                impl_nom_enums(ast, &meta, ParserEndianness::BigEndian, &mut config)?
            }
            syn::Data::Struct(_) => {
                gen_struct_impl(ast, &meta, ParserEndianness::BigEndian, &mut config)?
            }
            syn::Data::Union(_) => panic!("Unions not supported"),
        }
    } else {
        // generate stub to call parse_le
        let fn_decl = gen_fn_decl(ParserEndianness::BigEndian, None, &config);
        quote! {
            #fn_decl {
                Self::parse_le(#call_args)
            }
        }
    };
    // little-endian
    let tokens_parse_le = if endianness != ParserEndianness::BigEndian {
        match &ast.data {
            syn::Data::Enum(_) => {
                impl_nom_enums(ast, &meta, ParserEndianness::LittleEndian, &mut config)?
            }
            syn::Data::Struct(_) => {
                gen_struct_impl(ast, &meta, ParserEndianness::LittleEndian, &mut config)?
            }
            syn::Data::Union(_) => panic!("Unions not supported"),
        }
    } else {
        // generate stub to call parse_be
        let fn_decl = gen_fn_decl(ParserEndianness::LittleEndian, None, &config);
        quote! {
            #fn_decl {
                Self::parse_be(#call_args)
            }
        }
    };

    // 'parse' function
    let maybe_err = if config.generic_errors {
        quote!( , #ident_e )
    } else {
        quote!()
    };
    let special_case = extra_args.is_some() || config.selector().is_some();
    let mut scope = quote! {};
    if special_case {
        scope = quote! { pub };
    }
    let tokens_parse = {
        let (fn_generics, where_clause) = if config.generic_errors && special_case {
            (
                quote!(<#ident_e>),
                quote! {where
                    #ident_e: nom_derive::nom::error::ParseError<&#lft [u8]>,
                    #ident_e: std::fmt::Debug,
                },
            )
        } else {
            (quote!(), quote!())
        };
        quote! {
           #scope fn parse#fn_generics(#fn_args) -> nom::IResult<&'nom [u8], Self #maybe_err> #where_clause {
                Self::parse_be(#call_args)
            }
        }
    };

    // combine everything

    let name = &ast.ident;
    // extract impl parameters from struct
    let orig_generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = orig_generics.split_for_impl();

    let mut gen_impl: Generics = parse_quote!(#impl_generics);
    gen_impl
        .params
        .push(GenericParam::Lifetime(LifetimeDef::new(lft.clone())));
    let param_e = TypeParam::from(ident_e.clone());

    let mut gen_wh: WhereClause = if where_clause.is_none() {
        parse_quote!(where)
    } else {
        parse_quote!(#where_clause)
    };
    let lfts: Vec<_> = ast.generics.lifetimes().collect();
    if !lfts.is_empty() {
        // input slice must outlive all lifetimes from Self
        let wh: WherePredicate = parse_quote! { #lft: #(#lfts)+* };
        gen_wh.predicates.push(wh);
    };

    // make sure generic parameters inplement Parse
    for param in ast.generics.type_params() {
        let param_ident = &param.ident;
        let dep: WherePredicate = parse_quote! { #param_ident: Parse< &#lft [u8] #maybe_err > };
        gen_wh.predicates.push(dep);
    }

    // Global impl
    let impl_tokens = if extra_args.is_some() || config.selector().is_some() {
        // There are extra arguments, so we can't generate the Parse impl
        // Generate an equivalent implementation
        if config.generic_errors {
            // XXX will fail: generic type should be added to function (npt struct), or
            // XXX compiler will complain that
            // XXX "the type parameter `NomErr` is not constrained by the impl trait, self type, or predicates"
            // this happens only when not implementing trait (the trait constrains NomErr)
            // let wh: WherePredicate = parse_quote!(#ident_e: nom::error::ParseError<& #lft [u8]>);
            // gen_wh.predicates.push(wh);
            // gen_impl.params.push(GenericParam::Type(param_e));
        }
        quote! {
            impl #gen_impl #name #ty_generics #gen_wh {
                #tokens_parse_be
                #tokens_parse_le
                #tokens_parse
            }
        }
    } else {
        // Generate an impl block for the Parse trait
        let error;
        if config.generic_errors {
            let wh: WherePredicate = parse_quote!(#ident_e: nom::error::ParseError<& #lft [u8]>);
            gen_wh.predicates.push(wh);
            gen_impl.params.push(GenericParam::Type(param_e));
            error = quote! { #ident_e };
        } else {
            error = quote! { nom::error::Error<&#lft [u8]> };
        }
        quote! {
                impl #gen_impl nom_derive::Parse<& #lft [u8], #error> for #name #ty_generics #gen_wh {
                    #tokens_parse_be
                    #tokens_parse_le
                    #tokens_parse
                }

        }
    };
    // eprintln!("\n***\nglobal_impl: {}\n---\n", impl_tokens);
    if config.debug_derive {
        eprintln!("tokens:\n{}", impl_tokens);
    }
    Ok(impl_tokens)
}
