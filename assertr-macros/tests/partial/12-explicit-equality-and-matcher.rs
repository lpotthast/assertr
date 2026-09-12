use renamed_assertr::{
    matchers::{eq, equal_to},
    prelude::*,
};

// This expectation supports both equality and matching, with deliberately different behavior.
#[derive(Debug)]
struct Both(i32);

impl PartialEq<Both> for i32 {
    fn eq(&self, expected: &Both) -> bool {
        *self == expected.0
    }
}

impl<R> renamed_assertr::Expectation<i32, R> for Both {
    type Success<'a> = ();
    type Rejection<'a> = ();
    fn evaluate(
        &self,
        actual: &i32,
        _: &renamed_assertr::AssertionContext<'_, R>,
    ) -> Result<(), ()> {
        if *actual >= self.0 { Ok(()) } else { Err(()) }
    }
}
impl<R> ExpectationDiagnostics<i32, R> for Both {
    const KIND: renamed_assertr::FailureKind = renamed_assertr::FailureKind::Ordering;
    fn explain<Target>(
        &self,
        rejected: Option<(&i32, ())>,
        failure: renamed_assertr::failure::FailureBuilder<Target>,
        _: &renamed_assertr::AssertionContext<'_, R>,
    ) -> renamed_assertr::failure::FailureBuilder<Target> {
        match rejected {
            None => failure.relation("meets the lower bound"),
            Some((_, ())) => failure.relation("is below the lower bound"),
        }
    }
}

struct Score {
    value: i32,
}

fn main() {
    // A matcher remains a matcher even when it also supports equality.
    assert_that!(Score { value: 2 }).matches(partial!(Score { value: Both(1) }));
    // Both equality constructor names select ordinary equality.
    assert_that!(Score { value: 2 }).matches(partial!(Score {
        value: equal_to(Both(2)),
    }));
    let failures = assert_that!(Score { value: 2 })
        .capture(|it| it.matches(partial!(Score { value: eq(Both(1)) })));
    assert_that!(failures).has_length(1);

    assert_that!([2]).matches(elements_are![Both(1)]);
    assert_that!([2]).matches(elements_are_in_any_order![Both(1)]);
    assert_that!([2]).contains_exactly_matching(matchers![Both(1)]);
    assert_that!(std::collections::BTreeMap::from([("value", 2)]))
        .matches(entries_are![("value", Both(1))]);
}
