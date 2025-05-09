use crate::config::*;
use crate::meta;
use crate::meta::attr::{MetaAttr, MetaAttrType};
use crate::parsertree::{ParserExpr, ParserTreeItem};
use crate::structs::{get_pre_post_exec, parse_fields, StructParser, StructParserTree};
use syn::spanned::Spanned;
use syn::*;

#[derive(Debug)]
pub(crate) struct VariantParserTree {
    pub ident: syn::Ident,
    pub selector_type: String,
    pub struct_def: StructParserTree,
}

pub(crate) fn parse_variant(
    variant: &syn::Variant,
    config: &mut Config,
) -> Result<VariantParserTree> {
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
        selector_type: selector,
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

pub(crate) fn get_repr(attrs: &[syn::Attribute]) -> Option<Ident> {
    attrs.iter().find_map(|attr| {
        //
        let mut ident = None::<Ident>;
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if let Some(word) = meta.path.get_ident() {
                    ident = Some(word.clone());
                    Ok(())
                } else {
                    panic!("unsupported nested type for 'repr'");
                }
            })
            .ok()?;
        }
        ident
    })
}

pub(crate) fn is_input_fieldless_enum(ast: &syn::DeriveInput) -> bool {
    match ast.data {
        syn::Data::Enum(ref data_enum) => {
            // eprintln!("{:?}", data_enum);
            for v in data_enum.variants.iter() {
                if syn::Fields::Unit != v.fields {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}
