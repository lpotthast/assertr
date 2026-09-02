#[test]
fn assertr_eq() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assertr_eq/01-parse-single-field.rs");
    t.pass("tests/assertr_eq/02-parse-multiple-fields.rs");
    t.pass("tests/assertr_eq/03-handle-non-pub-fields.rs");
    t.pass("tests/assertr_eq/04-equality-check.rs");
    t.pass("tests/assertr_eq/05-replace-field-type.rs");
    t.pass("tests/assertr_eq/06-replace-deep-field-type.rs");
    t.pass("tests/assertr_eq/07-derive-impl-for-reference.rs");
    t.pass("tests/assertr_eq/08-default-impl.rs");
    t.pass("tests/assertr_eq/09-custom-renderer-non-debug-field.rs");
    t.pass("tests/assertr_eq/10-generic-struct.rs");
    t.pass("tests/assertr_eq/11-generic-private-field.rs");
    t.pass("tests/assertr_eq/12-generic-qualified-name-collision.rs");
    t.pass("tests/assertr_eq/13-renderer-name-collision.rs");
    t.pass("tests/assertr_eq/14-generic-bound-private-dependency.rs");
    t.pass("tests/assertr_eq/15-generated-items-are-documented.rs");
    t.compile_fail("tests/assertr_eq/16-reject-unit-struct.rs");
    t.compile_fail("tests/assertr_eq/17-reject-tuple-struct.rs");
    t.compile_fail("tests/assertr_eq/18-reject-enum.rs");
    t.compile_fail("tests/assertr_eq/19-reject-malformed-compare-bounds.rs");
}

#[test]
fn fluent_expressions() {
    // These fixtures need the matching, unpublished assertr runtime helper while the two crates
    // are prepared for release. They remain workspace tests and are not included in this crate's
    // independently runnable package archive.
    let fixtures =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fluent_expressions");
    if !fixtures.is_dir() {
        return;
    }

    let t = trybuild::TestCases::new();
    t.pass("tests/fluent_expressions/01-renamed-dependency.rs");
    t.pass("tests/fluent_expressions/02-nested-module.rs");
    t.pass("tests/fluent_expressions/03-macro-receiver.rs");
    t.pass("tests/fluent_expressions/04-annotated-closure.rs");
    t.compile_fail("tests/fluent_expressions/05-user-must.rs");
    t.pass("tests/fluent_expressions/06-user-verify.rs");
    t.pass("tests/fluent_expressions/07-user-verify-owned.rs");
}
