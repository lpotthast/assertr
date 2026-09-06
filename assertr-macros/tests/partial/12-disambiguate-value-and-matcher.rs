use renamed_assertr::{
    matchers::{MatchContext, MatchResult, as_matcher, equal_to},
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

impl<R> AssertrMatcher<i32, R> for Both {
    fn evaluate(&self, actual: &i32, _: &mut MatchContext<'_, R>) -> MatchResult {
        MatchResult::new(*actual >= self.0)
    }
}

struct Score {
    value: i32,
}

fn main() {
    // `as_matcher` selects the >= comparison. `equal_to` selects ordinary equality.
    assert_that!(Score { value: 2 }).matches(partial!(Score {
        value: as_matcher(Both(1)),
    }));
    assert_that!(Score { value: 2 }).matches(partial!(Score {
        value: equal_to(Both(2)),
    }));
}
