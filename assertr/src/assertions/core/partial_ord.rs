use core::cmp::Ordering;

use super::partial_eq::operand_expectation;
use crate::{
    AssertThat, Mode,
    borrow_for::BorrowFor,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Generates one reusable ordering bound: the struct, its constructors, and its expectation.
/// The bound accepts the listed [`Ordering`] results of `actual.partial_cmp(expected)`.
macro_rules! ordering_expectation {
    (
        $(#[$struct_doc:meta])*
        struct $name:ident;
        $(#[$fn_doc:meta])*
        fn $constructor:ident;
        accepts $($accepted:ident)|+;
        relation $relation:literal;
        rejection $rejection:literal;
    ) => {
        $(#[$struct_doc])*
        #[derive(Debug, Clone)]
        pub struct $name<E> {
            expected: E,
        }

        $(#[$fn_doc])*
        #[must_use]
        pub const fn $constructor<E>(expected: E) -> $name<E> {
            $name::new(expected)
        }

        impl<E> $name<E> {
            /// Owns an expected operand. Pass a reference to reuse an expected value.
            #[must_use]
            pub const fn new(expected: E) -> Self {
                Self { expected }
            }
        }

        operand_expectation! {
            impl [T: ?Sized, E, R] for $name<E>, subject T, where [
                T: PartialOrd<E::View>,
                E: BorrowFor<T>,
                R: ValueRenderer<T> + ValueRenderer<E::View>,
            ];
            borrow expected for T, view E::View;
            kind Ordering;
            holds |actual, expected| matches!(
                actual.partial_cmp(expected),
                Some($(Ordering::$accepted)|+)
            );
            actual |render, actual| render.value(actual);
            expected $relation, $rejection;
        }
    };
}

ordering_expectation! {
    /// Reusable strict upper bound with the borrowing and ordering rules of
    /// [`PartialOrdAssertions::is_less_than`]. Incomparable values do not match.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use assertr::matchers::LessThan;
    ///
    /// let maximum = LessThan::new(65);
    /// assert_that!(42).matches(&maximum);
    /// ```
    struct LessThan;
    /// Matches values less than `expected`. Incomparable values do not match.
    ///
    /// This is a convenience constructor for [`LessThan::new`].
    fn lt;
    accepts Less;
    relation "is less than";
    rejection "is not less than";
}

ordering_expectation! {
    /// Reusable strict lower bound with the borrowing and ordering rules of
    /// [`PartialOrdAssertions::is_greater_than`]. Incomparable values do not match.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use assertr::matchers::GreaterThan;
    ///
    /// let minimum = GreaterThan::new(18);
    /// assert_that!(42).matches(&minimum);
    /// ```
    struct GreaterThan;
    /// Matches values greater than `expected`. Incomparable values do not match.
    ///
    /// This is a convenience constructor for [`GreaterThan::new`].
    fn gt;
    accepts Greater;
    relation "is greater than";
    rejection "is not greater than";
}

ordering_expectation! {
    /// Reusable upper bound with the borrowing and ordering rules of
    /// [`PartialOrdAssertions::is_less_or_equal_to`]. Incomparable values do not match.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use assertr::matchers::LessOrEqual;
    ///
    /// let maximum = LessOrEqual::new(65);
    /// assert_that!(42).matches(&maximum);
    /// ```
    struct LessOrEqual;
    /// Matches values less than or equal to `expected`. Incomparable values do not match.
    ///
    /// This is a convenience constructor for [`LessOrEqual::new`].
    fn le;
    accepts Less | Equal;
    relation "is less than or equal to";
    rejection "is not less than or equal to";
}

ordering_expectation! {
    /// Reusable lower bound with the borrowing and ordering rules of
    /// [`PartialOrdAssertions::is_greater_or_equal_to`]. Incomparable values do not match.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use assertr::matchers::GreaterOrEqual;
    ///
    /// let minimum = GreaterOrEqual::new(18);
    /// assert_that!(42).matches(&minimum);
    /// ```
    struct GreaterOrEqual;
    /// Matches values greater than or equal to `expected`. Incomparable values do not match.
    ///
    /// This is a convenience constructor for [`GreaterOrEqual::new`].
    fn ge;
    accepts Greater | Equal;
    relation "is greater than or equal to";
    rejection "is not greater than or equal to";
}

/// Assertions for partially ordered values.
///
/// Expected values use [`BorrowFor`], with `PartialOrd` and renderer support for the selected
/// view. `String` has no `PartialOrd<str>` implementation, so compare two strings or two string
/// views:
///
/// ```
/// use assertr::prelude::*;
/// let bound = String::from("z");
/// assert_that!(String::from("a")).is_less_than(&bound);
/// assert_that!("a").is_less_than("z");
/// ```
///
/// ```compile_fail
/// use assertr::prelude::*;
/// assert_that!(String::from("a")).is_less_than("z");
/// ```
///
/// Against `str`, `String` selects `str`, but `&String` selects `String`, which `str` cannot order.
/// Use `as_str()` to borrow such a bound:
///
/// ```
/// use assertr::{prelude::*, matchers::{dereferenced, lt}};
/// let bound = String::from("z");
/// assert_that!("a").matches(dereferenced(lt(String::from("z"))));
/// assert_that!("a").matches(dereferenced(lt(bound.as_str())));
/// ```
///
/// ```compile_fail
/// use assertr::{prelude::*, matchers::{dereferenced, lt}};
/// let bound = String::from("z");
/// assert_that!("a").matches(dereferenced(lt(&bound)));
/// ```
///
/// Each assertion requires the corresponding concrete [`Ordering`] result. Incomparable values
/// therefore fail every ordering assertion. In particular, a floating-point comparison involving
/// `NaN` does not satisfy either strict or inclusive ordering.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PartialOrdAssertions<T, R = DebugRenderer> {
    /// Asserts that the subject is strictly less than `expected`.
    fn is_less_than<E>(self, expected: E) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E::View>,
        T: PartialOrd<E::View>,
        E: BorrowFor<T>;

    /// Asserts that the subject is strictly greater than `expected`.
    fn is_greater_than<E>(self, expected: E) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E::View>,
        T: PartialOrd<E::View>,
        E: BorrowFor<T>;

    /// Asserts that the subject is less than or equal to `expected`.
    fn is_less_or_equal_to<E>(self, expected: E) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E::View>,
        T: PartialOrd<E::View>,
        E: BorrowFor<T>;

    /// Asserts that the subject is greater than or equal to `expected`.
    fn is_greater_or_equal_to<E>(self, expected: E) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E::View>,
        T: PartialOrd<E::View>,
        E: BorrowFor<T>;
}

impl<T, M: Mode, R> PartialOrdAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_less_than<E>(self, expected: E) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E::View>,
        T: PartialOrd<E::View>,
        E: BorrowFor<T>,
    {
        self.matches(LessThan::new(expected))
    }

    #[track_caller]
    fn is_greater_than<E>(self, expected: E) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E::View>,
        T: PartialOrd<E::View>,
        E: BorrowFor<T>,
    {
        self.matches(GreaterThan::new(expected))
    }

    #[track_caller]
    fn is_less_or_equal_to<E>(self, expected: E) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E::View>,
        T: PartialOrd<E::View>,
        E: BorrowFor<T>,
    {
        self.matches(LessOrEqual::new(expected))
    }

    #[track_caller]
    fn is_greater_or_equal_to<E>(self, expected: E) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E::View>,
        T: PartialOrd<E::View>,
        E: BorrowFor<T>,
    {
        self.matches(GreaterOrEqual::new(expected))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            3.must()
                .be_less_than(4)
                .be_greater_than(2)
                .be_less_or_equal_to(3)
                .be_greater_or_equal_to(3);
        }
    }

    mod renderer_contract {
        use core::{borrow::Borrow, cmp::Ordering, fmt};

        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => PartialOrdAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn borrowed_operands_only_require_the_target_renderer() {
            struct Actual(&'static str);
            struct Operand(Actual);
            struct Renderer;

            impl PartialEq for Actual {
                fn eq(&self, _: &Actual) -> bool {
                    false
                }
            }

            impl PartialOrd for Actual {
                fn partial_cmp(&self, _: &Actual) -> Option<Ordering> {
                    None
                }
            }
            impl borrow_for::BorrowFor<Actual> for Operand {
                type View = Actual;
            }

            impl Borrow<Actual> for Operand {
                fn borrow(&self) -> &Actual {
                    &self.0
                }
            }

            impl ValueRenderer<Actual> for Renderer {
                fn fmt(&self, value: &Actual, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str(value.0)
                }
            }

            let failures = assert_that!(Actual("actual"))
                .with_renderer(Renderer)
                .capture(|it| {
                    it.is_less_than(Operand(Actual("expected")))
                        .is_greater_than(Operand(Actual("expected")))
                        .is_less_or_equal_to(Operand(Actual("expected")))
                        .is_greater_or_equal_to(Operand(Actual("expected")))
                });
            assert_that!(failures).has_length(4);
            for failure in &failures {
                assert_that!(format!("{:#}", failure.actual.as_ref().unwrap()))
                    .is_equal_to("actual");
                assert_that!(format!("{:#}", failure.expected.as_ref().unwrap()))
                    .is_equal_to("expected");
            }
        }
    }

    mod diagnostics {
        use indoc::formatdoc;

        use crate::{prelude::*, test_support::FailureReportAssertions};

        #[test]
        fn incomparable_values_fail_every_bound() {
            macro_rules! case {
                ($method:ident, $relation:literal) => {{
                    let nan = f32::NAN;
                    let failures = assert_that!(nan)
                        .with_location(false)
                        .capture(|it| it.$method(0.0));
                    assert_that!(&failures[0]).has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Expression: `nan`

                        Actual: NaN

                        {}

                        Expected: 0.0
                        -------- assertr --------
                    ", $relation});
                }};
            }
            case!(is_less_than, "is not less than");
            case!(is_greater_than, "is not greater than");
            case!(is_less_or_equal_to, "is not less than or equal to");
            case!(is_greater_or_equal_to, "is not greater than or equal to");
        }
    }

    mod is_less_than {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::NAN), is_less_than(0.0));
        }

        #[test]
        fn accepts_only_less_values() {
            assert_that!(3).is_less_than(4);
            for actual in [4, 5] {
                let failures = assert_that!(actual).capture(|it| it.is_less_than(4));
                assert_that!(failures).has_length(1);
            }
        }
    }

    mod is_greater_than {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::NAN), is_greater_than(0.0));
        }

        #[test]
        fn accepts_only_greater_values() {
            assert_that!(7).is_greater_than(6);
            for actual in [5, 6] {
                let failures = assert_that!(actual).capture(|it| it.is_greater_than(6));
                assert_that!(failures).has_length(1);
            }
        }
    }

    mod is_less_or_equal_to {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::NAN), is_less_or_equal_to(0.0));
        }

        #[test]
        fn accepts_less_and_equal_values() {
            assert_that!(3)
                .is_less_or_equal_to(4)
                .is_less_or_equal_to(3);
            let failures = assert_that!(4).capture(|it| it.is_less_or_equal_to(3));
            assert_that!(failures).has_length(1);
        }
    }

    mod is_greater_or_equal_to {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::NAN), is_greater_or_equal_to(0.0));
        }

        #[test]
        fn accepts_greater_and_equal_values() {
            assert_that!(7)
                .is_greater_or_equal_to(6)
                .is_greater_or_equal_to(7);
            let failures = assert_that!(6).capture(|it| it.is_greater_or_equal_to(7));
            assert_that!(failures).has_length(1);
        }
    }

    mod borrowed_operands {
        use crate::{
            matchers::{ge, gt, le, lt},
            prelude::*,
        };
        #[derive(Debug, PartialEq, PartialOrd)]
        struct Point(i32, i32);
        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed temporaries are the contract under test.
        fn reusable_borrowed_matchers_and_temporaries() {
            let lower = Point(1, 2);
            let upper = Point(2, 3);
            let below = lt(&upper);
            let above = gt(&lower);
            for _ in 0..2 {
                assert_that!(&lower).matches(&below).matches(le(&lower));
                assert_that!(&upper).matches(&above).matches(ge(&upper));
            }
            assert_that!(Point(1, 2))
                .is_less_than(&upper)
                .is_less_or_equal_to(&Point(1, 2));
            assert_that!(&upper)
                .is_greater_than(Point(1, 2))
                .is_greater_or_equal_to(&lower);
            let context = AssertionContext::default();
            assert_that!(lt(String::from("z")).evaluate("a", &context).is_ok()).is_true();
            assert_that!(ge(vec![1, 2]).evaluate([2, 1].as_slice(), &context).is_ok()).is_true();
        }
    }
}
