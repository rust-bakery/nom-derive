use nom::number::Endianness;
use nom_derive::*;

#[derive(Debug, PartialEq, Nom)]
#[nom(ExtraArgs(endian: Endianness))]
#[nom(SetEndian(endian))] // Set dynamically the endianness
struct MixedEndianStruct {
    a: u32,
    b: u16,
    #[nom(BigEndian)] // Field c will always be parsed as BigEndian
    c: u16,
}

/// unnamed struct
#[derive(Debug, PartialEq, Eq, Clone, Copy, Nom)]
#[nom(GenericErrors)]
pub struct MessageType(pub u8);

/// An enum with unnamed fields
#[derive(Debug, PartialEq, Nom)]
#[nom(ExtraArgs(endian: Endianness))]
#[nom(SetEndian(endian))] // Set dynamically the endianness
#[nom(Selector = "MessageType")]
// #[nom(DebugDerive)]
pub enum U1 {
    #[nom(Selector = "MessageType(0)")]
    Field1(u32),
    #[nom(Selector = "MessageType(1)")]
    Field2(Option<u32>),
}

fn main() {
    let input: &[u8] = b"\x00\x01\x02\x03\x04\x05\x06\x07";

    let res = MixedEndianStruct::parse(input, Endianness::Big);
    assert_eq!(
        res,
        Ok((
            &input[8..],
            MixedEndianStruct {
                a: 0x10203,
                b: 0x0405,
                c: 0x0607,
            }
        ))
    );
    let res = MixedEndianStruct::parse(input, Endianness::Little);
    assert_eq!(
        res,
        Ok((
            &input[8..],
            MixedEndianStruct {
                a: 0x03020100,
                b: 0x0504,
                c: 0x0607,
            }
        ))
    );

    let rem = U1::parse(&input[1..], MessageType(input[0]), Endianness::Big).unwrap();
    assert_eq!(rem, (&input[5..], U1::Field1(0x01020304)));
}
