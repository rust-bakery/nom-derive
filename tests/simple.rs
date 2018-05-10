#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

#[macro_use]
extern crate nom;

use nom::*;

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

/// A structure containing a substructure
#[derive(Debug,PartialEq,Nom)]
struct StructWithSubStruct {
    pub a: u32,
    b: u64,
    #[Parse="call!(StructWithParser::parse)"]
    s: StructWithParser,
}

/// A simple structure with a verification
#[derive(Debug,PartialEq,Nom)]
struct StructWithVerify {
    #[Verify="a == 1"]
    pub a: u32,
}

#[test]
fn test_simple_struct() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = SimpleStruct::parse(input);
    assert_eq!(res, IResult::Done(&input[12..],SimpleStruct{a:1, b:0x1234567812345678}));
}

#[test]
fn test_struct_parse() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithParser::parse(input);
    assert_eq!(res, IResult::Done(&input[4..],StructWithParser{a:0x01000000}));
}

#[test]
fn test_struct_parse_substruct() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithSubStruct::parse(input);
    assert_eq!(res, IResult::Done(&input[16..],StructWithSubStruct{a:1,b:0x1234567812345678,s:StructWithParser{a:0x01000000}}));
}

#[test]
fn test_struct_with_verify() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithVerify::parse(input);
    assert_eq!(res, IResult::Done(&input[4..],StructWithVerify{a:1}));

    let res = StructWithVerify::parse(&input[4..]);
    assert_eq!(res, IResult::Error(ErrorKind::Verify));
}
