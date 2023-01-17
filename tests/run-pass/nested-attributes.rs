use nom_derive::*;

#[derive(Debug, PartialEq, Nom)]
// #[nom(DebugDerive)]
struct OptionVecS1 {
    a: u16,
    b: Option<Vec<u16>>,
}

#[derive(Debug, PartialEq, Nom)]
// #[nom(DebugDerive)]
struct OptionVecS2 {
    a: u16,
    #[nom(Cond = "a > 1")]
    b: Option<Vec<u16>>,
}

#[derive(Debug, PartialEq, Nom)]
// #[nom(DebugDerive)]
struct OptionVecS3 {
    a: u16,
    #[nom(Cond = "a > 1")]
    #[nom(Count = "4")]
    b: Option<Vec<u16>>,
}

fn main() {
    const INPUT_16: &[u8] = b"\x00\x04\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";

    let input = INPUT_16;
    let res = OptionVecS1::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[16..],
            OptionVecS1 {
                a: 4,
                b: Some(vec![1, 0x1234, 0x5678, 0x1234, 0x5678, 0, 1])
            }
        ))
    );

    let res = OptionVecS1::parse_le(input);
    assert_eq!(
        res,
        Ok((
            &input[16..],
            OptionVecS1 {
                a: 0x400,
                b: Some(vec![0x100, 0x3412, 0x7856, 0x3412, 0x7856, 0, 0x100])
            }
        ))
    );

    let res = OptionVecS2::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[16..],
            OptionVecS2 {
                a: 4,
                b: Some(vec![1, 0x1234, 0x5678, 0x1234, 0x5678, 0, 1])
            }
        ))
    );

    let res = OptionVecS2::parse_le(input);
    assert_eq!(
        res,
        Ok((
            &input[16..],
            OptionVecS2 {
                a: 0x400,
                b: Some(vec![0x100, 0x3412, 0x7856, 0x3412, 0x7856, 0, 0x100])
            }
        ))
    );

    let res = OptionVecS3::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[10..],
            OptionVecS3 {
                a: 4,
                b: Some(vec![1, 0x1234, 0x5678, 0x1234])
            }
        ))
    );

    let res = OptionVecS3::parse_le(input);
    assert_eq!(
        res,
        Ok((
            &input[10..],
            OptionVecS3 {
                a: 0x400,
                b: Some(vec![0x100, 0x3412, 0x7856, 0x3412])
            }
        ))
    );
}
