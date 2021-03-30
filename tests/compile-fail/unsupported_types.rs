extern crate nom;
extern crate nom_derive;
use nom_derive::Nom;
use std::collections::HashMap;

#[derive(Nom)]
pub struct S1 {
    h: HashMap<u64, u64>,
}

#[derive(Nom)]
pub struct S2 {
    h: ::std::primitive::u64, // ERROR: Nom-derive: multiple segments in type path are not supported
}

fn main() {}
