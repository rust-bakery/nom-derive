extern crate nom;
extern crate nom_derive;
use nom_derive::Nom;

#[derive(Nom)]
pub struct S {
    #[nom(Cond="true")]
    a: u32,
}

fn main() {}