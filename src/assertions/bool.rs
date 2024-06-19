use crate::{failure::ExpectedActualFailure, AssertThat, Mode};

/// Assertions for booleans.
impl<'t, M: Mode> AssertThat<'t, bool, M> {
    #[track_caller]
    pub fn is_true(self) -> Self {
        let actual = self.actual().borrowed();
        let expected = &true;
        if actual != expected {
            self.fail(ExpectedActualFailure { expected, actual });
        }
        self
    }

    #[track_caller]
    pub fn is_false(self) -> Self {
        let actual = self.actual().borrowed();
        let expected = &false;
        if actual != expected {
            self.fail(ExpectedActualFailure { expected, actual });
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
