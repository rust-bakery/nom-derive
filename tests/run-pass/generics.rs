use nom_derive::nom::IResult;
use nom_derive::*;

/// A struct with a generic parameter
#[derive(Nom, Debug, PartialEq)]
// #[nom(DebugDerive)]
pub struct StructWithGenerics<T> {
    pub t: T,
}

/// A struct with a generic parameter, and generic errors
#[derive(Nom, Debug, PartialEq)]
// #[nom(DebugDerive)]
#[nom(GenericErrors)]
pub struct StructWithGenericsAndErrors<T> {
    pub t: T,
}

fn main() {
    let input: &[u8] = b"\x00\x01\x02\x03\x04\x05\x06\x07";

    // test generics: u16
    let rem = StructWithGenerics::<u16>::parse(input).unwrap();
    assert_eq!(rem, (&input[2..], StructWithGenerics { t: 0x1u16 }));

    // test generics: u32
    let rem = StructWithGenerics::<u32>::parse(input).unwrap();
    assert_eq!(rem, (&input[4..], StructWithGenerics { t: 0x10203u32 }));

    // test generics: u16 and custom error: unit
    let rem: IResult<_, _, ()> = StructWithGenericsAndErrors::<u16>::parse(input);
    assert_eq!(
        rem.unwrap(),
        (&input[2..], StructWithGenericsAndErrors { t: 0x1u16 })
    );

    // test generics: u16 and custom error: VerboseError
    let rem: IResult<_, _, nom::error::Error<_>> = StructWithGenericsAndErrors::<u16>::parse(input);
    assert_eq!(
        rem.unwrap(),
        (&input[2..], StructWithGenericsAndErrors { t: 0x1u16 })
    );
}
