#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

use nom::number::streaming::*;
use nom::IResult;

#[derive(Debug, PartialEq, Nom)]
#[nom(BigEndian)]
struct BigEndianStruct {
    pub a: u32,
    b: u64,
}

#[derive(Debug, PartialEq, Nom)]
#[nom(LittleEndian)]
struct LittleEndianStruct {
    pub a: u32,
    b: u64,
}

#[test]
fn big_endian_struct() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = BigEndianStruct::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[12..],
            BigEndianStruct {
                a: 1,
                b: 0x1234_5678_1234_5678
            }
        ))
    );
}

#[test]
fn little_endian_struct() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = LittleEndianStruct::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[12..],
            LittleEndianStruct {
                a: 0x0100_0000,
                b: 0x7856_3412_7856_3412
            }
        ))
    );
}

// XXX panics at compile time, not runtime
// #[test]
// #[should_panic]
// fn both_little_and_big_endian() {
//     #[derive(Debug, PartialEq, NomDeriveDebug)]
//     #[nom(Little, Big)]
//     struct BothEndianStruct {
//         pub a: u32,
//         b: u64,
//     }
// }
