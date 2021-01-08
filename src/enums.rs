use proc_macro::TokenStream;
use proc_macro2::Span;

use crate::config::Config;
use crate::meta;
use crate::meta::attr::{MetaAttr, MetaAttrType};
use crate::parsertree::ParserTree;
use crate::structs::{get_pre_post_exec, parse_fields, StructParser, StructParserTree};

#[derive(Debug)]
struct VariantParserTree {
    pub ident: syn::Ident,
    pub selector: String,
    pub struct_def: StructParserTree,
}

fn parse_variant(variant: &syn::Variant, config: &mut Config) -> VariantParserTree {
    // eprintln!("variant: {:?}", variant);
    let meta_list =
        meta::parse_nom_attribute(&variant.attrs).expect("Parsing the 'nom' meta attribute failed");
    let selector = get_selector(&meta_list).unwrap_or_else(|| {
        panic!(
            "The 'Selector' attribute must be used to give the value of selector item (variant {})",
            variant.ident
        )
    });
    let mut struct_def = parse_fields(&variant.fields, config);
    if variant.fields == syn::Fields::Unit {
        let mut p = None;
        for meta in &meta_list {
            if meta.attr_type == MetaAttrType::Parse {
                let s = meta.arg().unwrap().to_string();
                p = Some(ParserTree::Raw(s));
            }
        }
        let (pre, post) = get_pre_post_exec(&meta_list, config);
        let p = p.unwrap_or_else(|| ParserTree::Nop);
        let sp = StructParser::new("_".to_string(), p, pre, post);
        struct_def.parsers.push(sp);
    }
    // discriminant ?
    VariantParserTree {
        ident: variant.ident.clone(),
        selector,
        struct_def,
    }
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
                syn::Meta::NameValue(_) => (),
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
                syn::Meta::Path(_) => (),
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

fn impl_nom_fieldless_enums(
    ast: &syn::DeriveInput,
    repr: String,
    meta_list: &[MetaAttr],
    config: &Config,
) -> TokenStream {
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    let orig_input_name = syn::Ident::new(
        &("orig_".to_string() + &config.input_name),
        Span::call_site(),
    );
    let (tl_pre, tl_post) = get_pre_post_exec(&meta_list, config);
    let parser = match repr.as_ref() {
        "u8" | "u16" | "u24" | "u32" | "u64" | "u128" | "i8" | "i16" | "i24" | "i32" | "i64"
        | "i128" => {
            let is_big_endian = if meta_list.iter().any(|m| m.is_type(MetaAttrType::BigEndian)) {
                true
            } else if meta_list
                .iter()
                .any(|m| m.is_type(MetaAttrType::LittleEndian))
            {
                false
            } else {
                config.big_endian
            };
            if is_big_endian {
                Some(ParserTree::Raw(format!(
                    "nom::number::streaming::be_{}",
                    repr
                )))
            } else {
                Some(ParserTree::Raw(format!(
                    "nom::number::streaming::le_{}",
                    repr
                )))
            }
        }
        _ => panic!("Cannot parse 'repr' content"),
    };
    let variant_names: Vec<_> = match ast.data {
        syn::Data::Enum(ref data_enum) => {
            // eprintln!("{:?}", data_enum);
            data_enum
                .variants
                .iter()
                .map(|v| v.ident.to_string())
                .collect()
        }
        _ => {
            panic!("expect enum");
        }
    };
    let generics = &ast.generics;
    let name = &ast.ident;
    let ty = syn::Ident::new(&repr, Span::call_site());
    let variants_code: Vec<_> = variant_names
        .iter()
        .map(|variant_name| {
            let id = syn::Ident::new(variant_name, Span::call_site());
            quote! { if selector == #name::#id as #ty { #name::#id } }
        })
        .collect();
    let tokens = quote! {
        impl#generics #name#generics {
            pub fn parse(#orig_input_name: &[u8]) -> nom::IResult<&[u8],#name> {
                let #input_name = #orig_input_name;
                #tl_pre
                let (#input_name, selector) = #parser(#input_name)?;
                let enum_def =
                    #(#variants_code else)*
                { return Err(::nom::Err::Error((#orig_input_name, ::nom::error::ErrorKind::Switch))); };
                #tl_post
                Ok((#input_name, enum_def))
            }
        }
    };
    if config.debug_derive {
        eprintln!("impl_nom_enums: {}", tokens);
    }

    tokens.into()
}

pub(crate) fn impl_nom_enums(ast: &syn::DeriveInput, config: &mut Config) -> TokenStream {
    let name = &ast.ident;
    // eprintln!("{:?}", ast.attrs);
    let meta_list = meta::parse_nom_top_level_attribute(&ast.attrs)
        .expect("Parsing the 'nom' meta attribute failed");
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    let orig_input_name = syn::Ident::new(
        &("orig_".to_string() + &config.input_name),
        Span::call_site(),
    );
    let selector = match get_selector(&meta_list) {
        //.expect("The 'Selector' attribute must be used to give the type of selector item");
        Some(s) => s,
        None => {
            if is_input_fieldless_enum(ast) {
                // check that we have a repr attribute
                let repr = get_repr(&ast.attrs)
                    .expect("Nom-derive: fieldless enums must have a 'repr' attribute");
                return impl_nom_fieldless_enums(ast, repr, &meta_list, config);
            } else {
                panic!("Nom-derive: enums must specify the 'selector' attribute");
            }
        }
    };
    let mut variants_defs: Vec<_> = match ast.data {
        syn::Data::Enum(ref data_enum) => {
            // eprintln!("{:?}", data_enum);
            data_enum
                .variants
                .iter()
                .map(|v| parse_variant(v, config))
                .collect()
        }
        _ => {
            panic!("expect enum");
        }
    };
    // parse string items and prepare tokens for each variant
    let (tl_pre, tl_post) = get_pre_post_exec(&meta_list, config);
    let generics = &ast.generics;
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
                        (id, &sp.parser)
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
                        let (#input_name, #idents) = #parser_tokens (#input_name) ?;
                        #post
                    )*
                    let struct_def = #struct_def;
                    Ok((#input_name, struct_def))
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
    // generate code
    let default_case = if default_case_handled {
        quote! {}
    } else {
        quote! { _ => Err(nom::Err::Error(nom::error_position!(#input_name, nom::error::ErrorKind::Switch))) }
    };
    let tokens = quote! {
        impl#generics #name#generics {
            pub fn parse(#orig_input_name: &[u8], selector: #selector_type) -> nom::IResult<&[u8],#name> {
                let #input_name = #orig_input_name;
                #tl_pre
                let (#input_name, enum_def) = match selector {
                    #(#variants_code)*
                    #default_case
                }?;
                #tl_post
                Ok((#input_name, enum_def))
            }
        }
    };

    if config.debug_derive {
        eprintln!("impl_nom_enums: {}", tokens);
    }

    tokens.into()
}
