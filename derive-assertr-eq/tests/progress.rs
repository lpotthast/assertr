#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse-single-field.rs");
    t.pass("tests/02-parse-multiple-fields.rs");
    t.pass("tests/03-handle-non-pub-fields.rs");
    t.pass("tests/04-equality-check.rs");
}
