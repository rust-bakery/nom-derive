#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

use nom::*;
use nom::bytes::streaming::take;
use nom::combinator::{complete, opt};
use nom::number::streaming::*;

#[derive(Debug,PartialEq,Eq,Clone,Copy,Nom)]
pub struct MessageType(pub u8);

/// An enum with unnamed fields
#[derive(Debug,PartialEq,Nom)]
#[nom(Selector="MessageType")]
pub enum U1{
    #[nom(Selector="MessageType(0)")] Field1(u32),
    #[nom(Selector="MessageType(1)")] Field2(Option<u32>),
}

/// An enum with unnamed fields and primitive selector
#[derive(Debug,PartialEq,Nom)]
#[nom(Selector="u8")]
pub enum U1b{
    #[nom(Selector="0")] Field1(u32),
    #[nom(Selector="1")] Field2(Option<u32>),
}

/// An structure containing an enum
#[derive(Debug,PartialEq,Nom)]
pub struct S1{
    pub msg_type: MessageType,
    #[nom(Parse="{ |i| U1::parse(i, msg_type) }")]
    pub msg_value: U1
}

/// An enum with named fields
#[derive(Debug,PartialEq,Nom)]
#[nom(Selector="MessageType")]
pub enum U2{
    #[nom(Selector="MessageType(0)")] Field1{ a: u32 },
    #[nom(Selector="MessageType(1)")] Field2{ a:Option<u32> },
}

/// An enum with lifetime
#[derive(Debug,PartialEq,Nom)]
#[nom(Selector="MessageType")]
pub enum U3<'a>{
    #[nom(Selector="MessageType(0)")] Field1(u32),
    // next variant has to be annotated for parsing (inside variant definition, not outside!)
    #[nom(Selector="MessageType(1)")]
    Field2(#[nom(Parse="take(4 as usize)")] &'a[u8]),
}

// /// An enum with fields and Parse attribute
// #[derive(Debug,PartialEq,Nom)]
// #[Selector="MessageType"]
// pub enum U4{
//     #[Selector("MessageType(0)")] Field1{ a: u32 },
//     #[Selector("MessageType(1)")]
//     #[Parse="be_u32"] // XXX unsupported
//     Field2,
// }

/// An enum with a default case
#[derive(Debug,PartialEq,Nom)]
#[nom(Selector="MessageType")]
pub enum U5{
    #[nom(Selector="MessageType(0)")] Field1(u32),
    #[nom(Selector="_")] Field2(Option<u32>),
}

/// A fieldless enum with values
#[derive(Debug,PartialEq,Nom)]
#[repr(u8)]
pub enum U6{
    A,
    B = 2,
    C,
}

/// An enum with a default case, before the end
#[derive(Debug,PartialEq,Nom)]
#[nom(Selector="MessageType")]
pub enum U7{
    #[nom(Selector="_")] Field2(u32),
    #[nom(Selector="MessageType(0)")] Field1(u32),
}

/// An unnamed enum with a structure in fields (common case)
#[derive(Debug,PartialEq,Nom)]
#[nom(Selector="u8")]
pub enum U8 {
    #[nom(Selector="0")] Field1(U8S1),
    #[nom(Selector="1")] Field2(u32),
}

#[derive(Debug,PartialEq,Nom)]
pub struct U8S1 {
    pub a: u32,
}


#[test]
fn test_enum_unnamed() {
    let input = b"\x00\x00\x00\x02";
    let res = U1::parse(input, MessageType(0));
    assert_eq!(res, Ok((&input[4..],U1::Field1(2))));
    let res = U1::parse(input, MessageType(1));
    assert_eq!(res, Ok((&input[4..],U1::Field2(Some(2)))));
    let res = U1b::parse(input, 0);
    assert_eq!(res, Ok((&input[4..],U1b::Field1(2))));
}

#[test]
fn test_enum_named() {
    let input = b"\x00\x00\x00\x02";
    let res = U2::parse(input, MessageType(0));
    assert_eq!(res, Ok((&input[4..],U2::Field1{a:2})));
    let res = U2::parse(input, MessageType(1));
    assert_eq!(res, Ok((&input[4..],U2::Field2{a:Some(2)})));
}

#[test]
fn test_enum_in_struct() {
    let input = b"\x00\x00\x00\x00\x02";
    let res = S1::parse(input);
    assert_eq!(res, Ok((&input[5..],
                        S1{msg_type:MessageType(0), msg_value:U1::Field1(2)}
                        )));
}

#[test]
fn test_enum_match_default() {
    let input = b"\x00\x00\x00\x02";
    let res = U5::parse(input, MessageType(123));
    assert_eq!(res, Ok((&input[4..],U5::Field2(Some(2)))));
}

#[test]
fn test_enum_fieldless() {
    let empty : &[u8] = b"";
    assert_eq!(
        U6::parse(b"\x00"),
        Ok((empty,U6::A))
    );
    assert!(
        U6::parse(b"\x01").is_err()
    );
    assert_eq!(
        U6::parse(b"\x02"),
        Ok((empty,U6::B))
    );
}

#[test]
fn test_enum_match_default_before_end() {
    let input = b"\x00\x00\x00\x02";
    let res = U7::parse(input, MessageType(123));
    assert_eq!(res, Ok((&input[4..],U7::Field2(2))));
    let res = U7::parse(input, MessageType(0));
    assert_eq!(res, Ok((&input[4..],U7::Field1(2))));
}

#[test]
fn test_struct_in_enum() {
    let input = b"\x00\x00\x00\x02";
    let res = U8::parse(input, 0);
    assert_eq!(res, Ok((&input[4..],
                        U8::Field1(U8S1{a:2})
                        )));
    let res = U8::parse(input, 1);
    assert_eq!(res, Ok((&input[4..],
                        U8::Field2(2)
                        )));
}
