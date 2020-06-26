use nom_derive::Nom;

/// test for the `Debug` attribute (field)
#[derive(Debug, PartialEq, Nom)]
struct S1 {
    pub a: u8,
    #[nom(Debug)]
    pub b: u64,
}

/// test for the `Debug` attribute (top-level)
#[derive(Debug, PartialEq, Nom)]
#[nom(Debug)]
struct S2 {
    pub a: u8,
    pub b: u64,
}

// if test is used with '--nocapture', output will go to stderr
#[test]
fn test_struct_dbg() {
    let input = b"\x12\x34";
    let res = S1::parse(input).unwrap_err();
    assert!(res.is_incomplete());
}
