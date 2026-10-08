//! Compile-fail tests for `#[derive(ProjectionField)]` diagnostics.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
