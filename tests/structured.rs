#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

use nom::*;
use nom::combinator::{complete, opt};
use nom::multi::many0;
use nom::number::streaming::*;

/// A simple structure with an Option type
#[derive(Debug,PartialEq,Nom)]
struct StructWithOption {
    pub a: u32,
    b: Option<u64>,
}

/// A simple structure with an Option type
#[derive(Debug,PartialEq,Nom)]
struct StructWithOptionOption {
    pub a: u32,
    b: Option<Option<u64>>,
}

/// A simple structure with a Vec type
#[derive(Debug,PartialEq,Nom)]
struct StructWithVec {
    pub a: u32,
    b: Vec<u32>,
}

#[test]
fn test_struct_with_option() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithOption::parse(input);
    assert_eq!(res, Ok((&input[12..],StructWithOption{a:1, b:Some(0x1234567812345678)})));

    let input2 = &input[0..4];
    let res = StructWithOption::parse(input2);
    assert_eq!(res, Ok((&input2[4..],StructWithOption{a:1, b:None})));
}

#[test]
fn test_struct_with_option_option() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithOptionOption::parse(input);
    assert_eq!(res, Ok((&input[12..],StructWithOptionOption{a:1, b:Some(Some(0x1234567812345678))})));
}

#[test]
fn test_struct_with_vec() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithVec::parse(input);
    assert_eq!(res, Ok((&input[16..],StructWithVec{a:1, b:
        vec![0x12345678,0x12345678,0x1]
        })));
}
