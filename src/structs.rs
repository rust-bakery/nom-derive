use syn::*;

use crate::config::Config;
use crate::meta;
use crate::parsertree::ParserTree;

#[derive(Debug)]
pub(crate) struct StructParserTree{
    pub unnamed: bool,
    pub parsers: Vec<(String,ParserTree)>,
}

fn get_type_parser(ty: &Type, meta_list: &[meta::Meta], config: &Config) -> Option<ParserTree> {
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
                "u32" |
                "u64" |
                "i8"  |
                "i16" |
                "i32" |
                "i64"    => {
                    let is_big_endian = if meta_list.contains(&meta::Meta::BigEndian) {
                        true
                    } else if meta_list.contains(&meta::Meta::LittleEndian) {
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

fn get_parser(field: &::syn::Field, meta_list: &[meta::Meta], config: &Config) -> Option<ParserTree> {
    // eprintln!("field: {:?}", field);
    let ty = &field.ty;
    // first check if we have attributes set
    // eprintln!("attrs: {:?}", field.attrs);
    // eprintln!("meta_list: {:?}", meta_list);
    for meta in meta_list {
        match meta {
            meta::Meta::Parse(s) => {
                return Some(ParserTree::Raw(s.clone()));
            }
            meta::Meta::Count(s) => {
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
                return Some(ParserTree::Count(s2, s.clone()));
            }
            _ => (),
        }
    }
    // else try primitive types knowledge
    get_type_parser(ty, meta_list, config)
}

fn add_verify(field: &syn::Field, p: ParserTree, meta_list: &[meta::Meta]) -> ParserTree {
    if field.ident == None { return p; }
    let ident = field.ident.as_ref().expect("empty field ident (add_verify)");
    for meta in meta_list {
        match meta {
            meta::Meta::Verify(s) => {
                return ParserTree::Verify(Box::new(p), format!("{}",ident), s.clone())
            },
            _ => ()
        }
    }
    p
}

fn patch_condition(field: &syn::Field, p: ParserTree, meta_list: &[meta::Meta]) -> ParserTree {
    if field.ident == None { return p; }
    let ident = field.ident.as_ref().expect("empty field ident (patch condition)");
    for meta in meta_list {
        match meta {
            meta::Meta::Cond(s) => {
                match p {
                    ParserTree::Opt(sub) => {
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
        let meta_list = meta::parse_nom_attribute(&field.attrs).expect("Parsing the 'nom' meta attribute failed");
        // eprintln!("meta_list: {:?}", meta_list);
        let opt_parser = get_parser(&field, &meta_list, config);
        match opt_parser {
            Some(p) => {
                // Check if a condition was given, and set it
                let p = patch_condition(&field, p, &meta_list);
                // add verify field, if present
                let p = add_verify(&field, p, &meta_list);
                parsers.push( (ident_str, p) )
            },
            None    => panic!("Could not infer parser for field {}", ident_str)
        }
    }
    StructParserTree{
        unnamed,
        parsers
    }
}

pub(crate) fn parse_struct(s: &DataStruct, config: &Config) -> StructParserTree {
    parse_fields(&s.fields, config)
}
