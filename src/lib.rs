//! This crate provides Nom's derive macros.
//!
//! ```rust
//! # use nom_derive::Nom;
//! # use nom::{do_parse,IResult,be_u32,call};
//! #
//! #[derive(Nom)]
//! # struct S(u32);
//! #
//! # fn main() {}
//! ```
//!
//! For more documentation and examples, see the [Nom derive](derive.Nom.html) documentation.

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

/// The `Nom` derive automatically generates a `parse` function for the structure
/// using [nom] parsers. It will try to infer parsers for primitive of known
/// types, but also allows you to specify parsers using custom attributes.
///
/// [nom]: https://github.com/Geal/nom
///
/// # Deriving parsers
///
/// For simple structures, the parsers are automatically generated:
///
/// ```rust
/// use nom_derive::Nom;
/// use nom::{do_parse,IResult,be_u16,be_u32,call};
///
/// #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
///
/// # fn main() {
/// let input = b"\x00\x00\x00\x01\x12\x34\x56\x78";
/// let res = S::parse(input);
/// assert_eq!(res, Ok((&input[8..],S{a:1,b:0x1234,c:0x5678})));
/// # }
/// ```
///
/// This also work for tuple structs:
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,be_u16,be_u32,call};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S(u32);
/// #
/// # fn main() {
/// # let input = b"\x00\x00\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[4..],S(1))));
/// # }
/// ```
///
/// By default, integers are parsed are Big Endian.
///
/// `nom-derive` is also able to derive default parsers for some usual types:
///
/// # Option types
///
/// If a field is an `Option<T>`, the generated parser is `opt!(complete!(T::parse))`
///
/// For ex:
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,opt,complete,be_u32};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: Option<u32>
/// }
///
/// # fn main() {
/// let input = b"\x00\x00\x00\x01";
/// let res = S::parse(input);
/// assert_eq!(res, Ok((&input[4..],S{a:Some(1)})));
/// # }
/// ```
///
/// # Vec types
///
/// If a field is an `Vec<T>`, the generated parser is `many0!(complete!(T::parse))`
///
/// For ex:
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,many0,complete,be_u16};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: Vec<u16>
/// }
///
/// # fn main() {
/// let input = b"\x00\x00\x00\x01";
/// let res = S::parse(input);
/// assert_eq!(res, Ok((&input[4..],S{a:vec![0,1]})));
/// # }
/// ```
///
/// # Default parsing function
///
/// If a field with type `T` is not a primitive or known type, the generated parser is
/// `call!(T::parse)`.
///
/// This function can be automatically derived, or specified as a method for the struct.
/// In that case, the function must be a static method with the same API as a
/// [nom] combinator, returning the wrapped struct when parsing succeeds.
///
/// Example (using `Nom` derive):
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,call,be_u16};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S2 {
///   c: u16
/// }
///
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: u16,
///   b: S2
/// }
/// #
/// # fn main() {
/// # let input = b"\x00\x00\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[4..],S{a:0,b:S2{c:1}})));
/// # }
/// ```
///
/// Example (defining `parse` method):
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,call,map,be_u16,le_u16};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// // no Nom derive
/// struct S2 {
///   c: u16
/// }
///
/// impl S2 {
///     fn parse(i:&[u8]) -> IResult<&[u8],S2> {
///         map!(
///             i,
///             call!(le_u16), // little-endian
///             |c| S2{c} // return a struct S2
///             )
///     }
/// }
///
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: u16,
///   b: S2
/// }
/// #
/// # fn main() {
/// # let input = b"\x00\x00\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[4..],S{a:0,b:S2{c:256}})));
/// # }
/// ```
///
/// # Specifying parsers
///
/// Sometimes, the default parsers generated automatically are not those you
/// want.
///
/// The `Parse` custom attribute allows for specifying the parser, using code that
/// will be inserted in the `do_parse` block of the nom parser.
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,call,le_u16};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     #[Parse="le_u16"]
///     a: u16
/// }
/// #
/// # fn main() {
/// # let input = b"\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[2..],S{a:256})));
/// # }
/// ```
///
/// The `Parse` argument can be a complex expression:
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,be_u8,be_u16,call,cond};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[Parse="cond!(a > 0,be_u16)"]
///     pub b: Option<u16>,
/// }
/// #
/// # fn main() {
/// # let input = b"\x01\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[3..],S{a:1,b:Some(1)})));
/// # }
/// ```
/// Note that you are responsible from providing correct code.
///
/// # Adding conditions
///
/// The `Cond` custom attribute allows for specifying a condition.
/// The generated parser will use the `cond!` combinator, which calls the
/// child parser only if the condition is met.
/// The type with this attribute must be an `Option` type.
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,cond,complete,be_u8,be_u16,call};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[Cond="a == 1"]
///     pub b: Option<u16>,
/// }
/// #
/// # fn main() {
/// # let input = b"\x01\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[3..],S{a:1,b:Some(1)})));
/// # }
/// ```
///
/// # Adding verifications
///
/// The `Verify` custom attribute allows for specifying a verifying function.
/// The generated parser will use the `verify!` combinator, which calls the
/// child parser only if is verifies a condition (and otherwise raises an error).
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{do_parse,IResult,verify,complete,be_u8,be_u16,call};
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     #[Verify="a == 1"]
///     pub a: u8,
/// }
/// #
/// # fn main() {
/// # let input = b"\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[1..],S{a:1})));
/// # }
/// ```
///
/// # Known problems
///
/// The generated parsers use the [nom] combinators directly, so they must be
/// visible in the current namespace (*i.e* imported in a `use` statement).
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
