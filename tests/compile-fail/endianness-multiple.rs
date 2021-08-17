extern crate nom;
extern crate nom_derive;
use nom_derive::*;

#[derive(Nom)]
#[nom(BigEndian, LittleEndian)] // ERROR: Struct cannot be both big and little endian
pub struct BothEndian1 {
    #[nom(Parse="u32")]
    a: u32,
}

#[derive(NomLE)] // ERROR: Object cannot be both big and little endian
#[nom(BigEndian)]
pub struct BothEndian2 {
    a: u32,
}

fn main() {}
