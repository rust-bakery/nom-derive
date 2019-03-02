use syn::*;

use crate::parsertree::ParserTree;

#[derive(Debug)]
pub(crate) struct StructParserTree{
    pub unnamed: bool,
    pub parsers: Vec<(String,ParserTree)>,
}

fn get_type_parser(ty: &Type) -> Option<ParserTree> {
    match ty {
        Type::Path(ref typepath) => {
            let path = &typepath.path;
            if path.segments.len() != 1 {
                panic!("Multiple segments in type path are not supported");
            }
            let pair = path.segments.last().expect("empty segments list");
            let segment = pair.value();
            let ident_s = segment.ident.to_string();
            match ident_s.as_ref() {
                "u8"  |
                "u16" |
                "u32" |
                "u64" |
                "i8"  |
                "i16" |
                "i32" |
                "i64"    => Some(ParserTree::Raw(format!("be_{}", ident_s))),
                "Option" => {
                    match segment.arguments {
                        PathArguments::AngleBracketed(ref ab) => {
                            // eprintln!("Option type: {:?}", ab);
                            if ab.args.len() != 1 { panic!("Option type with multiple types are unsupported"); }
                            match &ab.args[0] {
                                GenericArgument::Type(ref ty) => {
                                    let s = get_type_parser(ty);
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
                                    let s = get_type_parser(ty);
                                    // eprintln!("    recursion: {:?}", s);
                                    s.map(|x| ParserTree::Many0(Box::new(ParserTree::Complete(Box::new(x)))))
                                },
                                _ => panic!("Vec generic argument is not a type")
                            }
                        },
                        _ => panic!("Unsupported Vec/parameterized type"),
                    }
                },
                s        => {
                    Some(ParserTree::CallParse(s.to_owned()))
                }
            }
        },
        _ => None
    }
}

fn get_parser(field: &::syn::Field) -> Option<ParserTree> {
    // eprintln!("field: {:?}", field);
    let ty = &field.ty;
    // first check if we have an attribute
    // eprintln!("attrs: {:?}", field.attrs);
    for attr in &field.attrs {
        // eprintln!("meta: {:?}", attr.parse_meta());
        if let Ok(ref meta) = attr.parse_meta() {
            match meta {
                Meta::NameValue(ref namevalue) => {
                    if &namevalue.ident == &"Parse" {
                        match &namevalue.lit {
                            Lit::Str(s) => {
                                return Some(ParserTree::Raw(s.value()))
                            },
                            _ => panic!("Invalid 'Parse' attribute type/value")
                        }
                    }
                    if &namevalue.ident == &"Count" {
                        match &namevalue.lit {
                            Lit::Str(s) => {
                                // try to infer subparser
                                let sub = get_type_parser(ty);
                                let s1 = match sub {
                                    Some(ParserTree::Many0(m)) => { m },
                                    _ => panic!("Unable to infer parser for 'Count' attribute. Is item type a Vec ?")
                                };
                                let s2 = match *s1 {
                                    ParserTree::Complete(m) => { m },
                                    _ => panic!("Unable to infer parser for 'Count' attribute. Is item type a Vec ?")
                                };
                                return Some(ParserTree::Count(s2, s.value()));
                            },
                            _ => panic!("Invalid 'Count' attribute type/value")
                        }
                    }
                }
                _ => ()
            }
        }
    }
    // else try primitive types knowledge
    get_type_parser(ty)
}

fn add_verify(field: &syn::Field, p: ParserTree) -> ParserTree {
    if field.ident == None { return p; }
    let ident = field.ident.as_ref().expect("empty field ident (add_verify)");
    for attr in &field.attrs {
        if let Ok(ref meta) = attr.parse_meta() {
            match meta {
                Meta::NameValue(ref namevalue) => {
                    if &namevalue.ident == &"Verify" {
                        match &namevalue.lit {
                            Lit::Str(s) => {
                                return ParserTree::Verify(Box::new(p), format!("{}",ident), s.value())
                            },
                            _ => panic!("Invalid 'Verify' attribute type/value")
                        }
                    }
                },
                _ => ()
            }
        }
    }
    p
}

fn patch_condition(field: &syn::Field, p: ParserTree) -> ParserTree {
    if field.ident == None { return p; }
    let ident = field.ident.as_ref().expect("empty field ident (patch condition)");
    for attr in &field.attrs {
        if let Ok(ref meta) = attr.parse_meta() {
            match meta {
                Meta::NameValue(ref namevalue) => {
                    if &namevalue.ident == &"Cond" {
                        match &namevalue.lit {
                            Lit::Str(s) => {
                                match p {
                                    ParserTree::Opt(sub) => {
                                        return ParserTree::Cond(sub, s.value());
                                    }
                                    _ => panic!("A condition was given on field {}, which is not an option type. Hint: use Option<...>", ident),
                                }
                            },
                            _ => panic!("Invalid 'Cond' attribute type/value")
                        }
                    }
                },
                _ => ()
            }
        }
    }
    p
}

pub(crate) fn parse_fields(f: &Fields) -> StructParserTree {
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
        // eprintln!("Field: {:?}", ident);
        // eprintln!("Type: {:?}", field.ty);
        // eprintln!("Attrs: {:?}", field.attrs);
        let opt_parser = get_parser(&field);
        // eprintln!("    get_parser -> {:?}", ty);
        match opt_parser {
            Some(p) => {
                // Check if a condition was given, and set it
                let p = patch_condition(&field, p);
                // add verify field, if present
                let p = add_verify(&field, p);
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

pub(crate) fn parse_struct(s: &DataStruct) -> StructParserTree {
    parse_fields(&s.fields)
}
