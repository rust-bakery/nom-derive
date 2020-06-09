#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

use nom::*;
use nom::combinator::cond;
use nom::number::streaming::*;

/// A simple structure, with a complex sub-parser expression
#[derive(Debug,PartialEq,Nom)]
struct StructWithComplexParser {
    pub a: u32,
    #[nom(Parse="cond(a > 0,be_u64)")]
    pub b: Option<u64>,
}


#[test]
fn test_struct_complex_parse() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithComplexParser::parse(input);
    assert_eq!(res, Ok((&input[12..],StructWithComplexParser{a:1,b:Some(0x1234567812345678)})));
}
