use nom_derive::{Nom, Parse, Parser};

#[derive(Nom)]
// #[nom(DebugDerive)]
pub struct RawIdent {
    pub r#type: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Nom)]
pub struct MessageType(pub u8);

/// An enum with named fields
#[derive(Debug, PartialEq, Nom)]
#[nom(Selector = "MessageType")]
pub enum U2 {
    #[nom(Selector = "MessageType(0)")]
    Field1 { r#type: u32 },
    #[nom(Selector = "MessageType(1)")]
    Field2 { a: Option<u32> },
}
