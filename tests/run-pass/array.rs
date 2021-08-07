use nom_derive::*;

/// A structure with a fixed-length array
#[derive(Nom, Debug, PartialEq)]
struct StructWithArray {
    pub a: [u8; 4],
    pub b: u8,
    c: u8,
}

#[derive(Nom, Debug, PartialEq)]
struct SubStruct {
    pub a: u8,
    pub b: u8,
}

/// A structure with an array of structs
#[derive(Nom, Debug, PartialEq)]
struct StructWithArrayOfStructs {
    pub a: [SubStruct; 2],
    pub b: u8,
    c: u8,
}

fn main() {
    let input = b"\x00\xff\x00\xff\x01\x02";

    let res = StructWithArray::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[6..],
            StructWithArray {
                a: [0, 255, 0, 255],
                b: 1,
                c: 2
            }
        ))
    );

    let res = StructWithArrayOfStructs::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[6..],
            StructWithArrayOfStructs {
                a: [ SubStruct {a:0, b:255},
                     SubStruct {a:0, b:255}],
                b: 1,
                c: 2
            }
        ))
    );
}
