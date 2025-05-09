use nom_derive::nom::IResult;
use nom_derive::nom::error::{ErrorKind, ParseError};
use nom_derive::*;

#[derive(Debug)]
struct CustomError<I: std::fmt::Debug> {
    input: I,
    code: ErrorKind,
}

impl<I: std::fmt::Debug> ParseError<I> for CustomError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self { input, code: kind }
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

#[derive(Nom, Debug, PartialEq)]
#[nom(GenericErrors)]
pub struct S1 {
    pub a: u32,
}

#[derive(Nom, Debug, PartialEq)]
#[nom(GenericErrors)]
pub struct StructWithLifetime<'a> {
    pub a: u32,
    #[nom(Default)]
    pub b: Option<&'a [u8]>,
}

#[derive(Nom, Debug, PartialEq)]
#[nom(GenericErrors)]
pub struct StructWithTwoLifetimes<'a, 'b> {
    pub a: u32,
    #[nom(Cond = "a == 0", Take(4))]
    pub b: Option<&'a [u8]>,
    #[nom(Take(4))]
    pub c: &'b [u8],
}

fn main() {
    let input: &[u8] = b"\x00\x01\x02\x03\x04\x05\x06\x07";

    // test error type: unit
    let rem: IResult<_, _, ()> = S1::parse(input);
    assert_eq!(rem.unwrap(), (&input[4..], S1 { a: 0x10203 }));

    // test error type: VerboseError
    let rem: IResult<_, _, CustomError<_>> = S1::parse(input);
    assert_eq!(rem.unwrap(), (&input[4..], S1 { a: 0x10203 }));

    // test lifetimes and error type: VerboseError
    let rem: IResult<_, _, CustomError<_>> = StructWithLifetime::parse(input);
    assert_eq!(
        rem.unwrap(),
        (
            &input[4..],
            StructWithLifetime {
                a: 0x10203,
                b: None
            }
        )
    );

    // test two lifetimes and error type: VerboseError
    let rem: IResult<_, _, CustomError<_>> = StructWithTwoLifetimes::parse(input);
    assert_eq!(
        rem.unwrap(),
        (
            &input[8..],
            StructWithTwoLifetimes {
                a: 0x10203,
                b: None,
                c: &input[4..],
            }
        )
    );
}
