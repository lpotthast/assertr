use core::fmt::Debug;
use core::fmt::Write;
use core::ops::Bound;
use core::ops::RangeBounds;
use indoc::writedoc;

use crate::{AssertThat, Mode, tracking::AssertionTracking};

/// Assertions for any type `R` representing a range (using bound type `B`).
pub trait RangeBoundAssertions<B, R: RangeBounds<B>> {
    fn contains_element(&self, expected: B)
    where
        R: Debug,
        B: PartialOrd + Debug;

    fn contain_element(&self, expected: B)
    where
        R: Debug,
        B: PartialOrd + Debug,
    {
        self.contains_element(expected)
    }

    fn does_not_contain_element(&self, expected: B)
    where
        R: Debug,
        B: PartialOrd + Debug;

    fn not_contain_element(&self, expected: B)
    where
        R: Debug,
        B: PartialOrd + Debug,
    {
        self.does_not_contain_element(expected)
    }
}

/// Assertions for any type `B` that can interact with a range `R` (using bound type `B`).
pub trait RangeAssertions<B> {
    fn is_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd + Debug;

    fn be_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd + Debug,
        Self: Sized,
    {
        self.is_in_range(expected)
    }

    fn is_not_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd + Debug;

    fn not_be_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd + Debug,
        Self: Sized,
    {
        self.is_not_in_range(expected)
    }

    fn is_outside_of_range(self, expected: impl RangeBounds<B>) -> Self
    where
        Self: Sized,
        B: PartialOrd + Debug,
    {
        self.is_not_in_range(expected)
    }

    fn be_outside_of_range(self, expected: impl RangeBounds<B>) -> Self
    where
        Self: Sized,
        B: PartialOrd + Debug,
    {
        self.is_outside_of_range(expected)
    }
}

impl<B, R: RangeBounds<B>, M: Mode> RangeBoundAssertions<B, R> for AssertThat<'_, R, M> {
    #[track_caller]
    fn contains_element(&self, expected: B)
    where
        R: Debug,
        B: PartialOrd + Debug,
    {
        self.track_assertion();
        if !self.actual().contains(&expected) {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Actual range: {actual:#?}

                    Does not contain expected: {expected:#?}
                "#,actual = self.actual()}
            })
        }
    }

    #[track_caller]
    fn does_not_contain_element(&self, expected: B)
    where
        R: Debug,
        B: PartialOrd + Debug,
    {
        self.track_assertion();
        if self.actual().contains(&expected) {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Actual range: {actual:#?}

                    Contains element expected not to be contained: {expected:#?}
                "#,actual = self.actual()}
            })
        }
    }
}

impl<B, M: Mode> RangeAssertions<B> for AssertThat<'_, B, M> {
    fn is_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd + Debug,
    {
        self.track_assertion();

        let actual = self.actual();

        if !expected.contains(actual) {
            let mut range = String::new();
            render_range(&mut range, expected);
            self.fail(|err: &mut String| {
                writedoc! {err, r#"
                    Actual: {actual:#?}
                    is not in range: {range}
                "#}
            });
        }

        self
    }

    fn is_not_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd + Debug,
    {
        self.track_assertion();

        let actual = self.actual();

        if expected.contains(actual) {
            let mut range = String::new();
            render_range(&mut range, expected);
            self.fail(|err: &mut String| {
                writedoc! {err, r#"
                    Actual: {actual:#?}
                    was not expected to be in range: {range}
                "#}
            });
        }

        self
    }
}

fn render_range<B: Debug>(w: &mut impl Write, range: impl RangeBounds<B>) {
    fn write_bound<W: Write, B: Debug>(to: &mut W, bound: &B) {
        to.write_fmt(format_args!("{bound:?}")).unwrap()
    }

    match range.start_bound() {
        Bound::Included(b) => write_bound(w, b),
        Bound::Excluded(b) => write_bound(w, b),
        Bound::Unbounded => {}
    };
    w.write_str("..").unwrap();
    match range.end_bound() {
        Bound::Included(b) => {
            w.write_char('=').unwrap();
            write_bound(w, b);
        }
        Bound::Excluded(b) => write_bound(w, b),
        Bound::Unbounded => {}
    };
}

#[cfg(test)]
mod tests {

    mod contains_element {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_element_is_contained() {
            ("aa"..="zz").must().contain_element("aa");
            ("aa"..="zz").must().contain_element("ab");
            ("aa"..="zz").must().contain_element("ac");
            ("aa"..="zz").must().contain_element("zx");
            ("aa"..="zz").must().contain_element("zy");
            ("aa"..="zz").must().contain_element("zz");
        }

        #[test]
        fn panics_when_element_is_not_contained() {
            assert_that_panic_by(|| {
                ("aa".."zz")
                    .assert()
                    .with_location(false)
                    .contains_element("zz")
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual range: "aa".."zz"
                    
                    Does not contain expected: "zz"
                    -------- assertr --------
                "#});
        }
    }

    mod does_not_contain_element {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_element_is_not_contained() {
            ("aa"..="zz").must().not_contain_element("a");
            ("aa"..="zz").must().not_contain_element("AA");
        }

        #[test]
        fn panics_when_element_is_contained() {
            assert_that_panic_by(|| {
                ("aa".."zz")
                    .assert()
                    .with_location(false)
                    .does_not_contain_element("cc")
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual range: "aa".."zz"
                    
                    Contains element expected not to be contained: "cc"
                    -------- assertr --------
                "#});
        }
    }

    mod is_in_range {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_in_range() {
            'a'.must().be_in_range('a'..='z');
            'p'.must().be_in_range('a'..='z');
            'z'.must().be_in_range('a'..='z');
        }

        #[test]
        fn panics_when_not_in_range() {
            assert_that_panic_by(|| 'A'.assert().with_location(false).is_in_range('a'..='z'))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: 'A'
                    is not in range: 'a'..='z'
                    -------- assertr --------
                "#});
        }
    }

    mod is_not_in_range {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_not_in_range() {
            (-1).must().not_be_in_range(0..=7);
            8.must().not_be_in_range(0..=7);
            9.must().not_be_in_range(0..=7);
        }

        #[test]
        fn panics_when_in_range() {
            assert_that_panic_by(|| 5.assert().with_location(false).is_not_in_range(0..=7))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: 5
                    was not expected to be in range: 0..=7
                    -------- assertr --------
                "#});
        }
    }
}
