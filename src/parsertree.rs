use std::fmt;
use quote::ToTokens;

#[derive(Debug)]
pub enum ParserTree {
    Cond(Box<ParserTree>, String),
    Verify(Box<ParserTree>, String, String),
    Complete(Box<ParserTree>),
    Opt(Box<ParserTree>),
    Many0(Box<ParserTree>),
    Map(Box<ParserTree>, String),
    CallParse(String),
    Count(Box<ParserTree>, String),
    Raw(String),
    Value(String),
    PhantomData,

}

impl fmt::Display for ParserTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserTree::Cond(p, c)      => write!(f, "nom::combinator::cond({}, {})", c, p),
            ParserTree::Verify(p, i, c) => write!(f, "nom::combinator::verify({}, |{}| {{ {} }})", p, i, c),
            ParserTree::Complete(p)     => write!(f, "nom::combinator::complete({})", p),
            ParserTree::Map(p, m)       => write!(f, "nom::combinator::map({}, {})", p, m),
            ParserTree::Opt(p)          => write!(f, "nom::combinator::opt({})", p),
            ParserTree::Many0(p)        => write!(f, "nom::multi::many0({})", p),
            ParserTree::CallParse(s)    => write!(f, "{}::parse", s),
            ParserTree::Count(s,n)      => write!(f, "nom::multi::count({}, {{ {} }} as usize)", s, n),
            ParserTree::PhantomData     => write!(f, "{{ |i| Ok((i, PhantomData)) }}"),
            ParserTree::Raw(s)          => f.write_str(s),
            ParserTree::Value(s)        => write!(f, "{{ |i| Ok((i, {})) }}", s),
        }
    }
}

impl ToTokens for ParserTree {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = format!("{}",self);
        let input : proc_macro2::TokenStream = s.parse().expect("Unable to tokenize ParserTree");
        input.to_tokens(tokens);
    }
}
