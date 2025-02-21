use nom::Parser;
use nom_derive::nom::{Err, Needed, error::*};
use nom_derive::*;

/// By default, derived parsers use "streaming"
#[derive(Debug, PartialEq, NomBE)]
struct SimpleStruct1 {
    pub a: u32,
    b: u64,
}

/// Derive a "complete" parser (top-level attribute)
#[derive(Debug, PartialEq, NomBE)]
#[nom(Complete)]
struct SimpleStruct2 {
    pub a: u32,
    b: u64,
}

/// Derive a partially "complete" parser (field-level attribute)
#[derive(Debug, PartialEq, NomBE)]
struct SimpleStruct3 {
    pub a: u32,
    #[nom(Complete)]
    b: u64,
}

fn main() {
    const INPUT_16: &[u8] = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";

    let input = INPUT_16;
    let res = SimpleStruct1::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[12..],
            SimpleStruct1 {
                a: 1,
                b: 0x1234567812345678
            }
        ))
    );

    let res = SimpleStruct1::parse(&input[10..]);
    assert_eq!(res, Err(Err::Incomplete(Needed::new(6))));

    let res = SimpleStruct2::parse(&input[10..]);
    assert_eq!(
        res,
        Err(Err::Error(Error {
            input: &input[14..],
            code: ErrorKind::Complete
        }))
    );

    let res = SimpleStruct3::parse(&input[10..]);
    assert_eq!(
        res,
        Err(Err::Error(Error {
            input: &input[14..],
            code: ErrorKind::Complete
        }))
    );
}
