use std::fmt;

#[derive(Debug)]
pub enum ParserTree {
    Cond(Box<ParserTree>, String),
    Verify(Box<ParserTree>, String, String),
    Complete(Box<ParserTree>),
    Opt(Box<ParserTree>),
    Many0(Box<ParserTree>),
    CallParse(String),
    Count(Box<ParserTree>, String),
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
            ParserTree::Count(s,n)      => write!(f, "count!({}, {{ {} }} as usize)", s, n),
            ParserTree::Raw(s)          => f.write_str(s)
        }
    }
}


