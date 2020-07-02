#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

use nom_derive::Nom;

use nom::bytes::complete::take_till;
use nom::combinator::cond;
use nom::number::streaming::{be_u64, be_u8};
use nom::number::Endianness;
use std::ffi::CString;

/// A simple structure, with a complex sub-parser expression
#[derive(Debug, PartialEq, Nom)]
struct StructWithComplexParser {
    pub a: u32,
    #[nom(Parse = "cond(a > 0,be_u64)")]
    pub b: Option<u64>,
}

/// A simple structure, ignoring one field
#[derive(Debug, PartialEq, Nom)]
struct StructWithIgnore {
    pub a: u32,
    #[nom(Ignore)]
    pub b: Option<u64>,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithoutComplete {
    pub a: u32,
    pub b: u64,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithComplete {
    pub a: u32,
    #[nom(Complete)]
    pub b: u64,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithMap {
    pub a: u32,
    #[nom(Parse = "be_u8", Map = "|x: u8| x.to_string()")]
    int_str: String,
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
// #[nom(DebugDerive)]
#[nom(PostExec(println!("parsing done: {:?}", struct_def);))]
struct TopLevelPostExec {
    pub a: u32,
    pub b: u8,
    pub c: u8,
}

#[derive(Debug, PartialEq, Nom)]
#[repr(u8)]
// #[nom(DebugDerive)]
#[nom(PostExec(println!("parsing done: {:?}", enum_def);))]
pub enum FieldLessEnumPostExec {
    A,
    B = 2,
    C,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Nom)]
pub struct MessageType(pub u8);

#[derive(Debug, PartialEq, Nom)]
// #[nom(DebugDerive)]
#[nom(Selector = "MessageType")]
#[nom(PostExec(println!("parsing done: {:?}", enum_def);))]
pub enum EnumPostExec {
    #[nom(Selector = "MessageType(0)")]
    Field1(u32),
    #[nom(Selector = "MessageType(1)")]
    Field2(Option<u32>),
}

#[derive(Debug, PartialEq, Nom)]
// #[nom(DebugDerive)]
#[nom(InputName(iii))]
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

// order matters!
#[derive(Debug, PartialEq, Nom)]
struct StructWithAlignmentAndPadding {
    #[nom(AlignAfter(4), SkipAfter(2))]
    pub a: u8,
    b: u32,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithLongSkip {
    #[nom(SkipAfter(200))]
    pub a: u8,
    b: u32,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithOffset {
    pub a: u8,
    #[nom(Move(3))]
    b: u32,
    #[nom(Move(-4))]
    b2: u32,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithOffsetAbs {
    pub a: u8,
    #[nom(MoveAbs(4))]
    pub a2: u8,
}

#[derive(Debug, PartialEq, Nom)]
struct StructWithPossibleError {
    pub a: u8,
    #[nom(ErrorIf(a != 0))]
    b: u32,
}

#[derive(Debug, PartialEq, Nom)]
#[nom(Exact)]
struct StructExact {
    pub a: u32,
    pub b: u32,
}

fn bytes_to_cstring(s: &[u8]) -> CString {
    CString::new(s).unwrap()
}

#[derive(Debug, Nom)]
pub struct MultipleAttributes1 {
    pub a: u32,
    #[nom(
        Cond = "a == 0",
        Parse = "take_till(|b| b == 0)",
        Map = "bytes_to_cstring"
    )]
    cstring: Option<CString>,
}

#[derive(Debug, Nom, PartialEq)]
#[nom(BigEndian)]
pub struct MixedEndian {
    #[nom(SetEndian(Endianness::Big))]
    pub a: u32,
    #[nom(SetEndian(Endianness::Little))]
    pub b: u32,
}

fn test_value(x: u8) -> bool {
    x >> 3 & 1 == 1
}

#[derive(Debug, Nom, PartialEq)]
#[nom(Selector(u8))]
pub enum EnumWithGuard {
    #[nom(Selector(x if test_value(x)))]
    Foo(u8),
    #[nom(Selector(_))]
    Bar(u8),
}

const INPUT_16: &[u8] = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
const INPUT_CSTRING: &[u8] = b"\x00\x00\x00\x00Hello, world!\x00\x01";

#[test]
fn test_struct_complex_parse() {
    let res = StructWithComplexParser::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[12..],
            StructWithComplexParser {
                a: 1,
                b: Some(0x1234567812345678)
            }
        ))
    );
}

#[test]
fn test_struct_ignore() {
    let res = StructWithIgnore::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((&INPUT_16[4..], StructWithIgnore { a: 1, b: None }))
    );
}

