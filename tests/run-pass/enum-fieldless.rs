use nom::number::streaming::be_u8;
use nom_derive::Nom;

/// A fieldless enum, using the 'Selector' attribute
///
/// Note that this could be implemented or derived as a `From` trait
/// in a more efficient way.
#[derive(Debug, PartialEq, Nom)]
#[nom(Selector = "u8")]
enum U1 {
    #[nom(Selector = "0")]
    Foo,
    #[nom(Selector = "_")]
    Bar,
}

/// A fieldless enum with values
#[derive(Debug, PartialEq, Nom)]
#[repr(u8)]
pub enum U2 {
    A,
    B = 2,
    C,
}

fn main() {
    let empty: &[u8] = b"";
    let (rem, b) = be_u8::<&[u8], ()>(b"\x00").unwrap();
    assert_eq!(U1::parse(rem, b), Ok((empty, U1::Foo)));
    //
    assert_eq!(U2::parse(b"\x00"), Ok((empty, U2::A)));
    assert!(U2::parse(b"\x01").is_err());
    assert_eq!(U2::parse(b"\x02"), Ok((empty, U2::B)));
}
