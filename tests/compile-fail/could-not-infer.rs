extern crate nom;
extern crate nom_derive;
use nom_derive::Nom;

#[derive(Nom)]
pub struct S {
    a: [u32; 4],
}

fn main() {}