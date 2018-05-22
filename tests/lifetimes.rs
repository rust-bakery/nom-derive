#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate nom_derive;

#[macro_use]
extern crate nom;

use nom::*;

/// A structure with different lifetimes
#[derive(Debug,PartialEq,Nom)]
struct StructWithLifetimes<'a,'b> {
    #[Parse="take!(4)"]
    s: &'a[u8],
    #[Parse="take!(4)"]
    t: &'b[u8],
}



#[test]
fn test_struct_with_lifetimes() {
    let input = b"\x00\x00\x00\x01\x12\x34\x56\x78\x12\x34\x56\x78\x00\x00\x00\x01";
    let res = StructWithLifetimes::parse(input);
    assert_eq!(res, Ok((&input[8..],StructWithLifetimes{s:&input[0..4], t:&input[4..8]})));
}

// XXX generics are not supported

// fn parse_generics<G>(i:&[u8]) -> IResult<&[u8],Option<G>> {
//     IResult::Done(i,None)
// }
// 
// /// A structure with lifetimes and generics
// #[derive(Debug,PartialEq,Nom)]
// struct StructWithGenerics<'a,'b,G>
//         where G: Debug + PartialEq {
//     #[Parse="take!(4)"]
//     s: &'a[u8],
//     #[Parse="take!(4)"]
//     t: &'b[u8],
//     #[Parse="parse_generics"]
//     g: Option<G>,
// }
