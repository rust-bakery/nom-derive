use nom_derive::nom::error::VerboseError;
use nom_derive::*;

#[derive(Nom, Debug, PartialEq)]
// #[nom(DebugDerive)]
#[nom(GenericErrors)]
pub struct S1 {
    pub a: u32,
}

#[derive(Nom, Debug, PartialEq)]
// #[nom(DebugDerive)]
#[nom(GenericErrors)]
pub struct StructWithLifetime<'a> {
    pub a: u32,
    #[nom(Default)]
    pub b: Option<&'a [u8]>,
}

#[derive(Nom, Debug, PartialEq)]
// #[nom(DebugDerive)]
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
    let rem = S1::parse::<()>(input).unwrap();
    assert_eq!(rem, (&input[4..], S1 { a: 0x10203 }));

    // test error type: VerboseError
    let rem = S1::parse::<VerboseError<_>>(input).unwrap();
    assert_eq!(rem, (&input[4..], S1 { a: 0x10203 }));

    // test lifetimes and error type: VerboseError
    let rem = StructWithLifetime::parse::<VerboseError<_>>(input).unwrap();
    assert_eq!(
        rem,
        (
            &input[4..],
            StructWithLifetime {
                a: 0x10203,
                b: None
            }
        )
    );

    // test two lifetimes and error type: VerboseError
    let rem = StructWithTwoLifetimes::parse::<VerboseError<_>>(input).unwrap();
    assert_eq!(
        rem,
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
