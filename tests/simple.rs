#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

use nom::*;
use nom::number::streaming::*;

/// A simple structure, deriving a trivial parser
#[derive(Debug,PartialEq,Nom)]
struct SimpleStruct {
    pub a: u32,
    b: u64,
}

/// A simple structure, giving the parser explicitly
#[derive(Debug,PartialEq,Nom)]
struct StructWithParser {
    #[Parse="le_u32"]
    pub a: u32,
}

/// A simple structure, giving the parser explicitly
#[derive(Debug,PartialEq,Nom)]
struct StructWithParser2 {
    #[Parse="opt!(le_u32)"]
    pub a: Option<u32>,
}

/// A structure containing a substructure
#[derive(Debug,PartialEq,Nom)]
struct StructWithSubStruct {
    pub a: u32,
    b: u64,
    s: StructWithParser,
}

/// A simple structure with a verification
#[derive(Debug,PartialEq,Nom)]
struct StructWithVerify {
    #[Verify="*a == 1"]
    pub a: u32,
}

/// A simple structure with a condition
#[derive(Debug,PartialEq,Nom)]
struct StructWithCondition {
    pub a: u32,
    #[Cond="a == 1"]
    pub b: Option<u32>,
}

/// A tuple struct with one field (newtype)
#[derive(Debug,PartialEq,Nom)]
struct NewType(pub u32);

/// A tuple struct with two fields (newtype)
#[derive(Debug,PartialEq,Nom)]
struct NewType2(pub u32, pub u16);

#[test]
fn test_simple_struct() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = SimpleStruct::parse(input);
    assert_eq!(res, Ok((&input[12..],SimpleStruct{a:1, b:0x1234567812345678})));
}

#[test]
fn test_struct_parse() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithParser::parse(input);
    assert_eq!(res, Ok((&input[4..],StructWithParser{a:0x01000000})));
}

#[test]
fn test_struct_parse_substruct() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithSubStruct::parse(input);
    assert_eq!(res, Ok((&input[16..],StructWithSubStruct{a:1,b:0x1234567812345678,s:StructWithParser{a:0x01000000}})));
}

#[test]
fn test_struct_with_verify() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithVerify::parse(input);
    assert_eq!(res, Ok((&input[4..],StructWithVerify{a:1})));

    let res = StructWithVerify::parse(&input[4..]);
    assert_eq!(res, Err(Err::Error(error_position!(&input[4..], nom::error::ErrorKind::Verify))));
}

#[test]
fn test_struct_with_condition() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithCondition::parse(input);
    assert_eq!(res, Ok((&input[8..],StructWithCondition{a:1,b:Some(0x12345678)})));

    let res = StructWithCondition::parse(&input[4..]);
    assert_eq!(res, Ok((&input[8..],StructWithCondition{a:0x12345678,b:None})));
}

#[test]
fn test_tuple_struct_1() {
    let input = b"\x00\x00\x00\x01";
    let res = NewType::parse(input);
    assert_eq!(res, Ok((&input[4..],NewType(1))));
}

#[test]
fn test_tuple_struct_2() {
    let input = b"\x00\x00\x00\x01\xff\xff";
    let res = NewType2::parse(input);
    assert_eq!(res, Ok((&input[6..],NewType2(1,0xffff))));
}
