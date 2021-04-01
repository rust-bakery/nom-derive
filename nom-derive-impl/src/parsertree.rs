use quote::ToTokens;
use std::fmt;

#[derive(Debug)]
pub enum ParserTree {
    CallParse(String),
    Complete(Box<ParserTree>),
    Cond(Box<ParserTree>, String),
    Count(Box<ParserTree>, String),
    DbgDmp(Box<ParserTree>, String),
    LengthCount(Box<ParserTree>, String),
    Many0(Box<ParserTree>),
    Map(Box<ParserTree>, String),
    Nop,
    Opt(Box<ParserTree>),
    PhantomData,
    Raw(String),
    Tag(String),
    Take(String),
    Value(String),
    Verify(Box<ParserTree>, String, String),
}

impl fmt::Display for ParserTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserTree::CallParse(s) => write!(f, "{}::parse", s),
            ParserTree::Complete(p) => write!(f, "nom::combinator::complete({})", p),
            ParserTree::Cond(p, c) => write!(f, "nom::combinator::cond({}, {})", c, p),
            ParserTree::Count(s, n) => write!(f, "nom::multi::count({}, {{ {} }} as usize)", s, n),
            ParserTree::DbgDmp(s, c) => write!(f, "nom::dbg_dmp({}, \"{}\")", s, c),
            ParserTree::LengthCount(s, n) => write!(f, "nom::multi::length_count({}, {})", n, s),
            ParserTree::Many0(p) => write!(f, "nom::multi::many0({})", p),
            ParserTree::Map(p, m) => write!(f, "nom::combinator::map({}, {})", p, m),
            ParserTree::Nop => write!(f, "{{ |__i__| Ok((__i__, ())) }}"),
            ParserTree::Opt(p) => write!(f, "nom::combinator::opt({})", p),
            ParserTree::PhantomData => write!(f, "{{ |__i__| Ok((__i__, PhantomData)) }}"),
            ParserTree::Raw(s) => f.write_str(s),
            ParserTree::Tag(s) => write!(f, "nom::bytes::streaming::tag({})", s),
            ParserTree::Take(s) => write!(f, "nom::bytes::streaming::take({} as usize)", s),
            ParserTree::Value(s) => write!(f, "{{ |__i__| Ok((__i__, {})) }}", s),
            ParserTree::Verify(p, i, c) => {
                write!(f, "nom::combinator::verify({}, |{}| {{ {} }})", p, i, c)
            }
        }
    }
}

impl ToTokens for ParserTree {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = format!("{}", self);
        let input: proc_macro2::TokenStream = s.parse().expect("Unable to tokenize ParserTree");
        input.to_tokens(tokens);
    }
}
