use super::partial_eq::operand_expectation;
use crate::borrow_for::BorrowFor;
use crate::{
    AssertThat, AssertionContext, DebugRenderer, Expectation, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
    renderer::RenderingContext,
};
use alloc::{
    format,
    string::{String, ToString},
};
use core::marker::PhantomData;
use core::ops::{
    Bound::{Excluded, Included, Unbounded},
    RangeBounds,
};

/// Checks whether a range contains an element.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
pub struct ContainsElement<B, E = B>(E, PhantomData<fn() -> B>);

impl<B> ContainsElement<B> {
    /// Owns the expected operand, using its type as the range bound type.
    ///
    /// The operand determines the type even before this definition is used with a range.
    /// Use [`Self::borrowing`] to select a different bound type for a borrowed operand.
    ///
    /// ```
    /// use assertr::{matchers::range::ContainsElement, prelude::*};
    ///
    /// let expected = ContainsElement::new(&2);
    /// assert_that!(&1..&3).matches(&expected);
    /// ```
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self(expected, PhantomData)
    }

    /// Owns an operand whose borrowed view is selected for bound type `B`.
    ///
    /// Pass a reference to reuse an expected value. Construction does not borrow or clone the
    /// operand. Evaluation borrows it through [`BorrowFor`]. Diagnostics require renderers for
    /// `B` and `E::View`, without requiring one for the operand wrapper.
    /// For borrowed bounds, selecting the pointee type also avoids needing a reference renderer.
    ///
    /// ```
    /// use assertr::{matchers::range::ContainsElement, prelude::*};
    ///
    /// let value = String::from("b");
    /// let expected = ContainsElement::<String>::borrowing(&value);
    /// assert_that!(String::from("a")..String::from("c")).matches(&expected);
    /// ```
    #[must_use]
    pub const fn borrowing<E: BorrowFor<B>>(expected: E) -> ContainsElement<B, E> {
        ContainsElement(expected, PhantomData)
    }
}

operand_expectation! {
    impl [B, E, Range: RangeBounds<B> + ?Sized, R] for ContainsElement<B, E>, subject Range, where [
        B: PartialOrd<E::View>,
        E: BorrowFor<B>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>,
    ];
    borrow 0 for B, view E::View;
    kind Membership;
    holds |actual, expected| actual.contains(expected);
    actual |render, actual| render_range(render, actual);
    expected "contains", "does not contain";
}

/// Checks whether a range does not contain an element.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
pub struct DoesNotContainElement<B, E = B>(E, PhantomData<fn() -> B>);

impl<B> DoesNotContainElement<B> {
    /// Owns the unexpected operand, using its type as the range bound type.
    ///
    /// The operand determines the type even before this definition is used with a range.
    /// Use [`Self::borrowing`] to select a different bound type for a borrowed operand.
    ///
    /// ```
    /// use assertr::{matchers::range::DoesNotContainElement, prelude::*};
    ///
    /// let unexpected = DoesNotContainElement::new(&3);
    /// assert_that!(&1..&3).matches(&unexpected);
    /// ```
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self(expected, PhantomData)
    }

    /// Owns an operand whose borrowed view is selected for bound type `B`.
    ///
    /// Construction, borrowing, and rendering follow [`ContainsElement::borrowing`].
    ///
    /// ```
    /// use assertr::{matchers::range::DoesNotContainElement, prelude::*};
    ///
    /// let value = String::from("z");
    /// let unexpected = DoesNotContainElement::<String>::borrowing(&value);
    /// assert_that!(String::from("a")..String::from("c")).matches(&unexpected);
    /// ```
    #[must_use]
    pub const fn borrowing<E: BorrowFor<B>>(expected: E) -> DoesNotContainElement<B, E> {
        DoesNotContainElement(expected, PhantomData)
    }
}

operand_expectation! {
    impl [B, E, Range: RangeBounds<B> + ?Sized, R] for DoesNotContainElement<B, E>, subject Range,
    where [
        B: PartialOrd<E::View>,
        E: BorrowFor<B>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>,
    ];
    borrow 0 for B, view E::View;
    kind Membership;
    holds |actual, expected| actual.contains(expected);
    actual |render, actual| render_range(render, actual);
    unexpected "does not contain", "contains";
}

