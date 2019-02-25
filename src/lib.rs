extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::*;
use syn::export::Span;


use std::fmt;

enum ParserTree {
    Cond(Box<ParserTree>, String),
    Verify(Box<ParserTree>, String, String),
    Complete(Box<ParserTree>),
    Opt(Box<ParserTree>),
    Many0(Box<ParserTree>),
    CallParse(String),
    Raw(String)
}

impl fmt::Display for ParserTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserTree::Cond(p, c)      => write!(f, "cond!({}, {})", c, p),
            ParserTree::Verify(p, i, c) => write!(f, "verify!({}, |{}| {{ {} }})", p, i, c),
            ParserTree::Complete(p)     => write!(f, "complete!({})", p),
            ParserTree::Opt(p)          => write!(f, "opt!({})", p),
            ParserTree::Many0(p)        => write!(f, "many0!({})", p),
            ParserTree::CallParse(s)    => write!(f, "call!({}::parse)", s),
            ParserTree::Raw(s)          => f.write_str(s)
        }
    }
}

#[proc_macro_derive(Nom, attributes(Parse,Verify,Cond))]
pub fn nom(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build the impl
    let gen = impl_nom(&ast);

    // Return the generated impl
    gen
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
    let ty = &field.ty;
    // first check if we have an attribute
    for attr in &field.attrs {
        // eprintln!("attr: {:?}", attr);
        // eprintln!("meta: {:?}", attr.parse_meta());
        if let Ok(ref meta) = attr.parse_meta() {
            match meta {
                Meta::NameValue(ref namevalue) => {
                    if &namevalue.ident == &"Parse" {
                        match &namevalue.lit {
                            Lit::Str(s) => {
                                return Some(ParserTree::Raw(s.value()))
                            },
                            _ => unimplemented!()
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
                            _ => unimplemented!()
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
                            _ => unimplemented!()
                        }
                    }
                },
                _ => ()
            }
        }
    }
    p
}

fn impl_nom(ast: &syn::DeriveInput) -> TokenStream {
    // eprintln!("ast: {:#?}", ast);
    let mut parsers = vec![];
    let mut is_unnamed = false;
    // test if struct has a lifetime
    let generics = &ast.generics;
    // iter fields
    match &ast.data {
        &syn::Data::Enum(_)       => panic!("Enums not supported"),
        &syn::Data::Struct(ref s) => {
            // eprintln!("s: {:?}", ast.data);
            match s.fields {
                syn::Fields::Named(_) => (),
                syn::Fields::Unnamed(_) => {
                    is_unnamed = true;
                },
                syn::Fields::Unit => {
                    panic!("unit struct, nothing to parse");
                }
            }
            for (idx,field) in s.fields.iter().enumerate() {
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
        }
        &syn::Data::Union(_)       => panic!("Unions not supported"),
    }
    // Code generation
    let name = &ast.ident;
    let mut idents = vec![];
    for (ref name,_) in parsers.iter() {
        idents.push(Ident::new(name.as_ref(), Span::call_site()));
    };
    let idents2 = idents.clone();
    let struct_def = match is_unnamed {
        false => quote!{ ( #name { #(#idents2),* } ) },
        true  => quote!{ ( #name ( #(#idents2),* ) ) },
    };
    let mut parser_idents = vec![];
    for (_, ref parser) in parsers.iter() {
        let s = format!("{}",parser);
        let tts : TokenStream = s.parse().unwrap();
        let input = proc_macro2::TokenStream::from(tts);
        parser_idents.push(input);
    };
    // eprintln!("idents: {:?}", idents);
    // eprintln!("parser_idents: {:?}", parser_idents);
    let tokens = quote! {
        impl#generics #name#generics {
            fn parse(i: &[u8]) -> IResult<&[u8],#name> {
                do_parse!{
                    i,
                    #(#idents: #parser_idents >>)*
                    #struct_def
                }
            }
        }
    };
    // eprintln!("tokens: {:#?}", tokens);
    // eprintln!("tokens: {}", tokens);
    tokens.into()
}
