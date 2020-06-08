#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

use nom::*;
use nom::bytes::complete::take;
use nom::combinator::map;
use nom::number::streaming::*;
use std::marker::PhantomData;

/// A simple structure, with a lifetime
#[derive(Debug,PartialEq,Nom)]
struct StructWithLifetime<'a> {
    /// This field provides the parsing code, and calls a macro
    #[Parse="take(4 as usize)"]
    s: &'a[u8],
}

/// A structure with different lifetimes
#[derive(Debug,PartialEq,Nom)]
struct StructWithLifetimes<'a,'b> {
    #[Parse="take(4 as usize)"]
    s: &'a[u8],
    #[Parse="take(4 as usize)"]
    t: &'b[u8],
}

// /// A structure with PhantomData
#[derive(Debug,PartialEq,Nom)]
struct StructWithPhantomData<'a> {
    #[Parse="map(be_u64, |x| x as *const u8)"]
    start: *const u8,
    phantom: PhantomData<&'a u8>,
}

#[test]
fn test_struct_with_lifetime() {
    let input = b"\x00\x00\x00\x01";
    let res = StructWithLifetime::parse(input);
    assert_eq!(res, Ok((&input[4..],StructWithLifetime{s:&input[0..4]})));
}

#[test]
fn test_struct_with_lifetimes() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithLifetimes::parse(input);
    assert_eq!(res, Ok((&input[8..],StructWithLifetimes{s:&input[0..4], t:&input[4..8]})));
}

#[test]
fn test_struct_with_phantomdata() {
    let input = b"\x12\x34\x56\x78\x12\x34\x56\x78";
    let res = StructWithPhantomData::parse(input);
    assert_eq!(res,
               Ok((&input[8..],
               StructWithPhantomData{start:0x1234567812345678 as *const u8, phantom:PhantomData}))
              );
}

// XXX generics are not supported

// fn parse_generics<G>(i:&[u8]) -> IResult<&[u8],Option<G>> {
//     Ok((i,None))
// }
//
// use std::fmt::Debug;
//
// /// A structure with lifetimes and generics
// #[derive(Debug,PartialEq,Nom)]
// struct StructWithGenerics<'a,'b,G>
//         where G: Debug + PartialEq {
//     #[Parse="take!(4)"]
//     s: &'a[u8],
//     #[Parse="take!(4)"]
//     t: &'b[u8],
//     #[Parse="call!(parse_generics)"]
//     g: Option<G>,
// }
