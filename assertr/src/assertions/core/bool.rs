use crate::{AssertThat, Mode, tracking::AssertionTracking};

/// Assertions for boolean values.
pub trait BoolAssertions {
    fn is_true(self) -> Self;
    fn be_true(self) -> Self
    where
        Self: Sized,
    {
        self.is_true()
    }

    fn is_false(self) -> Self;

    fn be_false(self) -> Self
    where
        Self: Sized,
    {
        self.is_false()
    }
}

impl<M: Mode> BoolAssertions for AssertThat<'_, bool, M> {
    #[track_caller]
    fn is_true(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let expected = &true;
        if actual != expected {
            self.fail(format_args!(
                "Expected: {expected:#?}\n\n  Actual: {actual:#?}\n",
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
            self.fail(format_args!(
                "Expected: {expected:#?}\n\n  Actual: {actual:#?}\n",
            ));
        }
        self
    }
}

#[cfg(test)]
mod tests {

    mod is_true {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_true() {
            true.must().be_true();
        }

        #[test]
        fn panics_when_false() {
            assert_that_panic_by(|| false.must().with_location(false).be_true())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: true

                      Actual: false
                    -------- assertr --------
                "#});
        }
    }

    mod is_false {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_false() {
            false.must().be_false();
        }

        #[test]
        fn panics_when_true() {
            assert_that_panic_by(|| true.must().with_location(false).be_false())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: false

                      Actual: true
                    -------- assertr --------
                "#});
        }
    }
}
