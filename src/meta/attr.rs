use proc_macro2::TokenStream;
use quote::ToTokens;
use std::fmt;
use syn::parse::{Parse, ParseStream};
// use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parenthesized, token, Ident, Token};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetaAttrType {
    // declarative parsers
    Selector,
    Value,
    Parse,
    Count,
    Take,
    // options and combinators
    Ignore,
    Complete,
    BigEndian,
    LittleEndian,
    Cond,
    Map,
    Verify,
    PreExec,
    PostExec,
    Debug,
    DebugDerive,
    InputName,
}

impl MetaAttrType {
    pub fn from_ident(ident: &syn::Ident) -> Option<Self> {
        match ident.to_string().as_ref() {
            "BigEndian" => Some(MetaAttrType::BigEndian),
            "Complete" => Some(MetaAttrType::Complete),
            "Cond" => Some(MetaAttrType::Cond),
            "Count" => Some(MetaAttrType::Count),
            "Debug" => Some(MetaAttrType::Debug),
            "DebugDerive" => Some(MetaAttrType::DebugDerive),
            "Default" => Some(MetaAttrType::Ignore),
            "If" => Some(MetaAttrType::Cond),
            "InputName" => Some(MetaAttrType::InputName),
            "Ignore" => Some(MetaAttrType::Ignore),
            "LittleEndian" => Some(MetaAttrType::LittleEndian),
            "Map" => Some(MetaAttrType::Map),
            "Parse" => Some(MetaAttrType::Parse),
            "PostExec" => Some(MetaAttrType::PostExec),
            "PreExec" => Some(MetaAttrType::PreExec),
            "Selector" => Some(MetaAttrType::Selector),
            "Take" => Some(MetaAttrType::Take),
            "Value" => Some(MetaAttrType::Value),
            "Verify" => Some(MetaAttrType::Verify),
            _ => None,
        }
    }

    pub fn takes_argument(&self) -> bool {
        match *self {
            MetaAttrType::Cond
            | MetaAttrType::Count
            | MetaAttrType::InputName
            | MetaAttrType::Map
            | MetaAttrType::Parse
            | MetaAttrType::PostExec
            | MetaAttrType::PreExec
            | MetaAttrType::Selector
            | MetaAttrType::Take
            | MetaAttrType::Value
            | MetaAttrType::Verify => true,
            _ => false,
        }
    }
}

impl fmt::Display for MetaAttrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MetaAttrType::BigEndian => "BigEndian",
            MetaAttrType::Complete => "Complete",
            MetaAttrType::Cond => "Cond",
            MetaAttrType::Count => "Count",
            MetaAttrType::Debug => "Debug",
            MetaAttrType::DebugDerive => "DebugDerive",
            MetaAttrType::InputName => "InputName",
            MetaAttrType::Ignore => "Ignore",
            MetaAttrType::LittleEndian => "LittleEndian",
            MetaAttrType::Map => "Map",
            MetaAttrType::Parse => "Parse",
            MetaAttrType::PostExec => "PostExec",
            MetaAttrType::PreExec => "PreExec",
            MetaAttrType::Selector => "Selector",
            MetaAttrType::Take => "Take",
            MetaAttrType::Value => "Value",
            MetaAttrType::Verify => "Verify",
        };
        f.write_str(s)
    }
}

#[derive(Debug)]
pub struct MetaAttr {
    pub attr_type: MetaAttrType,
    arg0: Option<TokenStream>,
}

impl MetaAttr {
    pub fn new(attr_type: MetaAttrType, arg0: Option<TokenStream>) -> Self {
        MetaAttr { attr_type, arg0 }
    }

    pub fn acceptable_tla(&self) -> bool {
        match self.attr_type {
            MetaAttrType::DebugDerive
            | MetaAttrType::Debug
            | MetaAttrType::InputName
            | MetaAttrType::LittleEndian
            | MetaAttrType::BigEndian
            | MetaAttrType::Selector => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_type(&self, attr_type: MetaAttrType) -> bool {
        self.attr_type == attr_type
    }

    #[inline]
    pub fn arg(&self) -> Option<&TokenStream> {
        self.arg0.as_ref()
    }
}

impl fmt::Display for MetaAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.attr_type)?;
        if let Some(arg) = &self.arg0 {
            write!(f, "({})", arg)?;
        }
        Ok(())
    }
}

impl Parse for MetaAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let attr_type =
            MetaAttrType::from_ident(&ident).expect(&format!("Wrong meta name {}", ident));
        let arg0 = if attr_type.takes_argument() {
            // read (value), or ="value"
            let token_stream = match attr_type {
                MetaAttrType::PreExec | MetaAttrType::PostExec => {
                    parse_content::<syn::Stmt>(input)?
                }
                MetaAttrType::Selector => parse_content::<syn::Pat>(input)?,
                _ => parse_content::<syn::Expr>(input)?,
            };
            Some(token_stream)
        } else {
            None
        };
        Ok(MetaAttr::new(attr_type, arg0))
    }
}

fn parse_content<P>(input: ParseStream) -> syn::Result<TokenStream>
where
    P: Parse + ToTokens + fmt::Debug,
{
    if input.peek(Token![=]) {
        // eprintln!("Exec Peek: =");
        let _: Token![=] = input.parse()?;
        // next item is a string containing the real value
        let x = syn::Lit::parse(input)?;
        // eprintln!("content: {:?}", x);
        match x {
            syn::Lit::Str(s) => {
                let xx: P = s.parse()?;
                // eprintln!("xx: {:?}", xx);
                Ok(quote! { #xx })
            }
            _ => Err(syn::Error::new(
                x.span(),
                "Unexpected type for nom attribute content (!LitStr)",
            )),
        }
    } else if input.peek(token::Paren) {
        // eprintln!("Exec Peek: (");
        let content;
        let _paren_token = parenthesized!(content in input);
        let x = P::parse(&content)?;
        // let x: Punctuated<Type, Token![,]> = content.parse_terminated(Type::parse)?;
        // eprintln!("content: {:?}", x);
        Ok(quote! { #x })
    } else {
        Err(syn::Error::new(
            input.span(),
            "Unexpected type for nom attribute content (!LitStr)",
        ))
    }
}
