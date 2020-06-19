use proc_macro2::TokenStream;
use syn::*;
use syn::export::Span;

use crate::config::Config;
use crate::meta;
use crate::meta::attr::{MetaAttr, MetaAttrType};
use crate::parsertree::ParserTree;

#[derive(Debug)]
pub(crate) struct StructParser{
    pub name: String,
    pub parser: ParserTree,
    pub pre_exec: Option<TokenStream>,
    pub post_exec: Option<TokenStream>,
}

impl StructParser {
    pub fn new(name: String, parser: ParserTree, pre_exec: Option<TokenStream>, post_exec: Option<TokenStream>) -> Self {
        StructParser {
            name,
            parser,
            pre_exec,
            post_exec,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StructParserTree{
    pub unnamed: bool,
    pub parsers: Vec<StructParser>,
}

fn get_type_parser(ty: &Type, meta_list: &[MetaAttr], config: &Config) -> Option<ParserTree> {
    match ty {
        Type::Path(ref typepath) => {
            let path = &typepath.path;
            if path.segments.len() != 1 {
                panic!("Multiple segments in type path are not supported");
            }
            let segment = path.segments.last().expect("empty segments list");
            let ident_s = segment.ident.to_string();
            match ident_s.as_ref() {
                "u8"  |
                "u16" |
                "u24" |
                "u32" |
                "u64" |
                "i8"  |
                "i16" |
                "i24" |
                "i32" |
                "i64"    => {
                    let is_big_endian = if meta_list.iter().any(|m| m.is_type(MetaAttrType::BigEndian)) {
                        true
                    } else if meta_list.iter().any(|m| m.is_type(MetaAttrType::LittleEndian)) {
                        false
                    } else {
                        config.big_endian
                    };
                    if is_big_endian {
                        Some(ParserTree::Raw(format!("nom::number::streaming::be_{}", ident_s)))
                    } else {
                        Some(ParserTree::Raw(format!("nom::number::streaming::le_{}", ident_s)))
                    }
                },
                "Option" => {
                    match segment.arguments {
                        PathArguments::AngleBracketed(ref ab) => {
                            // eprintln!("Option type: {:?}", ab);
                            if ab.args.len() != 1 { panic!("Option type with multiple types are unsupported"); }
                            match &ab.args[0] {
                                GenericArgument::Type(ref ty) => {
                                    let s = get_type_parser(ty, meta_list, config);
                                    // eprintln!("    recursion: {:?}", s);
                                    s.map(|x| ParserTree::Opt(Box::new(ParserTree::Complete(Box::new(x)))))
                                },
                                _ => panic!("Option generic argument is not a type")
                            }
                        },
                        _ => panic!("Unsupported Option/parameterized type"),
                    }
                },
                "Vec"    => {
                    match segment.arguments {
                        PathArguments::AngleBracketed(ref ab) => {
                            // eprintln!("Vec type: {:?}", ab);
                            if ab.args.len() != 1 { panic!("Vec type with multiple types are unsupported"); }
                            match &ab.args[0] {
                                GenericArgument::Type(ref ty) => {
                                    let s = get_type_parser(ty, meta_list, config);
                                    // eprintln!("    recursion: {:?}", s);
                                    s.map(|x| ParserTree::Many0(Box::new(ParserTree::Complete(Box::new(x)))))
                                },
                                _ => panic!("Vec generic argument is not a type")
                            }
                        },
                        _ => panic!("Unsupported Vec/parameterized type"),
                    }
                },
                "PhantomData" => {
                    Some(ParserTree::PhantomData)
                }
                s        => {
                    Some(ParserTree::CallParse(s.to_owned()))
                },
            }
        },
        _ => None
    }
}

fn get_type_default(ty: &Type) -> Option<ParserTree> {
    match ty {
        Type::Path(ref typepath) => {
            let path = &typepath.path;
            if path.segments.len() != 1 {
                panic!("Multiple segments in type path are not supported");
            }
            let segment = path.segments.last().expect("empty segments list");
            let ident_s = segment.ident.to_string();
            let default = match ident_s.as_ref() {
                "u8"  |
                "u16" |
                "u32" |
                "u64" |
                "i8"  |
                "i16" |
                "i32" |
                "i64" => "0".to_string(),
                "Option" => "None".to_string(),
                "Vec" => "Vec::new()".to_string(),
                s => format!("{}::default()", s)
            };
            Some(ParserTree::Raw(
                format!("{{ |i| Ok((i, {})) }}", default)
            ))
        }
        _ => None
    }
}

fn get_parser(field: &::syn::Field, meta_list: &[MetaAttr], config: &Config) -> Option<ParserTree> {
    // eprintln!("field: {:?}", field);
    let ty = &field.ty;
    // first check if we have attributes set
    // eprintln!("attrs: {:?}", field.attrs);
    // eprintln!("meta_list: {:?}", meta_list);
    for meta in meta_list {
        match meta.attr_type {
            MetaAttrType::Take => {
                let s = meta.arg().unwrap().to_string();
                return Some(ParserTree::Take(s.clone()));
            }
            MetaAttrType::Value => {
                let s = meta.arg().unwrap().to_string();
                return Some(ParserTree::Value(s.clone()));
            }
            MetaAttrType::Parse => {
                let s = meta.arg().unwrap().to_string();
                return Some(ParserTree::Raw(s.clone()));
            }
            MetaAttrType::Ignore => {
                return get_type_default(ty);
            }
            MetaAttrType::Count => {
                // try to infer subparser
                let sub = get_type_parser(ty, meta_list, config);
                let s1 = match sub {
                    Some(ParserTree::Many0(m)) => { m },
                    _ => panic!("Unable to infer parser for 'Count' attribute. Is item type a Vec ?")
                };
                let s2 = match *s1 {
                    ParserTree::Complete(m) => { m },
                    _ => panic!("Unable to infer parser for 'Count' attribute. Is item type a Vec ?")
                };
                let s = meta.arg().unwrap().to_string();
                return Some(ParserTree::Count(s2, s.clone()));
            }
            _ => (),
        }
    }
    // else try primitive types knowledge
    get_type_parser(ty, meta_list, config)
}

fn quote_align(align: &TokenStream, config: &Config) -> TokenStream {
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    let orig_input_name = syn::Ident::new(&("orig_".to_string() + &config.input_name), Span::call_site());
    quote!{
        let (#input_name, _) = {
            let offset = #input_name.as_ptr() as usize - #orig_input_name.as_ptr() as usize;
            let align = #align as usize;
            let align = ((align - (offset % align)) % align);
            nom::bytes::streaming::take(align)(#input_name)
        }?;
    }
}

// like quote_skip, but offset is an isize
fn quote_move(offset: &TokenStream, config: &Config) -> TokenStream {
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    let orig_input_name = syn::Ident::new(&("orig_".to_string() + &config.input_name), Span::call_site());
    quote!{
        let #input_name = {
            let start = #orig_input_name.as_ptr() as usize;
            let pos = #input_name.as_ptr() as usize - start;
            let offset = #offset as isize;
            let offset_u = offset.abs() as usize;
            let new_offset = if offset < 0 {
                if offset_u > pos {
                    return Err(nom::Err::Error((#input_name, nom::error::ErrorKind::TooLarge)));
                }
                pos - offset_u
            } else {
                if pos + offset_u > #orig_input_name.len() {
                    return Err(nom::Err::Incomplete(nom::Needed::Size(offset_u)));
                }
                pos + offset_u
            };
            &#orig_input_name[new_offset..]
        };
    }
}

// like quote_move, with absolute value as offset
fn quote_move_abs(offset: &TokenStream, config: &Config) -> TokenStream {
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    let orig_input_name = syn::Ident::new(&("orig_".to_string() + &config.input_name), Span::call_site());
    quote!{
        let #input_name = {
            let start = #input_name.as_ptr() as usize;
            let offset = #offset as usize;
            if offset > #orig_input_name.len() {
                return Err(nom::Err::Incomplete(nom::Needed::Size(offset)));
            }
            &#orig_input_name[offset..]
        };
    }
}

fn quote_skip(skip: &TokenStream, config: &Config) -> TokenStream {
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    quote!{
        let (#input_name, _) = {
            let skip = #skip as usize;
            nom::bytes::streaming::take(skip)(#input_name)
        }?;
    }
}

fn quote_error_if(cond: &TokenStream, config: &Config) -> TokenStream {
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    quote!{
        if #cond {
            return Err(nom::Err::Error((#input_name, nom::error::ErrorKind::Verify)));
        }
    }
}

pub(crate) fn get_pre_post_exec(meta_list: &[MetaAttr], config: &Config) -> (Option<TokenStream>, Option<TokenStream>) {
    let mut tk_pre = proc_macro2::TokenStream::new();
    let mut tk_post = proc_macro2::TokenStream::new();
    for m in meta_list {
        match m.attr_type {
            MetaAttrType::PreExec => {
                // let code = m.arg().unwrap().to_string();
                // pre += &code;
                tk_pre.extend(m.arg().unwrap().clone());
            }
            MetaAttrType::PostExec => {
                // let code = m.arg().unwrap().to_string();
                tk_post.extend(m.arg().unwrap().clone());
            }
            MetaAttrType::AlignAfter => {
                let align = m.arg().unwrap();
                let qq = quote_align(align, &config);
                tk_post.extend(qq);
            }
            MetaAttrType::AlignBefore => {
                let align = m.arg().unwrap();
                let qq = quote_align(align, &config);
                tk_pre.extend(qq);
            }
            MetaAttrType::SkipAfter => {
                let skip = m.arg().unwrap();
                let qq = quote_skip(skip, &config);
                tk_post.extend(qq);
            }
            MetaAttrType::SkipBefore => {
                let skip = m.arg().unwrap();
                let qq = quote_skip(skip, &config);
                tk_pre.extend(qq);
            }
            MetaAttrType::Move => {
                let offset = m.arg().unwrap();
                let qq = quote_move(offset, &config);
                tk_pre.extend(qq);
            }
            MetaAttrType::MoveAbs => {
                let offset = m.arg().unwrap();
                let qq = quote_move_abs(offset, &config);
                tk_pre.extend(qq);
            }
            MetaAttrType::ErrorIf => {
                let cond = m.arg().unwrap();
                let qq = quote_error_if(cond, &config);
                tk_pre.extend(qq);
            }
            _ => (),
        }
    }
    let pre = if tk_pre.is_empty() {
        None
    } else {
        Some(tk_pre)
    };
    let post = if tk_post.is_empty() {
        None
    } else {
        Some(tk_post)
    };
    (pre, post)
}

fn add_complete(p: ParserTree, meta_list: &[MetaAttr], _config: &Config) -> ParserTree {
    if meta_list.iter().any(|m| m.is_type(MetaAttrType::Complete)) {
        return ParserTree::Complete(Box::new(p));
    }
    p
}

fn add_debug(field: &syn::Field, p: ParserTree, meta_list: &[MetaAttr], config: &Config) -> ParserTree {
    if let Some(ref ident) = field.ident {
        if config.debug || meta_list.iter().any(|m| m.is_type(MetaAttrType::Debug)) {
            let s = format!("{}::{} ({})", &config.struct_name, ident, p);
            return ParserTree::DbgDmp(Box::new(p), s);
        }
    }
    p
}

fn add_map(field: &syn::Field, p: ParserTree, meta_list: &[MetaAttr]) -> ParserTree {
    if field.ident == None { return p; }
    for meta in meta_list {
        match meta.attr_type {
            MetaAttrType::Map => {
                let s = meta.arg().unwrap().to_string();
                return ParserTree::Map(Box::new(p), s.clone());
            },
            _ => ()
        }
    }
    p
}

fn add_verify(field: &syn::Field, p: ParserTree, meta_list: &[MetaAttr]) -> ParserTree {
    if field.ident == None { return p; }
    let ident = field.ident.as_ref().expect("empty field ident (add_verify)");
    for meta in meta_list {
        match meta.attr_type {
            MetaAttrType::Verify => {
                let s = meta.arg().unwrap().to_string();
                return ParserTree::Verify(Box::new(p), format!("{}",ident), s.clone());
            },
            _ => ()
        }
    }
    p
}

fn patch_condition(field: &syn::Field, p: ParserTree, meta_list: &[MetaAttr]) -> ParserTree {
    if field.ident == None { return p; }
    let ident = field.ident.as_ref().expect("empty field ident (patch condition)");
    for meta in meta_list {
        match meta.attr_type {
            MetaAttrType::Cond => {
                match p {
                    ParserTree::Opt(sub) => {
                        let s = meta.arg().unwrap().to_string();
                        return ParserTree::Cond(sub, s.clone());
                    }
                    _ => panic!("A condition was given on field {}, which is not an option type. Hint: use Option<...>", ident),
                }
            },
            _ => (),
        }
    }
    p
}

pub(crate) fn parse_fields(f: &Fields, config: &Config) -> StructParserTree {
    let mut parsers = vec![];
    let mut unnamed = false;
    match f {
        Fields::Named(_) => (),
        Fields::Unnamed(_) => {
            unnamed = true;
        },
        Fields::Unit => panic!("Unit struct, nothing to generate")
    }
    for (idx,field) in f.iter().enumerate() {
        let ident_str = match field.ident.as_ref() {
            Some(s) => s.to_string(),
            None    => format!("_{}",idx)
        };
        let meta_list = meta::parse_nom_attribute(&field.attrs).expect("Parsing the 'nom' attribute failed");
        // eprintln!("meta_list: {:?}", meta_list);
        let opt_parser = get_parser(&field, &meta_list, config);
        let p = opt_parser.expect(&format!("Could not infer parser for field {}", ident_str));
        // add complete wrapper, if requested
        let p = add_complete(p, &meta_list, config);
        // add debug wrapper, if requested
        let p = add_debug(&field, p, &meta_list, config);
        // Check if a condition was given, and set it
        let p = patch_condition(&field, p, &meta_list);
        // add mapping function, if present
        let p = add_map(&field, p, &meta_list);
        // add verify field, if present
        let p = add_verify(&field, p, &meta_list);
        // add pre and post code (also takes care of alignment)
        let (pre, post) = get_pre_post_exec(&meta_list, config);
        let sp = StructParser::new(ident_str, p, pre, post);
        parsers.push(sp);
    }
    StructParserTree{
        unnamed,
        parsers
    }
}

pub(crate) fn parse_struct(s: &DataStruct, config: &Config) -> StructParserTree {
    parse_fields(&s.fields, config)
}
