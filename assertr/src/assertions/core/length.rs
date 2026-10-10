use crate::{
    AssertThat, Mode,
    assertions::HasLength,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{DebugRenderer, Rendered, RenderingContext, ValueRenderer},
};

// Emptiness failures show the subject type next to its value.
fn with_type_hint<T: ?Sized, R: ValueRenderer<T>>(
    render: RenderingContext<'_, R>,
    value: &T,
) -> Rendered {
    render.value(value).show_type_hint(true)
}

property_expectation! {
    /// Checks whether a subject implementing [`HasLength`] is empty.
    pub struct IsEmpty for<T: HasLength> T;
    kind Length;
    check |actual| actual.is_empty();
    relations "is empty", "is not empty";
    present with_type_hint;
}

property_expectation! {
    /// Checks whether a subject implementing [`HasLength`] is not empty.
    pub struct IsNotEmpty for<T: HasLength> T;
    kind Length;
    check |actual| !actual.is_empty();
    relations "is not empty", "is unexpectedly empty";
    present with_type_hint;
}

/// Checks a finite length and retains the observed count on rejection.
#[derive(Debug, Clone, Copy)]
pub struct HasLengthOf(usize);
impl HasLengthOf {
    /// Requires exactly this many elements or bytes according to the subject's native length.
    #[must_use]
    pub const fn new(expected: usize) -> Self {
        Self(expected)
    }
}

impl<T: HasLength + ?Sized, R> Expectation<T, R> for HasLengthOf
where
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = usize
    where
        Self: 'a,
        T: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let length = actual.length();
        if length == self.0 {
            Ok(())
        } else {
            Err(length)
        }
    }

    const KIND: FailureKind = FailureKind::Length;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let length = rejected
            .as_ref()
            .map(|(_, length)| Fact::labelled("Actual length", render.value(length)));
        failure
            .relations(
                rejected.map(|(actual, _)| with_type_hint(render, actual)),
                "has length",
                "does not have the expected length",
            )
            .facts(length)
            .expected(render.value(&self.0))
    }
}

