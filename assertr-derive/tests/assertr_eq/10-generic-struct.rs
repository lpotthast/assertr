use renamed_assertr::prelude::*;
use assertr_derive::AssertrEq;
use indoc::formatdoc;

#[derive(Debug, AssertrEq)]
pub struct Generic<'a, T, const N: usize>
where
    T: core::fmt::Debug + PartialEq,
{
    pub value: T,
    pub label: &'a str,
    pub bytes: [u8; N],
}

fn main() {
    let subject = Generic {
        value: 42,
        label: "answer",
        bytes: [1, 2, 3],
    };

    subject.must().be_equal_to(GenericAssertrEq {
        value: eq(42),
        label: eq("answer"),
        bytes: any(),
    });

    let _: GenericAssertrEq<'_, i32, 3> = GenericAssertrEq::default();

    // Failure output must render generic field values instead of falling back to `<unrendered>`.
    let failures = subject.verify(|it| {
        it.with_location(false).be_equal_to(GenericAssertrEq {
            value: eq(43),
            label: eq("question"),
            bytes: any(),
        })
    });
    ToHumanReadableText.render(&failures[0]).must().be_equal_to(formatdoc! {r#"
            -------- assertr --------
            Expected: GenericAssertrEq {{
                value: Eq::Eq(43),
                label: Eq::Eq("question"),
                bytes: Eq::Any,
            }}

              Actual: Generic {{
                value: 42,
                label: "answer",
                bytes: [
                    1,
                    2,
                    3,
                ],
            }}

            Details:
              - Differences: [
                    "value": expected 43, but was 42,
                    "label": expected "question", but was "answer",
                ]
            -------- assertr --------
        "#});
}
