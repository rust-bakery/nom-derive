use nom_derive::nom::IResult;
use nom_derive::*;

// strings and generic errors (#32)
#[derive(Debug, PartialEq, NomBE)]
#[nom(GenericErrors)]
struct SimpleStruct1 {
    b: String,
}

fn main() {
    let input = b"\x00\x00\x00\x04abcd";

    let res: IResult<_, _, ()> = SimpleStruct1::parse(input);
    assert!(res.is_ok());
    assert_eq!(
        res,
        Ok((
            &b""[..],
            SimpleStruct1 {
                b: "abcd".to_string()
            }
        ))
    );
}
