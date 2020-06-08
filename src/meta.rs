#[derive(Debug, Eq, PartialEq)]
pub enum Meta {
    /// Big-endian
    Big,
    /// Little-endian
    Little,
}

#[derive(Debug)]
pub struct MetaError;

pub(crate) fn parse_nom_meta(meta: &syn::Meta) -> Result<Vec<Meta>, MetaError> {
    let mut v = Vec::new();

    match meta {
        syn::Meta::List(l) => {
            for nested in l.nested.iter() {
                // eprintln!("kw: {:?}", nested);
                match nested {
                    // Meta: like the Copy in derive(Copy)
                    syn::NestedMeta::Meta(m) => {
                        let path = m.path();
                        if let Some(ident) = path.get_ident() {
                            let m = match ident.to_string().as_ref() {
                                "Big" => Meta::Big,
                                "Little" => Meta::Little,
                                _ => return Err(MetaError),
                            };
                            v.push(m);
                        }
                    },
                    // Lit: like the "new_name in derive("new_name")
                    syn::NestedMeta::Lit(_lit) => return Err(MetaError),
                }
            }
        },
        _ => panic!("unexpected Meta value for 'nom' keyword"),
    }

    Ok(v)
}
