#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

use nom_derive::*;

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

#[derive(Debug, PartialEq, Nom)]
#[nom(BigEndian)]
struct MixedEndianStruct {
    pub a: u32,
    #[nom(LittleEndian)]
    b: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Nom)]
#[nom(LittleEndian)]
pub struct UnitStruct(pub u32);

/// A fieldless enum with values
#[derive(Debug, PartialEq, Nom)]
#[nom(LittleEndian)]
#[repr(u32)]
pub enum Le16 {
    A = 1,
    B,
    C = 0x0100_0000,
}

const INPUT_16: &[u8] = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";

#[test]
fn big_endian_struct() {
    let res = BigEndianStruct::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[12..],
            BigEndianStruct {
                a: 1,
                b: 0x1234_5678_1234_5678
            }
        ))
    );
}

#[test]
fn little_endian_struct() {
    let res = LittleEndianStruct::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[12..],
            LittleEndianStruct {
                a: 0x0100_0000,
                b: 0x7856_3412_7856_3412
            }
        ))
    );
}

#[test]
fn mixed_endian_struct() {
    let res = MixedEndianStruct::parse(INPUT_16);
    assert_eq!(
        res,
        Ok((
            &INPUT_16[12..],
            MixedEndianStruct {
                a: 0x1,
                b: 0x7856_3412_7856_3412
            }
        ))
    );
}

#[test]
fn little_endian_unit_struct() {
    let res = UnitStruct::parse(INPUT_16);
    assert_eq!(res, Ok((&INPUT_16[4..], UnitStruct(0x0100_0000))));
}

#[test]
fn little_endian_enum() {
    let res = Le16::parse(INPUT_16);
    assert_eq!(res, Ok((&INPUT_16[4..], Le16::C)));
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
