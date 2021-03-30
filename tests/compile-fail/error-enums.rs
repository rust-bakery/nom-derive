extern crate nom;
extern crate nom_derive;
use nom_derive::Nom;

#[derive(Nom)]
pub enum E1 {
    // ERROR: Nom-derive: fieldless enums must have a 'repr' attribute
    A = 0,
}

#[derive(Nom)]
pub enum E2 {
    // ERROR: Nom-derive: enums must specify the 'selector' attribute
    A(u32),
}

#[derive(Nom)]
#[nom(Selector = "u8")]
pub enum E3 {
    // ERROR: Nom-derive: the 'Selector' attribute must be used to give the value of selector item
    A(u32),
}

pub type U24 = u32;

#[derive(Nom)]
#[repr(U24)] // ERROR: Nom-derive: cannot parse 'repr' content (must be a primitive type)
pub enum E4 {
    A = 0,
}

fn main() {}
