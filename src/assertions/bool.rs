use crate::{tracking::AssertionTracking, AssertThat, Mode};

pub trait BoolAssertions {
    fn is_true(self) -> Self;
    fn is_false(self) -> Self;
}

/// Assertions for booleans.
impl<'t, M: Mode> BoolAssertions for AssertThat<'t, bool, M> {
    #[track_caller]
    fn is_true(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let expected = &true;
        if actual != expected {
            self.fail_with_arguments(format_args!(
                "Expected: {expected:#?}\n\n  Actual: {actual:#?}",
            ));
        }
        self
    }

    #[track_caller]
    fn is_false(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let expected = &false;
        if actual != expected {
            self.fail_with_arguments(format_args!(
                "Expected: {expected:#?}\n\n  Actual: {actual:#?}",
            ));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn is_true_succeeds_when_true() {
        assert_that(true).is_true();
    }

    #[test]
    fn is_true_panics_when_false() {
        assert_that_panic_by(|| assert_that(false).with_location(false).is_true())
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected: true

                  Actual: false
                -------- assertr --------
            "#});
    }

    #[test]
    fn is_false_succeeds_when_false() {
        assert_that(false).is_false();
    }

    #[test]
    fn is_false_panics_when_true() {
        assert_that_panic_by(|| assert_that(true).with_location(false).is_false())
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected: false

                  Actual: true
                -------- assertr --------
            "#});
    }
}
