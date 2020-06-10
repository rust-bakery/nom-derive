use syn::Lit;

#[derive(Debug, Eq, PartialEq)]
pub enum Meta {
    Debug,
    BigEndian,
    Ignore,
    LittleEndian,
    Parse(String),
    Count(String),
    Cond(String),
    Selector(String),
    Verify(String),
}

#[derive(Debug)]
pub struct MetaError;

pub fn parse_nom_attribute(attrs: &[syn::Attribute]) -> Result<Vec<Meta>, MetaError> {
    let mut v = Vec::new();
    for attr in attrs {
        if let Some(ident) = attr.path.get_ident() {
            // eprintln!("ident: {}", ident);
            if "nom" == &ident.to_string() {
                let meta = attr
                    .parse_meta()
                    .expect("Parsing the 'nom' meta attribute failed");
                let mut res =
                    parse_nom_meta(&meta).expect("Unknown keywords in 'nom' meta attribute");
                v.append(&mut res);
            }
        } else {
            panic!("could not get ident");
        }
    }
    // XXX sort v before returning?
    Ok(v)
}

fn lit_to_string(lit: &Lit) -> Option<String> {
    if let Lit::Str(s) = lit {
        Some(s.value())
    } else {
        None
    }
}

pub(crate) fn parse_nom_meta(meta: &syn::Meta) -> Result<Vec<Meta>, MetaError> {
    let mut v = Vec::new();

    match meta {
        syn::Meta::List(l) => {
            for nested in l.nested.iter() {
                // eprintln!("kw: {:?}", nested);
                match nested {
                    // Meta: like the Copy in derive(Copy)
                    syn::NestedMeta::Meta(m) => match m {
                        syn::Meta::Path(p) => {
                            // eprintln!("path {:?}", p);
                            if let Some(ident) = p.get_ident() {
                                let m = match ident.to_string().as_ref() {
                                    "BigEndian" => Meta::BigEndian,
                                    "Debug" => Meta::Debug,
                                    "Default" => Meta::Ignore,
                                    "Ignore" => Meta::Ignore,
                                    "LittleEndian" => Meta::LittleEndian,
                                    _ => return Err(MetaError),
                                };
                                v.push(m);
                            } else {
                                eprintln!("Meta attribute is not an ident");
                                return Err(MetaError);
                            }
                        }
                        syn::Meta::List(_l) => {
                            // eprintln!("list {:?}", _l);
                            eprintln!("Unknown value for attribute nom(List ?)");
                            return Err(MetaError);
                        }
                        syn::Meta::NameValue(n) => {
                            // eprintln!("namevalue {:?}", n);
                            if let Some(ident) = n.path.get_ident() {
                                let value = match lit_to_string(&n.lit) {
                                    Some(value) => value,
                                    None => {
                                        eprintln!("Invalid value for attribute nom({})", ident);
                                        return Err(MetaError);
                                    }
                                };
                                let m = match ident.to_string().as_ref() {
                                    "Cond" => Meta::Cond(value),
                                    "Count" => Meta::Count(value),
                                    "If" => Meta::Cond(value),
                                    "Parse" => Meta::Parse(value),
                                    "Selector" => Meta::Selector(value),
                                    "Verify" => Meta::Verify(value),
                                    _ => {
                                        eprintln!("Unknown value for attribute nom({})", ident);
                                        return Err(MetaError);
                                    }
                                };
                                v.push(m);
                            }
                        }
                    },
                    // Lit: like the "new_name in derive("new_name")
                    syn::NestedMeta::Lit(_lit) => return Err(MetaError),
                }
            }
        }
        _ => panic!("unexpected Meta value for 'nom' keyword"),
    }

    Ok(v)
}