/// Implements `Clone` and `Debug` for a containment definition through its stored operand only.
macro_rules! operand_traits {
    ($($name:ident),+) => {$(
        impl<B, E: Clone> Clone for $name<B, E> {
            fn clone(&self) -> Self {
                Self(self.0.clone(), PhantomData)
            }
        }

        impl<B, E: core::fmt::Debug> core::fmt::Debug for $name<B, E> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.debug_tuple(stringify!($name)).field(&self.0).finish()
            }
        }
    )+};
}

operand_traits!(ContainsElement, DoesNotContainElement);

/// Checks whether a value is in range.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
#[derive(Debug, Clone)]
pub struct IsInRange<Range>(Range);

impl<Range> IsInRange<Range> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: Range) -> Self {
        Self(expected)
    }
}

impl<B: PartialOrd, Range: RangeBounds<B>, R: ValueRenderer<B>> Expectation<B, R>
    for IsInRange<Range>
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        B: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        B: 'a;
    fn evaluate<'a>(&'a self, actual: &'a B, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if self.0.contains(actual) {
            Ok(())
        } else {
            Err(())
        }
    }

    const KIND: FailureKind = FailureKind::Ordering;
    fn explain(
        &self,
        rejected: Option<(&B, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure
            .relations(
                rejected.map(|(actual, ())| render.value(actual)),
                "is in range",
                "is not in range",
            )
            .expected(render_range(render, &self.0))
    }
}

/// Checks whether a value is not in range.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
#[derive(Debug, Clone)]
pub struct IsNotInRange<Range>(Range);

impl<Range> IsNotInRange<Range> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: Range) -> Self {
        Self(expected)
    }
}

impl<B: PartialOrd, Range: RangeBounds<B>, R: ValueRenderer<B>> Expectation<B, R>
    for IsNotInRange<Range>
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        B: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        B: 'a;
    fn evaluate<'a>(&'a self, actual: &'a B, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if self.0.contains(actual) {
            Err(())
        } else {
            Ok(())
        }
    }

    const KIND: FailureKind = FailureKind::Ordering;
    fn explain(
        &self,
        rejected: Option<(&B, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure
            .relations(
                rejected.map(|(actual, ())| render.value(actual)),
                "is not in range",
                "is in range",
            )
            .unexpected(render_range(render, &self.0))
    }
}

/// Assertions over a range subject's membership.
///
/// Diagnostics use Rust range notation when possible. Ranges with excluded lower bounds use
/// explicit bound tuples, such as `(Excluded(1), Included(3))`.
///
/// Standard ranges with borrowed, sized bounds have inherent methods that select the bounds'
/// pointee type. This keeps owned and borrowed elements inferable even though those ranges
/// implement both `RangeBounds<B>` and `RangeBounds<&B>`. Custom ranges use this trait, and fully
/// qualified calls can select `B` explicitly when a range supports more than one bound type.
/// These methods execute [`ContainsElement::borrowing`] and [`DoesNotContainElement::borrowing`]
/// with the selected bound type. Reusable matchers' `new` constructors instead infer the bound
/// type from the operand, including its reference type when a reference is passed.
///
/// ```
/// use assertr::prelude::*;
/// let lower = String::from("a");
/// let upper = String::from("z");
/// let element = String::from("m");
/// assert_that!(&lower..&upper).contains_element(&element);
/// ```
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
#[allow(clippy::return_self_not_must_use)]
pub trait RangeBoundAssertions<B, Range: RangeBounds<B>, R = DebugRenderer> {
    /// Asserts that the range contains `expected`.
    fn contains_element<E: BorrowFor<B>>(self, expected: E) -> Self
    where
        B: PartialOrd<E::View>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>;

    /// Asserts that the range does not contain `expected`.
    fn does_not_contain_element<E: BorrowFor<B>>(self, expected: E) -> Self
    where
        B: PartialOrd<E::View>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>;
}

