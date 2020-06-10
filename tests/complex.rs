#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

use nom_derive::Nom;

use nom::combinator::cond;
use nom::number::streaming::be_u64;

/// A simple structure, with a complex sub-parser expression
#[derive(Debug,PartialEq,Nom)]
struct StructWithComplexParser {
    pub a: u32,
    #[nom(Parse="cond(a > 0,be_u64)")]
    pub b: Option<u64>,
}

/// A simple structure, ignoring one field
#[derive(Debug,PartialEq,Nom)]
struct StructWithIgnore {
    pub a: u32,
    #[nom(Ignore)]
    pub b: Option<u64>,
}

const INPUT_16: &[u8] = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";

#[test]
fn test_struct_complex_parse() {
    let res = StructWithComplexParser::parse(INPUT_16);
    assert_eq!(res, Ok((&INPUT_16[12..],StructWithComplexParser{a:1,b:Some(0x1234567812345678)})));
}

#[test]
fn test_struct_ignore() {
    let res = StructWithIgnore::parse(INPUT_16);
    assert_eq!(res, Ok((&INPUT_16[4..],StructWithIgnore{a:1,b:None})));
}
