#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

extern crate nom;

use nom::*;

#[derive(Debug,PartialEq,Eq,Clone,Copy,Nom)]
pub struct MessageType(pub u8);

/// An union with unnamed fields
#[derive(Debug,PartialEq,Nom)]
#[Selector="MessageType"]
pub enum U1{
    #[Selector("MessageType(0)")] Field1(u32),
    #[Selector("MessageType(1)")] Field2(Option<u32>),
}

/// An structure containing an union
#[derive(Debug,PartialEq,Nom)]
pub struct S1{
    pub msg_type: MessageType,
    #[Parse="call!(U1::parse,msg_type)"]
    pub msg_value: U1
}

/// An union with named fields
#[derive(Debug,PartialEq,Nom)]
#[Selector="MessageType"]
pub enum U2{
    #[Selector("MessageType(0)")] Field1{ a: u32 },
    #[Selector("MessageType(1)")] Field2{ a:Option<u32> },
}

/// An union with lifetime
#[derive(Debug,PartialEq,Nom)]
#[Selector="MessageType"]
pub enum U3<'a>{
    #[Selector("MessageType(0)")] Field1(u32),
    // next variant has to be annotated for parsing (inside variant definition, not outside!)
    #[Selector("MessageType(1)")]
    Field2(#[Parse="take!(4)"] &'a[u8]),
}

// /// An union with fields and Parse attribute
// #[derive(Debug,PartialEq,Nom)]
// #[Selector="MessageType"]
// pub enum U4{
//     #[Selector("MessageType(0)")] Field1{ a: u32 },
//     #[Selector("MessageType(1)")]
//     #[Parse="be_u32"] // XXX unsupported
//     Field2,
// }

#[test]
fn test_union_unnamed() {
    let input = b"\x00\x00\x00\x02";
    let res = U1::parse(input, MessageType(0));
    assert_eq!(res, Ok((&input[4..],U1::Field1(2))));
    let res = U1::parse(input, MessageType(1));
    assert_eq!(res, Ok((&input[4..],U1::Field2(Some(2)))));
}

#[test]
fn test_union_named() {
    let input = b"\x00\x00\x00\x02";
    let res = U2::parse(input, MessageType(0));
    assert_eq!(res, Ok((&input[4..],U2::Field1{a:2})));
    let res = U2::parse(input, MessageType(1));
    assert_eq!(res, Ok((&input[4..],U2::Field2{a:Some(2)})));
}

#[test]
fn test_union_in_struct() {
    let input = b"\x00\x00\x00\x00\x02";
    let res = S1::parse(input);
    assert_eq!(res, Ok((&input[5..],
                        S1{msg_type:MessageType(0), msg_value:U1::Field1(2)}
                        )));
}
