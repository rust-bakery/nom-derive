use nom::IResult;
use nom::Parser;
use nom::character::streaming::alpha1;
use nom_derive::*;

fn parser1(i: &[u8]) -> IResult<&[u8], &[u8]> {
    alpha1(i)
}

#[derive(Debug, PartialEq, NomBE)]
// #[nom(DebugDerive)]
struct IntoStruct {
    #[nom(Into, Parse = "parser1")]
    a: Vec<u8>,
    b: u16,
    c: u16,
}

fn main() {
    let input = b"abcd\x00\x01\x00\x02";

    let res = IntoStruct::parse(input);
    assert_eq!(
        res,
        Ok((
            &input[8..],
            IntoStruct {
                a: vec![97, 98, 99, 100],
                b: 1,
                c: 2,
            }
        ))
    )
}

