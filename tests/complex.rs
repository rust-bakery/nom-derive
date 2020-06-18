#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

use nom_derive::Nom;

use nom::combinator::cond;
use nom::number::streaming::{be_u8, be_u64};

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

#[derive(Debug,PartialEq,Nom)]
struct StructWithoutComplete {
    pub a: u32,
    pub b: u64,
}

#[derive(Debug,PartialEq,Nom)]
struct StructWithComplete {
    pub a: u32,
    #[nom(Complete)]
    pub b: u64,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithMap {
    pub a: u32,
    #[nom(Parse="be_u8", Map = "|x: u8| x.to_string()")]
    int_str: String
}

#[derive(Debug, PartialEq, Nom)]
// #[nom(DebugDerive)]
struct StructWithPostExec {
    pub a: u32,
    #[nom(PostExec(let c = b + 1;))]
    pub b: u8,
    #[nom(Value(c))]
    pub c: u8,
}

#[derive(Debug, PartialEq, Nom)]
#[nom(DebugDerive, InputName(iii))]
struct StructWithInputName {
    pub a: u32,
    #[nom(Value(iii.len()))]
    pub sz: usize,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithAlignment {
    #[nom(AlignAfter(4))]
    pub a: u8,
    b: u64,
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

#[test]
fn test_struct_complete() {
    let input = &INPUT_16[12..];
    let res= StructWithoutComplete::parse(input).expect_err("parse error");
    let res_complete = StructWithComplete::parse(input).expect_err("parse error");
    // res: Error(Incomplete(Size(8)))
    assert!(res.is_incomplete());
    // res_complete: Error(Error(([], Complete)))
    assert!(!res_complete.is_incomplete());
}

#[test]
fn test_struct_map() {
    let res = StructWithMap::parse(INPUT_16);
    assert_eq!(res, Ok((&INPUT_16[5..],StructWithMap{a:1,int_str:"18".to_string()})));
}

#[test]
fn test_struct_postexec() {
    let res = StructWithPostExec::parse(INPUT_16);
    assert_eq!(res, Ok((&INPUT_16[5..],StructWithPostExec{a:1,b:18,c:19})));
}

#[test]
fn test_struct_align() {
    let res = StructWithAlignment::parse(INPUT_16);
    assert_eq!(res, Ok((&INPUT_16[12..],StructWithAlignment{a:0,b:0x1234_5678_1234_5678})));
}