/// Assertions over a value subject's membership in a range.
///
/// Ranges are displayed as described in [`RangeBoundAssertions`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RangeAssertions<B, R = DebugRenderer> {
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
    #[track_caller]
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
    fn contains_element<E: BorrowFor<B>>(self, expected: E) -> Self
    where
        B: PartialOrd<E::View>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>,
    {
        self.apply_assertion(ContainsElement::<B>::borrowing(expected))
    }

    #[track_caller]
    fn does_not_contain_element<E: BorrowFor<B>>(self, expected: E) -> Self
    where
        B: PartialOrd<E::View>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>,
    {
        self.apply_assertion(DoesNotContainElement::<B>::borrowing(expected))
    }
}

// Standard ranges with borrowed bounds implement both RangeBounds<B> and RangeBounds<&B>.
// Inherent methods select the pointee view before borrowing the operand, avoiding ambiguity
// in the blanket assertion trait. Custom ranges keep using that trait unchanged.
macro_rules! borrowed_range_assertions {
    ($($range:ty),+ $(,)?) => {$(
        #[allow(clippy::return_self_not_must_use)]
        impl<'b, B, M: Mode, R> AssertThat<'_, $range, M, R> {
            borrowed_range_assertions! {
                @method
                /// Asserts that this range contains `expected`, using its bounds' pointee type.
                ///
                /// This is [`RangeBoundAssertions::contains_element`] with the native
                /// `RangeBounds<B>` view selected explicitly for borrowed bounds.
                contains_element => RangeBoundAssertions::<B, $range, R>::contains_element;
                /// Asserts that this range excludes `expected`, using its bounds' pointee type.
                ///
                /// This is [`RangeBoundAssertions::does_not_contain_element`] with the native
                /// `RangeBounds<B>` view selected explicitly for borrowed bounds.
                does_not_contain_element =>
                    RangeBoundAssertions::<B, $range, R>::does_not_contain_element;
                /// Fluent alias of [`Self::contains_element`].
                #[cfg(feature = "fluent")]
                contain_element => Self::contains_element;
                /// Fluent alias of [`Self::does_not_contain_element`].
                #[cfg(feature = "fluent")]
                not_contain_element => Self::does_not_contain_element;
            }
        }
    )+};
    (@method $($(#[$attr:meta])* $name:ident => $target:path;)+) => {$(
        $(#[$attr])*
        #[track_caller]
        pub fn $name<E: BorrowFor<B>>(self, expected: E) -> Self
        where
            B: PartialOrd<E::View>,
            E::View: PartialOrd<B>,
            R: ValueRenderer<B> + ValueRenderer<E::View>,
        {
            $target(self, expected)
        }
    )+};
}

borrowed_range_assertions!(
    core::ops::Range<&'b B>,
    core::ops::RangeInclusive<&'b B>,
    core::ops::RangeFrom<&'b B>,
    core::ops::RangeTo<&'b B>,
    core::ops::RangeToInclusive<&'b B>,
    (core::ops::Bound<&'b B>, core::ops::Bound<&'b B>),
);

impl<B, M: Mode, R> RangeAssertions<B, R> for AssertThat<'_, B, M, R> {
    #[track_caller]
    fn is_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.apply_assertion(IsInRange::new(expected))
    }

    #[track_caller]
    fn is_not_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.apply_assertion(IsNotInRange::new(expected))
    }
}

