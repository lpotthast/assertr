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
    t.pass("tests/09-custom-renderer-non-debug-field.rs");
    t.pass("tests/10-generic-struct.rs");
    t.compile_fail("tests/11-reject-tuple-struct.rs");
    t.pass("tests/12-generic-private-field.rs");
    t.pass("tests/13-generic-qualified-name-collision.rs");
    t.pass("tests/14-renderer-name-collision.rs");
    t.pass("tests/15-generic-bound-private-dependency.rs");
}
