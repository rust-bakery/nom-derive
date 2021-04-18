use nom_derive::*;

/// A simple structure, deriving a trivial parser
#[derive(Debug, PartialEq, NomBE)]
struct SimpleStruct1 {
    pub a: u32,
    b: u64,
}

/// A simple structure, deriving a trivial parser
#[derive(Debug, PartialEq, NomLE)]
struct SimpleStruct2 {
    pub a: u32,
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

    let res = SimpleStruct2::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[12..],
            SimpleStruct2 {
                a: 0x01000000,
                b: 0x7856341278563412
            }
        ))
    );
}
