#[test]
fn fluent_aliases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fluent_aliases/01-reject-non-string-alias.rs");
    t.compile_fail("tests/fluent_aliases/02-reject-invalid-alias.rs");
}

#[test]
fn fluent_expressions() {
    let t = trybuild::TestCases::new();
    t.pass("tests/fluent_expressions/01-renamed-dependency.rs");
    t.compile_fail("tests/fluent_expressions/02-user-must.rs");
    t.pass("tests/fluent_expressions/03-callback-types.rs");
    t.pass("tests/fluent_expressions/04-facade-runtime-path.rs");
    t.compile_fail("tests/fluent_expressions/05-reject-invalid-arguments.rs");
}

mod partial {
    #[test]
    fn accepts_supported_forms_and_rejects_invalid_inputs() {
        let t = trybuild::TestCases::new();

        // Start with everyday usage, then cover less common forms and inference constraints.
        // The runtime dependency is deliberately renamed in Cargo.toml for every fixture.
        t.pass("tests/partial/01-match-single-field.rs");
        t.pass("tests/partial/02-match-multiple-fields.rs");
        t.pass("tests/partial/03-match-field-with-matcher.rs");
        t.pass("tests/partial/04-match-nested-struct.rs");
        t.pass("tests/partial/05-match-tuple-struct.rs");
        t.pass("tests/partial/06-match-enum-variants.rs");
        t.pass("tests/partial/07-match-unit-struct.rs");
        t.pass("tests/partial/08-reuse-matcher.rs");
        t.pass("tests/partial/09-match-generic-struct.rs");
        t.pass("tests/partial/10-ignore-non-debug-field.rs");
        t.pass("tests/partial/11-handle-field-names.rs");
        t.pass("tests/partial/12-explicit-equality-and-matcher.rs");
        t.pass("tests/partial/13-borrow-temporary-expectations.rs");
        t.pass("tests/partial/14-match-without-renderer.rs");
        t.pass("tests/partial/15-generated-items-are-documented.rs");
        t.pass("tests/partial/16-infer-empty-matcher-list.rs");
        t.pass("tests/partial/30-qualified-variants.rs");
        t.pass("tests/partial/31-invoke-through-reexport.rs");

        // Invalid syntax and fields come before lifetime, inference, and capability boundaries.
        t.compile_fail("tests/partial/17-reject-duplicate-fields.rs");
        t.compile_fail("tests/partial/18-reject-misplaced-rest.rs");
        t.compile_fail("tests/partial/19-reject-missing-fields.rs");
        t.compile_fail("tests/partial/20-reject-unknown-field.rs");
        t.compile_fail("tests/partial/21-reject-wrong-field-type.rs");
        t.compile_fail("tests/partial/22-reject-private-field.rs");
        t.compile_fail("tests/partial/23-reject-unknown-unit-constructors.rs");
        t.compile_fail("tests/partial/24-reject-non-exhaustive-without-rest.rs");
        t.compile_fail("tests/partial/25-reject-outlived-borrow.rs");
        t.compile_fail("tests/partial/26-reject-plain-field-values.rs");
        t.compile_fail("tests/partial/27-reject-plain-value-as-matcher.rs");
        t.compile_fail("tests/partial/28-reject-missing-stable-order.rs");
        t.compile_fail("tests/partial/29-reject-plain-macro-values.rs");
    }
}
