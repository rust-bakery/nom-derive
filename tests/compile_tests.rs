#[test]
fn compile_run_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/*.rs");
}

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}

#[cfg(feature = "std")]
#[test]
fn compile_run_pass_std() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass-std/*.rs");
}

#[cfg(feature = "std")]
#[test]
fn compile_fail_std() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail-std/*.rs");
}
