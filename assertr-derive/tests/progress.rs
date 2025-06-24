#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse-single-field.rs");
    t.pass("tests/02-parse-multiple-fields.rs");
    t.pass("tests/03-handle-non-pub-fields.rs");
    t.pass("tests/04-equality-check.rs");
    t.pass("tests/05-replace-field-type.rs");
    t.pass("tests/06-replace-deep-field-type.rs");
    t.pass("tests/07-derive-impl-for-reference.rs");
    t.pass("tests/08-default-impl.rs");
}