#[test]
fn test_struct_complete() {
    let input = &INPUT_16[12..];
    let res = StructWithoutComplete::parse(input).expect_err("parse error");
    let res_complete = StructWithComplete::parse(input).expect_err("parse error");
    // res: Error(Incomplete(Size(8)))
    assert!(res.is_incomplete());
    // res_complete: Error(Error(([], Complete)))
    assert!(!res_complete.is_incomplete());
}

#[test]
fn test_struct_map() {
    let res = StructWithMap::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[5..],
            StructWithMap {
                a: 1,
                int_str: "18".to_string()
            }
        ))
    );
}

#[test]
fn test_struct_postexec() {
    let res = StructWithPostExec::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((&INPUT_16[5..], StructWithPostExec { a: 1, b: 18, c: 19 }))
    );
    let res = TopLevelPostExec::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[6..],
            TopLevelPostExec {
                a: 1,
                b: 0x12,
                c: 0x34
            }
        ))
    );
}

#[test]
fn test_struct_align_and_padding() {
    let res = StructWithAlignment::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[12..],
            StructWithAlignment {
                a: 0,
                b: 0x1234_5678_1234_5678
            }
        ))
    );
    let res = StructWithAlignmentAndPadding::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[10..],
            StructWithAlignmentAndPadding {
                a: 0,
                b: 0x5678_1234
            }
        ))
    );
    let res = StructWithLongSkip::parse(INPUT_16).expect_err("parse error");
    if let nom::Err::Incomplete(sz) = res {
        assert_eq!(sz, nom::Needed::Size(200));
    } else {
        panic!("wrong error type");
    }
    let res = StructWithOffset::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[8..],
            StructWithOffset {
                a: 0,
                b: 0x1234_5678,
                b2: 0x1234_5678
            }
        ))
    );
    let res = StructWithOffsetAbs::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((&INPUT_16[5..], StructWithOffsetAbs { a: 0, a2: 0x12 }))
    );
}

#[test]
fn test_struct_error_if() {
    // test without verification error
    let res = StructWithPossibleError::parse(INPUT_16);
    assert!(res.is_ok());
    // test with verification error
    let res = StructWithPossibleError::parse(&INPUT_16[4..]).expect_err("parsing failed");
    if let nom::Err::Error((_, error_kind)) = res {
        assert_eq!(error_kind, nom::error::ErrorKind::Verify);
    } else {
        panic!("wrong error type");
    }
}

#[test]
fn test_struct_exact() {
    // test without verification error
    let res = StructExact::parse(&INPUT_16[8..]);
    assert!(res.is_ok());
    // test with verification error
    let res = StructExact::parse(INPUT_16).expect_err("parsing failed");
    if let nom::Err::Error((_, error_kind)) = res {
        assert_eq!(error_kind, nom::error::ErrorKind::Verify);
    } else {
        panic!("wrong error type");
    }
}

#[test]
fn test_struct_multiple_attributes() {
    let (rem, res) = MultipleAttributes1::parse(INPUT_CSTRING).expect("parsing failed");
    assert!(!rem.is_empty());
    assert!(res.cstring.is_some());
    assert_eq!(res.cstring.unwrap().as_bytes(), b"Hello, world!");
}

#[test]
fn test_struct_mixed_endian() {
    let (rem, res) = MixedEndian::parse(INPUT_16).expect("parsing failed");
    assert!(!rem.is_empty());
    assert_eq!(
        res,
        MixedEndian {
            a: 1,
            b: 0x7856_3412,
        }
    );
}

#[test]
fn test_enum_selector_with_guard() {
    let input = &[0];
    let (_, res) = EnumWithGuard::parse(input, 0).expect("parsing failed");
    assert_eq!(res, EnumWithGuard::Bar(0));
    let (_, res) = EnumWithGuard::parse(input, 0b1000).expect("parsing failed");
    assert_eq!(res, EnumWithGuard::Foo(0));
}