/// Assertions for subjects implementing [`HasLength`].
///
/// Failures render the whole subject, so these methods require a renderer for the subject type
/// itself. This also applies to collections and maps, whose element assertions need only element,
/// key, or value rendering support.
///
/// [`HasLength`]: crate::assertions::HasLength
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait LengthAssertions<T: HasLength, R = DebugRenderer> {
    /// Asserts that the subject has length zero.
    fn is_empty(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the subject has nonzero length.
    fn is_not_empty(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the subject has exactly `expected` elements or bytes.
    fn has_length(self, expected: usize) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<usize>;
}

impl<T: HasLength, M: Mode, R> LengthAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_empty(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsEmpty)
    }

    #[track_caller]
    fn is_not_empty(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsNotEmpty)
    }

    #[track_caller]
    fn has_length(self, expected: usize) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<usize>,
    {
        self.matches(HasLengthOf::new(expected))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            ([] as [i32; 0]).must().be_empty();
            [42].must().not_be_empty().have_length(1);
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<u8>, Panic, NoRenderer> => LengthAssertions<Vec<u8>, NoRenderer>
            );
        }

        #[test]
        fn failures_render_the_subject_and_counts_with_the_active_renderer() {
            use crate::test_support::{CustomValueRenderer, assert_custom_value};

            let failures = assert_that!([1, 2])
                .with_renderer(CustomValueRenderer)
                .capture(|it| it.is_empty().has_length(3));
            assert_that!(failures).has_length(2);
            for failure in &failures {
                assert_that!(failure.to_string()).contains("Actual: [i32; 2] custom([1, 2])");
            }
            assert_custom_value(failures[1].expected.as_ref().unwrap(), &3_usize);
            assert_custom_value(&failures[1].facts[0].value, &2_usize);
        }

        #[test]
        fn passing_checks_render_nothing() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("success rendered")
                }
            }
            assert_that!([] as [i32; 0])
                .with_renderer(NeverRender)
                .is_empty()
                .has_length(0);
            assert_that!([1, 2])
                .with_renderer(NeverRender)
                .is_not_empty()
                .has_length(2);
        }
    }

    mod is_empty {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1]), is_empty());
        }

        #[test]
        fn succeeds_when_empty() {
            let arr: [i32; 0] = [];
            assert_that!(arr).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that!(|| assert_that!([1, 2, 3]).with_location(false).is_empty())
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3]`

                Actual: [i32; 3] [
                    1,
                    2,
                    3,
                ]

                is not empty
                -------- assertr --------
            "});
        }
    }

    mod is_not_empty {
        use alloc::collections::VecDeque;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(VecDeque::<i32>::new()), is_not_empty());
        }

        #[test]
        fn rejects_empty_subjects() {
            assert_that!(VecDeque::from([42])).is_not_empty();
            let failures =
                assert_that!(VecDeque::<i32>::new()).capture(LengthAssertions::is_not_empty);
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("is unexpectedly empty"));
        }
    }

    mod has_length {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1].as_slice()), has_length(2));
        }

        #[test]
        fn succeeds_when_length_matches() {
            let empty: &[i32] = [].as_slice();
            assert_that!(empty).has_length(0);
            assert_that!([1, 2, 3].as_slice()).has_length(3);
        }

        #[test]
        fn panics_when_length_does_not_match() {
            assert_that!(|| {
                assert_that!([42].as_slice())
                    .with_location(false)
                    .has_length(2);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[42].as_slice()`

                    Actual: [i32] [
                        42,
                    ]

                    does not have the expected length

                    Expected: 2

                    Details:
                      - Actual length: 1
                    -------- assertr --------
                "});
        }
    }

    // Assertion behavior and diagnostics live above. These checks cover built-in HasLength
    // adapters.
    mod adapters {
        use alloc::{
            borrow::Cow,
            boxed::Box,
            collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
        };

        use crate::prelude::*;

        #[test]
        fn sequences() {
            assert_that!([] as [i32; 0]).is_empty().has_length(0);
            assert_that!([1, 2]).is_not_empty().has_length(2);
            assert_that!([].as_slice() as &[i32])
                .is_empty()
                .has_length(0);
            assert_that!([1, 2].as_slice()).is_not_empty().has_length(2);
            assert_that!(Vec::<i32>::new()).is_empty().has_length(0);
            assert_that_owned!(vec![1, 2]).is_not_empty().has_length(2);
            assert_that!(VecDeque::from([1, 2]))
                .is_not_empty()
                .has_length(2);
            assert_that!(LinkedList::from([1, 2]))
                .is_not_empty()
                .has_length(2);
            assert_that!(BinaryHeap::from([1, 2]))
                .is_not_empty()
                .has_length(2);
        }

        #[test]
        fn strings_use_byte_length() {
            assert_that!("").is_empty().has_length(0);
            assert_that!("é🦀").is_not_empty().has_length(6);
            assert_that!(String::from("é🦀")).has_length(6);
            assert_that!(Box::<str>::from("é🦀")).has_length(6);
            assert_that!(Cow::Borrowed("é🦀")).has_length(6);
            assert_that!(Cow::<str>::Owned(String::new())).is_empty();
        }

        #[test]
        fn maps_and_sets() {
            assert_that!(BTreeMap::<i32, i32>::new()).is_empty();
            assert_that!(BTreeMap::from([(1, 2), (3, 4)])).has_length(2);
            assert_that!(BTreeSet::from([1, 2, 2])).has_length(2);
        }

        #[test]
        #[cfg(feature = "std")]
        fn hash_maps_and_sets() {
            use std::collections::{HashMap, HashSet};

            assert_that!(HashMap::<i32, i32>::new()).is_empty();
            assert_that!(HashMap::from([(1, 2), (3, 4)])).has_length(2);
            assert_that!(HashSet::from([1, 2, 2])).has_length(2);
        }

        #[test]
        fn references() {
            let mut deque = VecDeque::<i32>::new();
            assert_that!(&deque).is_empty().has_length(0);
            assert_that!(&mut deque).is_empty().has_length(0);
            deque.push_back(42);
            assert_that!(&deque).is_not_empty().has_length(1);
            assert_that!(&mut deque).is_not_empty().has_length(1);
        }

        #[test]
        fn ranges_reject_lengths_that_exceed_usize() {
            assert_that!(|| assert_that!(0_usize..=usize::MAX).has_length(0))
                .panics()
                .has_type::<&str>()
                .is_equal_to("range length exceeds usize::MAX");
            assert_that!(|| assert_that!(i64::MIN..=i64::MAX).has_length(0))
                .panics()
                .has_type::<&str>()
                .is_equal_to("range length exceeds usize::MAX");
        }
    }

    mod evaluation {
        use core::cell::Cell;

        use crate::{assertions::HasLength, prelude::*};

        #[derive(Debug)]
        struct Length {
            reads: Cell<usize>,
            empty_checks: Cell<usize>,
        }
        impl HasLength for Length {
            fn length(&self) -> usize {
                self.reads.set(self.reads.get() + 1);
                7
            }
            fn is_empty(&self) -> bool {
                self.empty_checks.set(self.empty_checks.get() + 1);
                false
            }
        }

        #[test]
        fn uses_the_native_empty_override_and_retains_the_observed_length() {
            let actual = Length {
                reads: Cell::new(0),
                empty_checks: Cell::new(0),
            };
            let failures =
                assert_that!(actual).capture(|it| it.is_empty().is_not_empty().has_length(8));
            assert_that!((actual.reads.get(), actual.empty_checks.get())).is_equal_to((1, 2));
            assert_that!(failures).has_length(2);
            assert_that!(format!("{:#}", failures[1].facts[0].value)).is_equal_to("7");
        }
    }
}
