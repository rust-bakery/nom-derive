extern crate nom;
extern crate nom_derive;
use nom_derive::Nom;

pub struct Toto;

#[derive(Nom)]
pub struct S {
    #[nom(Count = "4")]
    a: u32,
}

fn main() {}
