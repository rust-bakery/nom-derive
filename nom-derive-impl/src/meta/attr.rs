use core::convert::TryFrom;
use core::fmt;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{token, Expr, ExprLit, FnArg, Ident, Lit, Meta, Pat, Stmt, Token};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetaAttrType {
    AlignAfter,
    AlignBefore,
    BigEndian,
    Complete,
    Cond,
    Count,
    Debug,
    DebugDerive,
    ErrorIf,
    Exact,
    ExtraArgs,
    GenericErrors,
    Ignore,
    InputName,
    Into,
    LengthCount,
    LittleEndian,
    Map,
    Move,
    MoveAbs,
    Parse,
    PostExec,
    PreExec,
    Selector,
    SetEndian,
    SkipAfter,
    SkipBefore,
    Tag,
    Take,
    Value,
    Verify,
}

impl MetaAttrType {
    pub fn from_ident(ident: &syn::Ident) -> Option<Self> {
        match ident.to_string().as_ref() {
            "AlignAfter" => Some(MetaAttrType::AlignAfter),
            "AlignBefore" => Some(MetaAttrType::AlignBefore),
            "BigEndian" => Some(MetaAttrType::BigEndian),
            "Complete" => Some(MetaAttrType::Complete),
            "Count" => Some(MetaAttrType::Count),
            "Debug" => Some(MetaAttrType::Debug),
            "DebugDerive" => Some(MetaAttrType::DebugDerive),
            "ErrorIf" => Some(MetaAttrType::ErrorIf),
            "Exact" => Some(MetaAttrType::Exact),
            "ExtraArgs" => Some(MetaAttrType::ExtraArgs),
            "GenericErrors" => Some(MetaAttrType::GenericErrors),
            "If" | "Cond" => Some(MetaAttrType::Cond),
            "Ignore" | "Default" => Some(MetaAttrType::Ignore),
            "InputName" => Some(MetaAttrType::InputName),
            "Into" => Some(MetaAttrType::Into),
            "LengthCount" => Some(MetaAttrType::LengthCount),
            "LittleEndian" => Some(MetaAttrType::LittleEndian),
            "Map" => Some(MetaAttrType::Map),
            "Move" => Some(MetaAttrType::Move),
            "MoveAbs" => Some(MetaAttrType::MoveAbs),
            "Parse" => Some(MetaAttrType::Parse),
            "PostExec" => Some(MetaAttrType::PostExec),
            "PreExec" => Some(MetaAttrType::PreExec),
            "Selector" => Some(MetaAttrType::Selector),
            "SetEndian" => Some(MetaAttrType::SetEndian),
            "SkipAfter" => Some(MetaAttrType::SkipAfter),
            "SkipBefore" => Some(MetaAttrType::SkipBefore),
            "Tag" => Some(MetaAttrType::Tag),
            "Take" => Some(MetaAttrType::Take),
            "Value" => Some(MetaAttrType::Value),
            "Verify" => Some(MetaAttrType::Verify),
            _ => None,
        }
    }

    pub fn takes_argument(self) -> bool {
        matches!(
            self,
            MetaAttrType::AlignAfter
                | MetaAttrType::AlignBefore
                | MetaAttrType::Cond
                | MetaAttrType::Count
                | MetaAttrType::ErrorIf
                | MetaAttrType::ExtraArgs
                | MetaAttrType::InputName
                | MetaAttrType::LengthCount
                | MetaAttrType::Map
                | MetaAttrType::Move
                | MetaAttrType::MoveAbs
                | MetaAttrType::Parse
                | MetaAttrType::PostExec
                | MetaAttrType::PreExec
                | MetaAttrType::Selector
                | MetaAttrType::SetEndian
                | MetaAttrType::SkipAfter
                | MetaAttrType::SkipBefore
                | MetaAttrType::Tag
                | MetaAttrType::Take
                | MetaAttrType::Value
                | MetaAttrType::Verify
        )
    }
}

impl fmt::Display for MetaAttrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MetaAttrType::AlignAfter => "AlignAfter",
            MetaAttrType::AlignBefore => "AlignBefore",
            MetaAttrType::BigEndian => "BigEndian",
            MetaAttrType::Complete => "Complete",
            MetaAttrType::Cond => "Cond",
            MetaAttrType::Count => "Count",
            MetaAttrType::Debug => "Debug",
            MetaAttrType::DebugDerive => "DebugDerive",
            MetaAttrType::ErrorIf => "ErrorIf",
            MetaAttrType::Exact => "Exact",
            MetaAttrType::ExtraArgs => "ExtraArgs",
            MetaAttrType::GenericErrors => "GenericErrors",
            MetaAttrType::Ignore => "Ignore",
            MetaAttrType::InputName => "InputName",
            MetaAttrType::Into => "Into",
            MetaAttrType::LengthCount => "LengthCount",
            MetaAttrType::LittleEndian => "LittleEndian",
            MetaAttrType::Map => "Map",
            MetaAttrType::Move => "Move",
            MetaAttrType::MoveAbs => "MoveAbs",
            MetaAttrType::Parse => "Parse",
            MetaAttrType::PostExec => "PostExec",
            MetaAttrType::PreExec => "PreExec",
            MetaAttrType::Selector => "Selector",
            MetaAttrType::SetEndian => "SetEndian",
            MetaAttrType::SkipAfter => "SkipAfter",
            MetaAttrType::SkipBefore => "SkipBefore",
            MetaAttrType::Tag => "Tag",
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

    // the original token stream for the Meta attribute
    tokens: TokenStream,
}

