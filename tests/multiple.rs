#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

use nom_derive::Nom;

/// A structure with a length and a Vec
#[derive(Debug,PartialEq,Nom)]
struct S1 {
    pub a: u32,
    #[nom(Count="a")]
    pub b: Vec<u32>,
    pub c: u32,
}

#[derive(Debug,PartialEq,Nom)]
struct NewType(pub u8);

/// A structure with a length and a Vec of structs
#[derive(Debug,PartialEq,Nom)]
struct S2 {
    pub a: u8,
    #[nom(Count="a")]
    pub b: Vec<NewType>,
}

/// A structure Vec of structs, count with literal
#[derive(Debug,PartialEq,Nom)]
struct S3 {
    #[nom(Count="2")]
    pub b: Vec<NewType>,
}

#[derive(Debug, PartialEq, Nom)]
struct S4 {
    pub a: u8,
    #[nom(Value="a+1")]
    pub b: u8,
    #[nom(Value="b.to_string()")]
    pub s: String,
}


#[test]
fn test_struct_count() {
    let input = b"\x00\x00\x00\x02\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = S1::parse(input);
    assert_eq!(res, Ok((&input[16..],S1{a:2, b:vec![0x12345678,0x12345678], c:1})));
}

#[test]
fn test_struct_count_newtype() {
    let input = b"\x02\x12\x34";
    let res = S2::parse(input);
    assert_eq!(res, Ok((&input[3..],S2{a:2, b:vec![NewType(0x12),NewType(0x34)]})));
}

#[test]
fn test_struct_count_literal() {
    let input = b"\x12\x34";
    let res = S3::parse(input);
    assert_eq!(res, Ok((&input[2..],S3{b:vec![NewType(0x12),NewType(0x34)]})));
}

#[test]
fn test_struct_value() {
    let input = b"\x12\x34";
    let res = S4::parse(input);
    assert_eq!(res, Ok((&input[1..],
        S4{a:0x12, b:0x13, s:"19".to_string()}
        )));
}
