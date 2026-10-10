use renamed_assertr::{matchers::eq, prelude::*};

// This expectation supports both equality and matching, with deliberately different behavior.
#[derive(Debug)]
struct Both(i32);

impl renamed_assertr::borrow_for::BorrowFor<i32> for Both {
    type View = i32;
}

impl core::borrow::Borrow<i32> for Both {
    fn borrow(&self) -> &i32 {
        &self.0
    }
}

impl<R> renamed_assertr::expectation::Expectation<i32, R> for Both {
    type Success<'a>
        = ()
    where
        Self: 'a,
        i32: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        i32: 'a;

    fn evaluate(
        &self,
        actual: &i32,
        _: &renamed_assertr::expectation::AssertionContext<'_, R>,
    ) -> Result<(), ()> {
        if *actual >= self.0 { Ok(()) } else { Err(()) }
    }

    const KIND: renamed_assertr::failure::FailureKind = renamed_assertr::failure::FailureKind::Ordering;

    fn explain(
        &self,
        rejected: Option<(&i32, ())>,
        failure: renamed_assertr::failure::FailureBuilder,
        _: &renamed_assertr::expectation::AssertionContext<'_, R>,
    ) -> renamed_assertr::failure::FailureBuilder {
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
    // `eq` selects ordinary equality, even for a value that is also a matcher.
    assert_that!(Score { value: 2 }).matches(partial!(Score {
        value: eq(Both(2)),
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
