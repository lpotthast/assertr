use crate::{failure::GenericFailure, AssertThat};
use std::fmt::Debug;

/// Comparable
impl<'t, T: PartialOrd> AssertThat<'t, T> {
    #[track_caller]
    pub fn is_less_than(self, expected: T) -> Self
    where
        T: Debug,
    {
        let actual = self.actual.borrowed();
        let expected = &expected;

        if actual < expected {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not less than\n\nExpected: {expected:#?}"
                ),
            });
        }
        self
    }
}