impl MetaAttr {
    pub fn new(attr_type: MetaAttrType, arg0: Option<TokenStream>, tokens: TokenStream) -> Self {
        MetaAttr {
            attr_type,
            arg0,
            tokens,
        }
    }

    /// Is attribute acceptable for top-level
    pub fn acceptable_tla(&self) -> bool {
        matches!(
            self.attr_type,
            MetaAttrType::DebugDerive
                | MetaAttrType::Complete
                | MetaAttrType::Debug
                | MetaAttrType::ExtraArgs
                | MetaAttrType::GenericErrors
                | MetaAttrType::InputName
                | MetaAttrType::LittleEndian
                | MetaAttrType::BigEndian
                | MetaAttrType::SetEndian
                | MetaAttrType::PreExec
                | MetaAttrType::PostExec
                | MetaAttrType::Exact
                | MetaAttrType::Selector
        )
    }

    /// Is attribute acceptable for field-level
    pub fn acceptable_fla(&self) -> bool {
        !matches!(
            self.attr_type,
            MetaAttrType::DebugDerive
                | MetaAttrType::Exact
                | MetaAttrType::ExtraArgs
                | MetaAttrType::GenericErrors
                | MetaAttrType::InputName
        )
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

impl TryFrom<&'_ Meta> for MetaAttr {
    type Error = syn::Error;

    fn try_from(meta: &Meta) -> std::result::Result<Self, Self::Error> {
        let ident: Ident = meta.path().get_ident().cloned().unwrap();
        let attr_type =
            MetaAttrType::from_ident(&ident).unwrap_or_else(|| panic!("Wrong meta name {}", ident));

        let arg0 = if attr_type.takes_argument() {
            let token_stream = match attr_type {
                MetaAttrType::ExtraArgs => {
                    let list = meta.require_list()?;
                    let fields =
                        list.parse_args_with(Punctuated::<FnArg, Token![,]>::parse_terminated)?;
                    quote! { #fields }
                }
                MetaAttrType::PreExec | MetaAttrType::PostExec => parse_meta_content::<Stmt>(meta)?,
                MetaAttrType::Selector => parse_meta_content::<PatternAndGuard>(meta)?,
                _ => parse_meta_content::<Expr>(meta)?,
            };
            Some(token_stream)
        } else {
            None
        };

        Ok(MetaAttr::new(attr_type, arg0, meta.to_token_stream()))
    }
}

// Implemented to provided `Spanned``
impl quote::ToTokens for MetaAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.tokens.clone());
    }
}

impl Parse for MetaAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta: Meta = input.parse()?;
        Self::try_from(&meta)
    }
}

fn parse_meta_content<P>(meta: &Meta) -> syn::Result<TokenStream>
where
    P: Parse + ToTokens + fmt::Debug,
{
    match meta {
        Meta::Path(_path) => Err(syn::Error::new(
            meta.span(),
            "Unexpected type for nom attribute content (!LitStr)",
        )),
        Meta::List(meta_list) => {
            // let lit_str: LitStr = meta_list.parse_args()?;

            // let expr: Expr = lit_str.parse()?;
            // quote! { #expr }
            Ok(meta_list.tokens.clone())
        }
        Meta::NameValue(meta_name_value) => {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) = &meta_name_value.value
            {
                let p: P = lit_str.parse()?;
                Ok(quote! { #p })
            } else {
                Err(syn::Error::new(
                    meta.span(),
                    "Unexpected type for nom attribute content (!LitStr)",
                ))
            }
        }
    }
}

#[derive(Debug)]
struct PatternAndGuard {
    pat: syn::Pat,
    guard: Option<(token::If, Box<syn::Expr>)>,
}

impl Parse for PatternAndGuard {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = Pat::parse_single(input)?;
        let guard = if input.peek(Token![if]) {
            let tk_if: Token![if] = input.parse()?;
            let expr: syn::Expr = input.parse()?;
            Some((tk_if, Box::new(expr)))
        } else {
            None
        };
        Ok(PatternAndGuard { pat, guard })
    }
}

impl quote::ToTokens for PatternAndGuard {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pat.to_tokens(tokens);
        if let Some((tk_if, expr)) = &self.guard {
            tk_if.to_tokens(tokens);
            expr.to_tokens(tokens);
        }
    }
}
