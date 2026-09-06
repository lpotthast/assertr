use renamed_assertr::{
    matchers::{MatchContext, MatchResult},
    prelude::*,
};

// Both equality and matcher interpretation are valid. Neither may be chosen implicitly.
#[derive(Debug)]
struct Both;

impl PartialEq<Both> for i32 {
    fn eq(&self, _: &Both) -> bool {
        true
    }
}

impl<R> AssertrMatcher<i32, R> for Both {
    fn evaluate(&self, _: &i32, _: &mut MatchContext<'_, R>) -> MatchResult {
        MatchResult::new(true)
    }
}

struct Score {
    value: i32,
}

fn main() {
    // `equal_to(Both)` or `as_matcher(Both)` would make the intent explicit.
    assert_that!(Score { value: 1 }).matches(partial!(Score { value: Both }));
}
