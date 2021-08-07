extern crate nom;
extern crate nom_derive;
use nom_derive::Nom;

#[derive(Nom)]
pub struct S {
    a: dyn Fn(u16) -> u16,
}

fn main() {}
