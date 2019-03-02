use proc_macro::TokenStream;
use syn;
use syn::export::Span;

use crate::structs::{parse_fields,StructParserTree};

#[derive(Debug)]
struct VariantParserTree{
    pub ident: syn::Ident,
    pub selector: String,
    pub struct_def: StructParserTree,
}

fn parse_variant(variant: &syn::Variant) -> VariantParserTree {
    // eprintln!("variant: {:?}", variant);
    let selector = get_selector(&variant.attrs).expect(&format!("The 'Selector' attribute must be used to give the value of selector item (variant {})", variant.ident));
    let struct_def = parse_fields(&variant.fields);
    // discriminant ?
    VariantParserTree{
        ident: variant.ident.clone(),
        selector,
        struct_def
    }
}

fn get_selector(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let Ok(ref meta) = attr.parse_meta() {
            match meta {
                syn::Meta::NameValue(ref namevalue) => {
                    if &namevalue.ident == &"Selector" {
                        match &namevalue.lit {
                            syn::Lit::Str(litstr) => {
                                return Some(litstr.value())
                            },
                            _ => panic!("unsupported namevalue type")
                        }
                    }
                }
                syn::Meta::List(ref metalist) => {
                    if &metalist.ident == &"Selector" {
                        for n in metalist.nested.iter() {
                            match n {
                                syn::NestedMeta::Literal(lit) => {
                                    match lit {
                                        syn::Lit::Str(litstr) => {
                                            return Some(litstr.value())
                                        },
                                        _ => panic!("unsupported literal type")
                                    }
                                },
                                _ => panic!("unsupported meta type")
                            }
                        }
                    }
                }
                syn::Meta::Word(_) => ()
            }
        }
    }
    None
}

pub(crate) fn impl_nom_enums(ast: &syn::DeriveInput, debug:bool) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    // eprintln!("{:?}", ast.attrs);
    let selector = get_selector(&ast.attrs).expect("The 'Selector' attribute must be used to give the type of selector item");
    let variant_defs : Vec<_> =
        match ast.data {
            syn::Data::Enum(ref data_enum) => {
                // eprintln!("{:?}", data_enum);
                data_enum.variants.iter()
                    .map(parse_variant)
                    .collect()
            },
            _ => { panic!("expect enum"); }
        };
    // parse string items and prepare tokens for each variant
    let selector_type : proc_macro2::TokenStream = selector.parse().unwrap();
    let mut default_case_handled = false;
    let variants_code : Vec<_> = {
        variant_defs.iter()
            .map(|def| {
                if def.selector == "_" { default_case_handled = true; }
                let m : proc_macro2::TokenStream = def.selector.parse().expect("invalid selector value");
                let variantname = &def.ident;
                let (idents,parser_tokens) : (Vec<_>,Vec<_>) = def.struct_def.parsers.iter()
                    .map(|(name,parser)| {
                        let id = syn::Ident::new(name, Span::call_site());
                        let s = format!("{}",parser);
                        let input : proc_macro2::TokenStream = s.parse().unwrap();
                        (id,input)
                    })
                    .unzip();
                let idents2 = idents.clone();
                let struct_def = match def.struct_def.unnamed {
                    false => quote!{ ( #name::#variantname { #(#idents2),* } ) },
                    true  => quote!{ ( #name::#variantname ( #(#idents2),* ) ) },
                };
                quote!{
                    #m => {
                        do_parse!{
                            i,
                            #(#idents: #parser_tokens >>)*
                            #struct_def
                        }
                        // Err(nom::Err::Error(error_position!(i, nom::ErrorKind::Switch)))
                    },
                }
            })
            .collect()
    };
    // generate code
    let default_case =
        if default_case_handled { quote!{} }
        else { quote!{ _ => Err(nom::Err::Error(error_position!(i, nom::ErrorKind::Switch))) } };
    let tokens = quote!{
        impl#generics #name#generics {
            fn parse(i: &[u8], selector: #selector_type) -> IResult<&[u8],#name> {
                match selector {
                    #(#variants_code)*
                    #default_case
                }
            }
        }
    };

    if debug {
        eprintln!("impl_nom_enums: {}", tokens);
    }

    tokens.into()
}
