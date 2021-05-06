use crate::config::*;
use crate::endian::*;
use crate::gen::*;
use crate::meta;
use crate::meta::attr::{MetaAttr, MetaAttrType};
use crate::parsertree::{ParserExpr, ParserTreeItem};
use crate::structs::{get_pre_post_exec, parse_fields, StructParser, StructParserTree};
use proc_macro2::{Span, TokenStream};
use syn::{spanned::Spanned, *};

#[derive(Debug)]
struct VariantParserTree {
    pub ident: syn::Ident,
    pub selector: String,
    pub struct_def: StructParserTree,
}

fn parse_variant(variant: &syn::Variant, config: &mut Config) -> Result<VariantParserTree> {
    // eprintln!("variant: {:?}", variant);
    let meta_list =
        meta::parse_nom_attribute(&variant.attrs).expect("Parsing the 'nom' meta attribute failed");
    let selector = get_selector(&meta_list).ok_or_else(|| {
        Error::new(
            variant.span(),
            "Nom-derive: the 'Selector' attribute must be used to give the value of selector item",
        )
    })?;
    let mut struct_def = parse_fields(&variant.fields, config)?;
    if variant.fields == syn::Fields::Unit {
        let mut p = None;
        for meta in &meta_list {
            if meta.attr_type == MetaAttrType::Parse {
                let s = meta.arg().unwrap();
                p = Some(ParserExpr::Raw(s.clone()));
            }
        }
        let (pre, post) = get_pre_post_exec(&meta_list, config);
        let p = p.unwrap_or(ParserExpr::Nop);
        let item = ParserTreeItem::new(Some(variant.ident.clone()), p);
        let sp = StructParser::new("_".to_string(), item, pre, post);
        struct_def.parsers.push(sp);
    }
    // discriminant ?
    Ok(VariantParserTree {
        ident: variant.ident.clone(),
        selector,
        struct_def,
    })
}

fn get_selector(meta_list: &[MetaAttr]) -> Option<String> {
    for meta in meta_list {
        if MetaAttrType::Selector == meta.attr_type {
            return Some(meta.arg().unwrap().to_string());
        }
    }
    None
}

