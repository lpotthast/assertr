use alloc::string::String;
use core::fmt::Write;
use core::ops::Bound;
use core::ops::RangeBounds;
use indoc::writedoc;

use crate::{AssertThat, Mode, ValueRenderer};

/// Assertions over a range subject's membership.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RangeBoundAssertions<B, Range: RangeBounds<B>, R = crate::DebugRenderer> {
    /// Asserts that the range contains `expected`.
    fn contains_element(&self, expected: B)
    where
        B: PartialOrd,
        R: ValueRenderer<B>;

    /// Asserts that the range does not contain `expected`.
    fn does_not_contain_element(&self, expected: B)
    where
        B: PartialOrd,
        R: ValueRenderer<B>;
}

/// Assertions over a value subject's membership in a range.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RangeAssertions<B, R = crate::DebugRenderer> {
    /// Asserts that the subject is within `expected`.
    fn is_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>;

    /// Asserts that the subject is outside `expected`.
    fn is_not_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>;

    /// Alias of [`RangeAssertions::is_not_in_range`].
    fn is_outside_of_range(self, expected: impl RangeBounds<B>) -> Self
    where
        Self: Sized,
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.is_not_in_range(expected)
    }
}

impl<B, Range: RangeBounds<B>, M: Mode, R> RangeBoundAssertions<B, Range, R>
    for AssertThat<'_, Range, M, R>
{
    #[track_caller]
    fn contains_element(&self, expected: B)
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.track_assertion();
        if !self.actual().contains(&expected) {
            let actual = render_range(self, self.actual());
            let expected = self.render_value(&expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual range: {actual}

                    Does not contain expected: {expected:#?}
                "}
            });
        }
    }

    #[track_caller]
    fn does_not_contain_element(&self, expected: B)
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.track_assertion();
        if self.actual().contains(&expected) {
            let actual = render_range(self, self.actual());
            let expected = self.render_value(&expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual range: {actual}

                    Contains element expected not to be contained: {expected:#?}
                "}
            });
        }
    }
}

impl<B, M: Mode, R> RangeAssertions<B, R> for AssertThat<'_, B, M, R> {
    #[track_caller]
    fn is_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.track_assertion();

        let actual = self.actual();

        if !expected.contains(actual) {
            let range = render_range(&self, &expected);
            let actual = self.render_value(actual);
            self.fail(|err: &mut String| {
                writedoc! {err, r"
                    Actual: {actual:#?}
                    is not in range: {range}
                "}
            });
        }

        self
    }

    #[track_caller]
    fn is_not_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.track_assertion();

        let actual = self.actual();

        if expected.contains(actual) {
            let range = render_range(&self, &expected);
            let actual = self.render_value(actual);
            self.fail(|err: &mut String| {
                writedoc! {err, r"
                    Actual: {actual:#?}
                    was not expected to be in range: {range}
                "}
            });
        }

        self
    }
}

fn render_range<B, S, Range: RangeBounds<B> + ?Sized, M: Mode, R>(
    assert_that: &AssertThat<'_, S, M, R>,
    range: &Range,
) -> String
where
    R: ValueRenderer<B>,
{
    fn write_bound<B, S, M: Mode, R>(
        to: &mut impl Write,
        assert_that: &AssertThat<'_, S, M, R>,
        bound: &B,
    ) where
        R: ValueRenderer<B>,
    {
        let bound = assert_that.render_value(bound);
        to.write_fmt(format_args!("{bound:?}")).unwrap();
    }

    let mut rendered = String::new();
    match range.start_bound() {
        Bound::Included(b) | Bound::Excluded(b) => write_bound(&mut rendered, assert_that, b),
        Bound::Unbounded => {}
    }
    rendered.write_str("..").unwrap();
    match range.end_bound() {
        Bound::Included(b) => {
            rendered.write_char('=').unwrap();
            write_bound(&mut rendered, assert_that, b);
        }
        Bound::Excluded(b) => write_bound(&mut rendered, assert_that, b),
        Bound::Unbounded => {}
    }
    rendered
}

#[cfg(test)]
mod tests {

    mod contains_element {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ("aa"..="zz").must().contain_element("aa");
        }

        #[test]
        fn succeeds_when_element_is_contained() {
            assert_that!("aa"..="zz").contains_element("aa");
            assert_that!("aa"..="zz").contains_element("ab");
            assert_that!("aa"..="zz").contains_element("ac");
            assert_that!("aa"..="zz").contains_element("zx");
            assert_that!("aa"..="zz").contains_element("zy");
            assert_that!("aa"..="zz").contains_element("zz");
        }

        #[test]
        fn panics_when_element_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that!("aa".."zz")
                    .with_location(false)
                    .contains_element("zz");
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ("aa"..="zz").must().not_contain_element("a");
        }

        #[test]
        fn succeeds_when_element_is_not_contained() {
            assert_that!("aa"..="zz").does_not_contain_element("a");
            assert_that!("aa"..="zz").does_not_contain_element("AA");
        }

        #[test]
        fn panics_when_element_is_contained() {
            assert_that_panic_by(|| {
                assert_that!("aa".."zz")
                    .with_location(false)
                    .does_not_contain_element("cc");
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            'a'.must().be_in_range('a'..='z');
        }

        #[test]
        fn succeeds_when_in_range() {
            assert_that!('a').is_in_range('a'..='z');
            assert_that!('p').is_in_range('a'..='z');
            assert_that!('z').is_in_range('a'..='z');
        }

        #[test]
        fn panics_when_not_in_range() {
            assert_that_panic_by(|| {
                assert_that!('A')
                    .with_location(false)
                    .is_in_range('a'..='z')
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: 'A'
                    is not in range: 'a'..='z'
                    -------- assertr --------
                "});
        }
    }

    mod is_not_in_range {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            (-1).must().not_be_in_range(0..=7);
        }

        #[test]
        fn succeeds_when_not_in_range() {
            assert_that!(-1).is_not_in_range(0..=7);
            assert_that!(8).is_not_in_range(0..=7);
            assert_that!(9).is_not_in_range(0..=7);
        }

        #[test]
        fn panics_when_in_range() {
            assert_that_panic_by(|| assert_that!(5).with_location(false).is_not_in_range(0..=7))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: 5
                    was not expected to be in range: 0..=7
                    -------- assertr --------
                "});
        }
    }

    /// Synonym of `is_not_in_range`. Only the fluent name is pinned here. The behavior is covered by
    /// that module.
    mod is_outside_of_range {
        #[cfg(feature = "fluent")]
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            5.must().be_outside_of_range(1..3);
        }
    }
}
