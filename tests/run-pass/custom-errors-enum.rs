use nom_derive::nom::IResult;
use nom_derive::nom::bytes::streaming::take;
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

// unnamed struct
#[derive(Debug, PartialEq, Eq, Clone, Copy, Nom)]
#[nom(GenericErrors)]
pub struct MessageType(pub u8);

/// An enum with unnamed fields
#[derive(Debug, PartialEq, Nom)]
#[nom(GenericErrors)]
#[nom(Selector = "MessageType")]
pub enum U1 {
    #[nom(Selector = "MessageType(0)")]
    Field1(u32),
    #[nom(Selector = "MessageType(1)")]
    Field2(Option<u32>),
}

/// An enum with lifetime
#[derive(Debug, PartialEq, Nom)]
#[nom(GenericErrors)]
#[nom(Selector = "MessageType")]
pub enum U3<'a> {
    #[nom(Selector = "MessageType(0)")]
    Field1(u32),
    // next variant has to be annotated for parsing (inside variant definition, not outside!)
    #[nom(Selector = "MessageType(1)")]
    Field2(#[nom(Parse = "take(4 as usize)")] &'a [u8]),
}

/// An enum with two lifetimes
#[derive(Debug, PartialEq, Nom)]
#[nom(GenericErrors)]
#[nom(Selector = "MessageType")]
pub enum U4<'a, 'b> {
    #[nom(Selector = "MessageType(0)")]
    Field1(#[nom(Parse = "take(4 as usize)")] &'a [u8]),
    // next variant has to be annotated for parsing (inside variant definition, not outside!)
    #[nom(Selector = "MessageType(1)")]
    Field2(#[nom(Parse = "take(4 as usize)")] &'b [u8]),
}

/// A fieldless enum with values
#[derive(Debug, PartialEq, Nom)]
#[nom(GenericErrors)]
#[repr(u8)]
pub enum U6 {
    A,
    B = 2,
    C,
}

fn main() {
    let input: &[u8] = b"\x00\x01\x02\x03\x04\x05\x06\x07";

    // test error type: unit
    let rem = U1::parse::<()>(&input[1..], MessageType(input[0])).unwrap();
    assert_eq!(rem, (&input[5..], U1::Field1(0x01020304)));

    // test error type: VerboseError
    let rem = U1::parse::<CustomError<_>>(&input[1..], MessageType(input[0])).unwrap();
    assert_eq!(rem, (&input[5..], U1::Field1(0x01020304)));

    // test lifetime and error type: unit
    let rem = U3::parse::<()>(&input[1..], MessageType(input[0])).unwrap();
    assert_eq!(rem, (&input[5..], U3::Field1(0x01020304)));

    // test two lifetimes and error type: unit
    let rem = U4::parse::<()>(&input[1..], MessageType(input[0])).unwrap();
    assert_eq!(rem, (&input[5..], U4::Field1(b"\x01\x02\x03\x04")));

    // test fieldless enum and error type: unit
    let rem: IResult<_, _, CustomError<_>> = U6::parse(&input[2..]);
    assert_eq!(rem.unwrap(), (&input[3..], U6::B));
}