fn get_repr(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let Ok(ref meta) = attr.parse_meta() {
            match meta {
                syn::Meta::NameValue(_) | syn::Meta::Path(_) => (),
                syn::Meta::List(ref metalist) => {
                    if let Some(ident) = metalist.path.get_ident() {
                        if ident == "repr" {
                            for n in metalist.nested.iter() {
                                match n {
                                    syn::NestedMeta::Meta(meta) => match meta {
                                        syn::Meta::Path(path) => {
                                            if let Some(word) = path.get_ident() {
                                                return Some(word.to_string());
                                            } else {
                                                panic!("unsupported nested type for 'repr'")
                                            }
                                        }
                                        _ => panic!("unsupported nested type for 'repr'"),
                                    },
                                    _ => panic!("unsupported meta type for 'repr'"),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn is_input_fieldless_enum(ast: &syn::DeriveInput) -> bool {
    match ast.data {
        syn::Data::Enum(ref data_enum) => {
            // eprintln!("{:?}", data_enum);
            data_enum.variants.iter().fold(true, |acc, v| {
                if let syn::Fields::Unit = v.fields {
                    acc
                } else {
                    false
                }
            })
        }
        _ => false,
    }
}

fn impl_nom_fieldless_repr_enum(
    ast: &syn::DeriveInput,
    repr: &str,
    endianness: ParserEndianness,
    meta_list: &[MetaAttr],
    config: &Config,
) -> Result<TokenStream> {
    let input = syn::Ident::new(config.input_name(), Span::call_site());
    let orig_input = syn::Ident::new(config.orig_input_name(), Span::call_site());
    let (tl_pre, tl_post) = get_pre_post_exec(&meta_list, config);
    let parser = match repr {
        "u8" | "u16" | "u24" | "u32" | "u64" | "u128" | "i8" | "i16" | "i24" | "i32" | "i64"
        | "i128" => {
            let endian = get_object_endianness(config);
            match endian {
                ParserEndianness::BigEndian => {
                    let p = syn::Ident::new(&format!("be_{}", repr), Span::call_site());
                    let qq = quote! {
                        nom::number::streaming::#p
                    };
                    Some(ParserExpr::Raw(qq))
                }
                ParserEndianness::LittleEndian | ParserEndianness::Unspecified => {
                    let p = syn::Ident::new(&format!("le_{}", repr), Span::call_site());
                    let qq = quote! {
                        nom::number::streaming::#p
                    };
                    Some(ParserExpr::Raw(qq))
                }
                ParserEndianness::SetEndian => unimplemented!("SetEndian for fieldless enums"),
            }
        }
        _ => {
            return Err(Error::new(
                ast.span(),
                "Nom-derive: cannot parse 'repr' content (must be a primitive type)",
            ))
        }
    };
    let variant_names: Vec<_> = if let syn::Data::Enum(ref data_enum) = ast.data {
        // eprintln!("{:?}", data_enum);
        data_enum
            .variants
            .iter()
            .map(|v| v.ident.to_string())
            .collect()
    } else {
        panic!("expect enum");
    };
    let name = &ast.ident;
    let ty = syn::Ident::new(&repr, Span::call_site());
    let variants_code: Vec<_> = variant_names
        .iter()
        .map(|variant_name| {
            let id = syn::Ident::new(variant_name, Span::call_site());
            quote! { if selector == #name::#id as #ty { #name::#id } }
        })
        .collect();
    // note: fieldless enums cannot have lifetimes
    let fn_decl = gen_fn_decl(endianness, None, config);
    // generate impl
    let tokens = quote! {
        #fn_decl {
            let #input = #orig_input;
            #tl_pre
            let (#input, selector) = #parser(#input)?;
            let enum_def =
                #(#variants_code else)*
            { return Err(nom::Err::Error(nom::error::make_error(#orig_input, nom::error::ErrorKind::Switch))); };
            #tl_post
            Ok((#input, enum_def))
        }
    };

    Ok(tokens)
}

pub(crate) fn impl_nom_enums(
    ast: &syn::DeriveInput,
    meta: &[MetaAttr],
    endianness: ParserEndianness,
    config: &mut Config,
) -> Result<TokenStream> {
    let name = &ast.ident;
    // eprintln!("{:?}", ast.attrs);

    // endianness must be set before parsing enum
    set_object_endianness(ast.ident.span(), endianness, &meta, config)?;

    let input = syn::Ident::new(config.input_name(), Span::call_site());
    let orig_input = syn::Ident::new(config.orig_input_name(), Span::call_site());
    let extra_args = get_extra_args(&meta);
    let selector = match config.selector() {
        Some(s) => s.to_owned(),
        None => {
            if is_input_fieldless_enum(ast) {
                // check that we have a repr attribute
                let repr = get_repr(&ast.attrs).ok_or_else(|| {
                    Error::new(
                        ast.ident.span(),
                        "Nom-derive: fieldless enums must have a 'repr' or 'selector' attribute",
                    )
                })?;
                return impl_nom_fieldless_repr_enum(ast, &repr, endianness, &meta, config);
            } else {
                return Err(Error::new(
                    ast.ident.span(),
                    "Nom-derive: enums must specify the 'selector' attribute",
                ));
            }
        }
    };
    let variants_defs: Result<Vec<_>> = if let syn::Data::Enum(ref data_enum) = ast.data {
        // eprintln!("{:?}", data_enum);
        data_enum
            .variants
            .iter()
            .map(|v| parse_variant(v, config))
            .collect()
    } else {
        panic!("expect enum");
    };
    let mut variants_defs = variants_defs?;
    // parse string items and prepare tokens for each variant
    let (tl_pre, tl_post) = get_pre_post_exec(&meta, config);
    let selector_type: proc_macro2::TokenStream = selector.parse().unwrap();
    let mut default_case_handled = false;
    let mut variants_code: Vec<_> = {
        variants_defs
            .iter()
            .map(|def| {
                if def.selector == "_" {
                    default_case_handled = true;
                }
                let m: proc_macro2::TokenStream =
                    def.selector.parse().expect("invalid selector value");
                let variantname = &def.ident;
                let (idents, parser_tokens): (Vec<_>, Vec<_>) = def
                    .struct_def
                    .parsers
                    .iter()
                    .map(|sp| {
                        let id = syn::Ident::new(&sp.name, Span::call_site());
                        (id, &sp.item)
                    })
                    .unzip();
                let (pre, post): (Vec<_>, Vec<_>) = def
                    .struct_def
                    .parsers
                    .iter()
                    .map(|sp| (sp.pre_exec.as_ref(), sp.post_exec.as_ref()))
                    .unzip();
                let idents2 = idents.clone();
                let struct_def = match (def.struct_def.empty, def.struct_def.unnamed) {
                    (true, _) => quote! { ( #name::#variantname ) },
                    (_, true) => quote! { ( #name::#variantname ( #(#idents2),* ) ) },
                    (_, false) => quote! { ( #name::#variantname { #(#idents2),* } ) },
                };
                quote! {
                #m => {
                    #(
                        #pre
                        let (#input, #idents) = #parser_tokens (#input) ?;
                        #post
                    )*
                    let struct_def = #struct_def;
                    Ok((#input, struct_def))
                        // Err(nom::Err::Error(error_position!(#input_name, nom::ErrorKind::Switch)))
                },
                }
            })
            .collect()
    };
    // if we have a default case, make sure it is the last entry
    if default_case_handled {
        let pos = variants_defs
            .iter()
            .position(|def| def.selector == "_")
            .expect("default case is handled but couldn't find index");
        let last_index = variants_defs.len() - 1;
        if pos != last_index {
            variants_defs.swap(pos, last_index);
            variants_code.swap(pos, last_index);
        }
    }
    let extra_args = if extra_args.is_some() {
        quote! { selector: #selector_type, #extra_args }
    } else {
        quote! { selector: #selector_type }
    };
    let fn_decl = gen_fn_decl(endianness, Some(&extra_args), &config);
    // generate impl
    let default_case = if default_case_handled {
        quote! {}
    } else {
        quote! { _ => Err(nom::Err::Error(nom::error_position!(#input, nom::error::ErrorKind::Switch))) }
    };
    let tokens = quote! {
        #fn_decl {
            let #input = #orig_input;
            #tl_pre
            let (#input, enum_def) = match selector {
                #(#variants_code)*
                #default_case
            }?;
            #tl_post
            Ok((#input, enum_def))
        }
    };

    Ok(tokens)
}
