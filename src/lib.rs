extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Nom, attributes(Parse,Verify))]
pub fn nom(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_nom(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn get_type_parser(ty: &::syn::Ty) -> Option<String> {
    match ty {
        ::syn::Ty::Path(_, path) => {
            if path.segments.len() != 1 {
                panic!("Multiple segments in type path are not supported");
            }
            let segment = path.segments.last().unwrap();
            match segment.ident.as_ref() {
                "u8"  |
                "u16" |
                "u32" |
                "u64" |
                "i8"  |
                "i16" |
                "i32" |
                "i64"    => Some(format!("be_{}", segment.ident.as_ref())),
                "Option" => {
                    match segment.parameters {
                        ::syn::PathParameters::AngleBracketed(ref ab) => {
                            // eprintln!("Option type: {:?}", ab);
                            if ab.types.len() != 1 { panic!("Option type with multiple types are unsupported"); }
                            let s = get_type_parser(&ab.types[0]);
                            // eprintln!("    recursion: {:?}", s);
                            s.map(|x| format!("opt!(complete!({}))", x))
                        },
                        _ => panic!("Unsupported Option/parameterized type"),
                    }
                },
                "Vec"    => {
                    match segment.parameters {
                        ::syn::PathParameters::AngleBracketed(ref ab) => {
                            // eprintln!("Vec type: {:?}", ab);
                            if ab.types.len() != 1 { panic!("Vec type with multiple types are unsupported"); }
                            let s = get_type_parser(&ab.types[0]);
                            // eprintln!("    recursion: {:?}", s);
                            s.map(|x| format!("many0!({})", x))
                        },
                        _ => panic!("Unsupported Vec/parameterized type"),
                    }
                },
                s        => {
                    Some(format!("call!({}::parse)", s))
                }
            }
        },
        _ => None
    }
}

fn get_parser(field: &::syn::Field) -> Option<String> {
    let ty = &field.ty;
    // first check if we have an attribute
    for attr in &field.attrs {
        match attr.value {
            ::syn::MetaItem::NameValue(ref ident, ref lit) => {
                if &ident == &"Parse" {
                    match lit {
                        ::syn::Lit::Str(s,_) => {
                            return Some(s.to_owned())
                        },
                        _ => unimplemented!()
                    }
                }
            }
             _ => ()
        }
    }
    // else try primitive types knowledge
    get_type_parser(ty)
}

fn get_optional_lifetime(ast: &syn::DeriveInput) -> Option<String> {
    let mut res = String::from("<");
    if ast.generics.lifetimes.len() == 0 { return None; }
    for lifetime in &ast.generics.lifetimes {
        res.push_str(lifetime.lifetime.ident.as_ref());
        res.push(',');
    }
    if ast.generics.ty_params.len() > 0 { panic!("Generics not supported!"); }
    let _ = res.pop();
    res.push('>');
    Some(res)
}

fn add_verify(field: &syn::Field, p: String) -> String {
    let ident = field.ident.as_ref().unwrap();
    for attr in &field.attrs {
        match attr.value {
            ::syn::MetaItem::NameValue(ref attr_ident, ref lit) => {
                if &attr_ident == &"Verify" {
                    match lit {
                        ::syn::Lit::Str(s,_) => {
                            return format!("verify!({},|{}| {{ {} }})", p, ident, s)
                        },
                        _ => unimplemented!()
                    }
                }
            },
            _ => ()
        }
    }
    p
}

fn impl_nom(ast: &syn::DeriveInput) -> quote::Tokens {
    // eprintln!("ast: {:#?}", ast);
    let mut parsers = vec![];
    // test if struct has a lifetime
    let lifetime = ::syn::Ident::from(get_optional_lifetime(ast).unwrap_or_default().as_ref());
    // iter fields
    match &ast.body {
        &syn::Body::Enum(_)       => panic!("Enums not supported"),
        &syn::Body::Struct(ref s) => {
            match s {
                ::syn::VariantData::Struct(ref s) => {
                    for field in s.iter() {
                        let ident = field.ident.as_ref().unwrap();
                        // eprintln!("Field: {:?}", ident);
                        // eprintln!("Type: {:?}", field.ty);
                        // eprintln!("Attrs: {:?}", field.attrs);
                        let opt_parser = get_parser(&field);
                        // eprintln!("    get_parser -> {:?}", ty);
                        match opt_parser {
                            Some(p) => {
                                // add verify field, if present
                                let p = add_verify(&field, p);
                                parsers.push( (ident.as_ref(), p) )
                            },
                            None    => panic!("Could not infer parser for field {}", ident)
                        }
                    }
                },
                ::syn::VariantData::Tuple(ref _v) => {
                    unimplemented!();
                },
                ::syn::VariantData::Unit => { unimplemented!(); }
            }
        }
    }
    // Code generation
    let name = &ast.ident;
    let mut idents = vec![];
    for (ref name,_) in parsers.iter() {
        idents.push(syn::Ident::from(name.as_ref()));
    };
    let idents2 = idents.clone();
    let mut parser_idents = vec![];
    for (_, ref parser) in parsers.iter() {
        parser_idents.push(syn::Ident::from(parser.as_ref()));
    };
    let xxx = quote! {
        impl#lifetime #name#lifetime {
            fn parse(i: &[u8]) -> IResult<&[u8],#name> {
                do_parse!{
                    i,
                    #(#idents: #parser_idents >>)*
                    ( #name { #(#idents2),* } )
                }
            }
        }
    };
    // eprintln!("xxx: {:#?}", xxx);
    xxx
}
