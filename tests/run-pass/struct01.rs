use nom_derive::*;

/// A simple structure, deriving a trivial parser
#[derive(Debug, PartialEq, Nom)]
struct SimpleStruct {
    pub a: u32,
    b: u64,
}

fn main() {
    const INPUT_16: &[u8] = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";

    let input = INPUT_16;
    let res = SimpleStruct::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[12..],
            SimpleStruct {
                a: 1,
                b: 0x1234567812345678
            }
        ))
    );
}
