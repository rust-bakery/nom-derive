extern crate nom;
extern crate nom_derive;
use nom_derive::Nom;

#[derive(Nom)]
pub struct S {
    #[nom(Parse=u32)]
    a: u32,
}

#[derive(Nom)]
#[nom(Parse="be_u32")] // ERROR: Attribute Parse(be_u32) is not valid for top-level
pub struct S2 {
    a: u32,
}

#[derive(Nom)]
pub struct S3 {
    #[nom(Exact)] // ERROR: Attribute Exact is not valid for field-level
    a: u32,
}

fn main() {}