fn render_range<B, Range: RangeBounds<B> + ?Sized, R>(
    rendering: RenderingContext<'_, R>,
    range: &Range,
) -> String
where
    R: ValueRenderer<B>,
{
    /// Rendered bound text, displayed verbatim inside `Bound`'s `Debug` syntax.
    struct Leaf(String);

    impl core::fmt::Debug for Leaf {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(&self.0)
        }
    }

    // Bounds are embedded inline, so their leaves use the compact form.
    let rendering = rendering.compact();
    let start = range
        .start_bound()
        .map(|value| Leaf(rendering.value(value).to_string()));
    let end = range
        .end_bound()
        .map(|value| Leaf(rendering.value(value).to_string()));

    match (start, end) {
        // Rust's range operators cannot express an excluded start. Format the bounds around
        // their rendered leaves so the active renderer and budget still apply to each leaf.
        (start @ Excluded(_), end) => format!("({start:?}, {end:?})"),
        (Included(start), Included(end)) => format!("{start:?}..={end:?}"),
        (Included(start), Excluded(end)) => format!("{start:?}..{end:?}"),
        (Included(start), Unbounded) => format!("{start:?}.."),
        (Unbounded, Included(end)) => format!("..={end:?}"),
        (Unbounded, Excluded(end)) => format!("..{end:?}"),
        (Unbounded, Unbounded) => "..".into(),
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Bound::{self, Excluded, Included, Unbounded};

    use crate::prelude::*;
    use indoc::formatdoc;

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::*;

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn are_as_expected() {
            ("aa"..="zz")
                .must()
                .contain_element("aa")
                .not_contain_element("a");
            (&1..&4).must().contain_element(&2).not_contain_element(&4);
            (Included(&1), Excluded(&4))
                .must()
                .contain_element(&2)
                .not_contain_element(&4);
            'a'.must()
                .be_in_range('a'..='z')
                .not_be_in_range('b'..)
                .be_outside_of_range('b'..);
        }
    }

    mod renderer_contract {
        use core::ops::{Bound, RangeBounds};

        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, core::ops::Range<i32>, Panic, NoRenderer>
                    => RangeBoundAssertions<i32, core::ops::Range<i32>, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => RangeAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn failures_render_bounds_and_values_with_the_active_renderer() {
            let bound_failures = assert_that!(1..3)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.contains_element(4));
            assert_that!(bound_failures[0].to_string()).contains(format!("{SENTINEL}..{SENTINEL}"));

            let value_failures = assert_that!(1)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.is_in_range(2..=3));
            assert_that!(value_failures[0].to_string())
                .contains(SENTINEL)
                .contains(format!("{SENTINEL}..={SENTINEL}"));
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn borrowed_bounds_render_pointees_without_reference_or_clone_support() {
            struct BoundRenderer;
            impl ValueRenderer<i32> for BoundRenderer {
                fn fmt(&self, value: &i32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "bound({value})")
                }
            }

            let failures = assert_that!(&1..&3)
                .with_renderer(BoundRenderer)
                .with_location(false)
                .capture(|it| {
                    it.contains_element(&4)
                        .matches(super::super::ContainsElement::<i32>::borrowing(&4))
                });
            assert_that!(failures).has_length(2);
            for failure in &failures {
                assert_that!(format!("{:#}", failure.actual.as_ref().unwrap()))
                    .is_equal_to("bound(1)..bound(3)");
                assert_that!(format!("{:#}", failure.expected.as_ref().unwrap()))
                    .is_equal_to("bound(4)");
            }
        }

        #[test]
        fn custom_ranges_with_excluded_starts_honor_the_renderer_and_budget() {
            struct OpenStartRange {
                start: i32,
                end: Bound<i32>,
            }

            impl RangeBounds<i32> for OpenStartRange {
                fn start_bound(&self) -> Bound<&i32> {
                    Bound::Excluded(&self.start)
                }

                fn end_bound(&self) -> Bound<&i32> {
                    self.end.as_ref()
                }
            }

            for (end, rendered_end) in [
                (
                    Bound::Included(3),
                    "Included(<ren... 6 more characters ...)",
                ),
                (
                    Bound::Excluded(3),
                    "Excluded(<ren... 6 more characters ...)",
                ),
                (Bound::Unbounded, "Unbounded"),
            ] {
                let range = OpenStartRange { start: 1, end };
                let failures = assert_that!(range)
                    .with_renderer(SentinelRenderer)
                    .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(4))
                    .capture(|it| it.contains_element(1));
                assert_that!(failures).has_length(1);
                assert_that!(format!("{:#}", failures[0].actual.as_ref().unwrap())).is_equal_to(
                    format!("(Excluded(<ren... 6 more characters ...), {rendered_end})"),
                );
                assert_that!(format!("{:#}", failures[0].expected.as_ref().unwrap()))
                    .is_equal_to("<ren... 6 more characters ...");
            }
        }
    }

    mod range_notation {
        use super::*;

        type Bounds = (Bound<i32>, Bound<i32>);

        #[test]
        fn uses_range_syntax_or_explicit_bounds_for_excluded_starts() {
            // Every range contains 2 and excludes 1.
            const CASES: [(Bounds, &str); 4] = [
                ((Included(2), Excluded(3)), "2..3"),
                ((Excluded(1), Included(3)), "(Excluded(1), Included(3))"),
                ((Excluded(1), Excluded(3)), "(Excluded(1), Excluded(3))"),
                ((Excluded(1), Unbounded), "(Excluded(1), Unbounded)"),
            ];
            for (range, rendered_range) in CASES {
                assert_that!(range).contains_element(2);
                let failures = assert_that!(range).capture(|it| it.contains_element(1));
                assert_that!(format!("{:#}", failures[0].actual.as_ref().unwrap()))
                    .is_equal_to(rendered_range);
            }
        }
    }

    mod contains_element {
        use super::*;

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("aa".."zz"), contains_element("zz"));
            assert_caller_location!(assert_that!(&1..&4), contains_element(&4));
        }

        #[test]
        fn succeeds_when_element_is_contained() {
            assert_that!("aa"..="zz")
                .contains_element("aa")
                .contains_element("ac")
                .contains_element("zz");
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn borrowed_bounds_and_elements_infer_without_annotations() {
            let lower = 1;
            let upper = 4;
            let element = 2;
            assert_that!(&lower..&upper)
                .contains_element(element)
                .contains_element(&element);
            assert_that!(&lower..=&upper)
                .contains_element(element)
                .contains_element(&element);
            assert_that!(&lower..)
                .contains_element(element)
                .contains_element(&element);
            assert_that!(..&upper)
                .contains_element(element)
                .contains_element(&element);
            assert_that!(..=&upper)
                .contains_element(element)
                .contains_element(&element);
            assert_that!((Included(&lower), Excluded(&upper)))
                .contains_element(element)
                .contains_element(&element);
        }

        #[test]
        fn fails_when_element_is_not_contained() {
            assert_that_panic_by(|| assert_that!(2..3).with_location(false).contains_element(1))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `2..3`

                Actual: 2..3

                does not contain

                Expected: 1
                -------- assertr --------
            "});
        }
    }

    mod does_not_contain_element {
        use super::*;

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("aa".."zz"), does_not_contain_element("cc"));
            assert_caller_location!(assert_that!(&1..&4), does_not_contain_element(&2));
        }

        #[test]
        fn succeeds_when_element_is_not_contained() {
            assert_that!("aa"..="zz")
                .does_not_contain_element("a")
                .does_not_contain_element("AA");
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn borrowed_bounds_and_elements_infer_without_annotations() {
            let lower = 1;
            let upper = 4;
            assert_that!(&lower..&upper)
                .does_not_contain_element(upper)
                .does_not_contain_element(&upper);
            assert_that!(&lower..=&upper)
                .does_not_contain_element(0)
                .does_not_contain_element(&0);
            assert_that!(&lower..)
                .does_not_contain_element(0)
                .does_not_contain_element(&0);
            assert_that!(..&upper)
                .does_not_contain_element(upper)
                .does_not_contain_element(&upper);
            assert_that!(..=&upper)
                .does_not_contain_element(5)
                .does_not_contain_element(&5);
            assert_that!((Included(&lower), Excluded(&upper)))
                .does_not_contain_element(upper)
                .does_not_contain_element(&upper);
        }

        #[test]
        fn fails_when_element_is_contained() {
            assert_that_panic_by(|| {
                assert_that!(2..3)
                    .with_location(false)
                    .does_not_contain_element(2)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `2..3`

                Actual: 2..3

                contains

                Unexpected: 2
                -------- assertr --------
            "});
        }
    }

    mod is_in_range {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!('A'), is_in_range('a'..='z'));
        }

        #[test]
        fn succeeds_when_in_range() {
            assert_that!('a')
                .is_in_range('a'..='z')
                .is_in_range('a'..)
                .is_in_range(..='a');
        }

        #[test]
        fn fails_when_not_in_range() {
            assert_that_panic_by(|| assert_that!(1).with_location(false).is_in_range(2..3))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `1`

                    Actual: 1

                    is not in range

                    Expected: 2..3
                    -------- assertr --------
                "});
        }
    }

    mod is_not_in_range {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(5), is_not_in_range(0..=7));
        }

        #[test]
        fn succeeds_when_not_in_range() {
            assert_that!(-1).is_not_in_range(0..=7);
            assert_that!(8).is_not_in_range(0..=7);
        }

        #[test]
        fn fails_when_in_range() {
            assert_that_panic_by(|| assert_that!(2).with_location(false).is_not_in_range(2..3))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `2`

                    Actual: 2

                    is in range

                    Unexpected: 2..3
                    -------- assertr --------
                "});
        }
    }

    /// Synonym of `is_not_in_range`. The caller location is pinned here. The behavior is covered
    /// by that module.
    mod is_outside_of_range {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(5), is_outside_of_range(0..=10));
        }
    }

    mod reusable_definitions {
        use super::super::{ContainsElement, DoesNotContainElement};
        use super::*;
        use crate::matchers::{all_of, each};

        #[test]
        fn constructors_infer_without_a_subject_or_comparison_capabilities() {
            struct Opaque;
            let _ = ContainsElement::new(Opaque);
            let _ = DoesNotContainElement::new(Opaque);
            let _ = ContainsElement::new(&2);
            let _ = DoesNotContainElement::new(&2);
            let _ = ContainsElement::new(String::from("a"));
            let _ = DoesNotContainElement::new(String::from("a"));
        }

        #[test]
        fn borrowed_endpoints_infer_for_each_range_shape() {
            assert_that!(&1..&3)
                .matches(ContainsElement::new(&2))
                .matches(DoesNotContainElement::new(&3));
            assert_that!(&1..=&3)
                .matches(ContainsElement::new(&3))
                .matches(DoesNotContainElement::new(&4));
            assert_that!(&1..)
                .matches(ContainsElement::new(&2))
                .matches(DoesNotContainElement::new(&0));
            assert_that!(..&3)
                .matches(ContainsElement::new(&2))
                .matches(DoesNotContainElement::new(&3));
            assert_that!(..=&3)
                .matches(ContainsElement::new(&3))
                .matches(DoesNotContainElement::new(&4));
            assert_that!((Included(&1), Excluded(&3)))
                .matches(ContainsElement::new(&2))
                .matches(DoesNotContainElement::new(&3));
        }

        #[test]
        fn definitions_are_reusable_in_nested_composition() {
            let expected = ContainsElement::new(&2);
            let unexpected = DoesNotContainElement::new(&4);
            let matcher = all_of(matchers![&expected, &unexpected]);
            assert_that!(&1..&3).matches(&expected).matches(&unexpected);
            assert_that!(&1..&4).matches(&matcher);
            assert_that!([&1..&3, &0..&4]).matches(each(&matcher));
        }

        #[test]
        fn borrowing_definitions_are_reusable_across_owned_and_borrowed_bounds() {
            let lower = String::from("a");
            let upper = String::from("c");
            let value = String::from("b");
            let absent = String::from("z");
            let matcher = all_of(matchers![
                ContainsElement::<String>::borrowing(&value),
                DoesNotContainElement::<String>::borrowing(&absent)
            ]);
            assert_that!(String::from("a")..String::from("c")).matches(&matcher);
            assert_that!(&lower..&upper).matches(&matcher);
            assert_that!([&lower..&upper]).matches(each(&matcher));
        }

        #[test]
        fn non_copy_bounds_accept_owned_and_borrowed_elements() {
            let lower = String::from("a");
            let upper = String::from("c");
            let element = String::from("b");
            let absent = String::from("z");
            assert_that!(String::from("a")..String::from("c"))
                .contains_element(&element)
                .does_not_contain_element(&absent);
            assert_that!(&lower..&upper)
                .contains_element(String::from("b"))
                .contains_element(&element)
                .does_not_contain_element(String::from("z"))
                .does_not_contain_element(&absent);
        }
    }
}
