use crate::{AssertionContext, expectation::Evidence};
use alloc::vec::Vec;
use core::borrow::Borrow;

use super::{Collection, identity, value};
use crate::expectation::satisfying;
use crate::{
    AssertThat, Expectation, ExpectationDiagnostics, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
    mode::Capture,
};

/// A reusable collection membership assertion requiring at least one matching element.
///
/// Evaluation stops at the first match. On rejection, it retains the original child failures,
/// ordered and limited according to the collection's presentation and active rendering budget.
/// Empty collections reject the assertion. No collection or element renderer is required beyond
/// the capabilities of the supplied matcher.
/// [`CollectionAssertions::contains_matching`] executes this same definition.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::collection::ContainsMatching;
/// use assertr::matchers::equal_to;
///
/// let expected = ContainsMatching::new(equal_to(2));
/// assert_that!([1, 2, 3]).matches(&expected);
/// ```
pub struct ContainsMatching<M>(M);

/// Matches collections containing at least one matching element.
///
/// This is a convenience constructor for [`ContainsMatching::new`].
pub fn contains_matching<M>(matcher: M) -> ContainsMatching<M> {
    ContainsMatching::new(matcher)
}

impl<M> ContainsMatching<M> {
    /// Owns the element matcher. Pass a reference to reuse a borrowed matcher.
    #[must_use]
    pub const fn new(matcher: M) -> Self {
        Self(matcher)
    }
}

impl<C: Collection + ?Sized, R, M> Expectation<C, R> for ContainsMatching<M>
where
    M: ExpectationDiagnostics<C::Item, R>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = Evidence
    where
        Self: 'a,
        C: 'a;
    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated_for_order(C::PRESENTATION.order());
        for item in actual.elements() {
            let mut branch = context.isolated();
            if branch.evaluate(item, &self.0) {
                return Ok(());
            }
            context.append(branch.into_evidence());
        }
        if context.evidence.is_empty() {
            context.outcome(false, |context| {
                FailureBuilder::detached::<()>(FailureKind::Matching)
                    .relation("contains a matching element")
                    .children([context.describe(&self.0)])
                    .build()
            });
        }
        Err(context.into_evidence())
    }
}
impl<C: Collection + ?Sized, R, M> ExpectationDiagnostics<C, R> for ContainsMatching<M>
where
    M: ExpectationDiagnostics<C::Item, R>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;
    fn explain<Target>(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure
                .relation("contains a matching element")
                .children([context.describe(&self.0)]),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}

/// Checks that a collection contains no element satisfying the supplied assertion.
/// Its rejection identifies the unwanted elements. Each element is tested once.
pub struct ContainsNoMatching<M>(M);

/// Constructs an assertion that rejects collections with any matching elements.
pub fn contains_no_matching<M>(matcher: M) -> ContainsNoMatching<M> {
    ContainsNoMatching::new(matcher)
}

impl<M> ContainsNoMatching<M> {
    /// Owns the element expectation, which may itself be a reference.
    #[must_use]
    pub const fn new(matcher: M) -> Self {
        Self(matcher)
    }
}

impl<C: Collection + ?Sized, R: ValueRenderer<C::Item>, M: ExpectationDiagnostics<C::Item, R>>
    Expectation<C, R> for ContainsNoMatching<M>
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = Evidence
    where
        Self: 'a,
        C: 'a;
    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings
            .isolated()
            .isolated_for_order(C::PRESENTATION.order());
        let mut found = false;
        for item in actual.elements() {
            if context.probe(item, &self.0) {
                found = true;
                if context.is_diagnostic() {
                    context.record(
                        FailureBuilder::detached::<C::Item>(FailureKind::Membership)
                            .actual(context.render().value(item))
                            .relation("matches the unwanted constraint")
                            .constraint(context.describe(&self.0))
                            .build(),
                    );
                } else {
                    context.outcome(false, |context| context.describe(&self.0));
                }
            }
        }
        if found {
            Err(context.into_evidence())
        } else {
            Ok(())
        }
    }
}
impl<C: Collection + ?Sized, R: ValueRenderer<C::Item>, M: ExpectationDiagnostics<C::Item, R>>
    ExpectationDiagnostics<C, R> for ContainsNoMatching<M>
{
    const KIND: FailureKind = FailureKind::Membership;
    const FLATTEN: bool = true;
    fn explain<Target>(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure
                .relation("contains no matching elements")
                .children([context.describe(&self.0)]),
            Some((_, evidence)) => evidence.explain(failure.relation("contains matching elements")),
        }
    }
}

/// Assertions over the elements of a collection: slices, arrays, `Vec`, `VecDeque`, and every type
/// implementing [`Collection`].
///
/// The collection structure is rendered by Assertr, so value-based methods require rendering
/// support for the element type rather than the collection type. Identity methods display addresses
/// and require no rendering support.
///
/// For a type that supports borrowed traversal but does not implement [`Collection`], use
/// [`IntoIteratorAssertions`](crate::assertions::core::iter::IntoIteratorAssertions). Its methods
/// carry the `into_iter_` prefix.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait CollectionAssertions<T, R> {
    /// Asserts that at least one element borrows the same instance as `expected`.
    ///
    /// Compares `Borrow<U>` targets with [`core::ptr::eq`], without equality or rendering bounds.
    /// Stored values, references, and smart pointers are supported. The expected reference's type
    /// selects the borrowed view. For unsized targets, both address and metadata must match.
    /// Trait-object equality inherits `ptr::eq`'s vtable caveats, and distinct zero-sized values
    /// can have equal addresses. Identity is not a unique allocation or logical object ID.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// struct Key { _opaque: u8 }
    /// let keys = [Key { _opaque: 1 }, Key { _opaque: 1 }];
    /// assert_that!([&keys[0]])
    ///     .contains_same_instance_as(&keys[0])
    ///     .does_not_contain_same_instance_as(&keys[1]);
    /// ```
    fn contains_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        T: Borrow<U>;

    /// Asserts that no element borrows the same instance as `expected`.
    ///
    /// Uses the borrowed-target identity semantics of
    /// [`contains_same_instance_as`](Self::contains_same_instance_as), without rendering bounds.
    fn does_not_contain_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        T: Borrow<U>;

    /// Asserts that the collection borrows exactly the expected instances, ignoring order.
    ///
    /// Every expected occurrence must match a distinct actual occurrence, so duplicate counts must
    /// match. Uses the borrowed-target identity semantics of
    /// [`contains_same_instance_as`](Self::contains_same_instance_as), without rendering bounds.
    /// Arrays, slices, and vectors of expected references are accepted. An empty expectation may
    /// need a type annotation, such as `[] as [&Key; 0]`, to select the borrowed target.
    fn contains_exactly_same_instances_in_any_order<'e, U: ?Sized + 'e>(
        self,
        expected: impl AsRef<[&'e U]>,
    ) -> Self
    where
        T: Borrow<U>;

    /// Asserts that at least one element equals `expected`.
    fn contains<E>(self, expected: E) -> Self
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that at least one element matches `expected`.
    fn contains_matching<P>(self, expected: P) -> Self
    where
        P: ExpectationDiagnostics<T, R>;

    /// Asserts that at least one element satisfies `assertions`.
    ///
    /// The assertions run in capture mode against each element. An element matches when no
    /// assertion failure is raised. On failure, every element's captured failures are reported.
    /// Only the callback's assertions need rendering support. The element itself need not be
    /// renderable.
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Asserts that every expected element has an equal element in the subject.
    ///
    /// Extra subject elements are allowed. Expectations are independent, so duplicates do not
    /// require distinct matches. Use `contains_exactly_in_any_order` for multiset equality.
    ///
    /// `E` is the element type of the expected values, which only has to be comparable to `T`, not
    /// identical to it. Any iterable of expected values is accepted, including another collection.
    fn contains_all<E, I>(self, expected: I) -> Self
    where
        T: PartialEq<E>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that no element equals `not_expected`.
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that no element matches `expected`.
    fn does_not_contain_matching<P>(self, expected: P) -> Self
    where
        P: ExpectationDiagnostics<T, R>,
        R: ValueRenderer<T>;

    /// Asserts that no element satisfies `assertions`.
    ///
    /// The assertions run in capture mode against each element. An element matches when no
    /// assertion failure is raised. On failure, the matching elements are reported.
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Asserts multiset equality with `expected`.
    ///
    /// Order is ignored, but each expected element must match a distinct subject element, so
    /// duplicate counts must match. [`PartialEq`] permits different element types.
    fn contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts one-to-one matching between subject elements and predicates, independent of order.
    ///
    /// A maximum matching makes overlapping predicates order-independent.
    fn contains_exactly_in_any_order_matching<P>(self, expected: P) -> Self
    where
        P: crate::expectation::MatcherList<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts one-to-one matching between subject elements and assertion closures, independent of
    /// order. A maximum matching makes overlapping assertions order-independent.
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: Clone + ValueRenderer<usize>,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);
}

impl<C, M, R> CollectionAssertions<C::Item, R> for AssertThat<'_, C, M, R>
where
    C: Collection,
    M: Mode,
{
    #[track_caller]
    fn contains_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        C::Item: Borrow<U>,
    {
        self.apply_assertion(identity::ContainsSameInstanceAs::new(expected))
    }

    #[track_caller]
    fn does_not_contain_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        C::Item: Borrow<U>,
    {
        self.apply_assertion(identity::DoesNotContainSameInstanceAs::new(expected))
    }

    #[track_caller]
    fn contains_exactly_same_instances_in_any_order<'e, U: ?Sized + 'e>(
        self,
        expected: impl AsRef<[&'e U]>,
    ) -> Self
    where
        C::Item: Borrow<U>,
    {
        self.apply_assertion(identity::ContainsExactlySameInstancesInAnyOrder::new(
            expected,
        ))
    }

    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        C::Item: PartialEq<E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        self.apply_assertion(value::Contains::new(expected))
    }

    #[track_caller]
    fn contains_matching<P>(self, expected: P) -> Self
    where
        P: ExpectationDiagnostics<C::Item, R>,
    {
        self.apply_assertion(ContainsMatching::new(expected))
    }

    #[track_caller]
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.contains_matching(satisfying(assertions))
    }

    #[track_caller]
    fn contains_all<E, I>(self, expected: I) -> Self
    where
        C::Item: PartialEq<E>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        self.track_assertion();
        let expected = expected.into_iter().collect::<Vec<_>>();
        self.apply_assertion_after_tracking(value::ContainsAll::new(expected))
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        C::Item: PartialEq<E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        self.apply_assertion(value::DoesNotContain::new(not_expected))
    }

    #[track_caller]
    fn does_not_contain_matching<P>(self, expected: P) -> Self
    where
        P: ExpectationDiagnostics<C::Item, R>,
        R: ValueRenderer<C::Item>,
    {
        self.apply_assertion(ContainsNoMatching::new(expected))
    }

    #[track_caller]
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.does_not_contain_matching(satisfying(assertions))
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: PartialEq<E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        self.apply_assertion(value::ContainsExactlyInAnyOrder::new(expected))
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<P>(self, expected: P) -> Self
    where
        P: crate::expectation::MatcherList<C::Item, R>,
        R: ValueRenderer<usize>,
    {
        self.track_assertion();
        self.apply_assertion_after_tracking(
            crate::assertions::collection::elements_are_in_any_order(expected),
        )
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: Clone + ValueRenderer<usize>,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.track_assertion();
        self.apply_assertion_after_tracking(
            crate::assertions::collection::elements_are_in_any_order(
                assertions
                    .as_ref()
                    .iter()
                    .map(satisfying)
                    .collect::<Vec<_>>(),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{
                NoRenderer, RendererActual, RendererExpected, SENTINEL, SentinelRenderer,
                assert_trait_impl,
            },
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => CollectionAssertions<i32, NoRenderer>
            );
            assert_trait_impl!(
                crate::assertions::collection::ContainsMatching<
                    crate::expectation::Predicate<fn(&i32) -> bool>
                > => crate::Expectation<[i32], NoRenderer>
            );
        }

        #[test]
        fn equality_and_failures_use_the_active_renderer_type() {
            assert_that!([RendererActual(1), RendererActual(2)].as_slice())
                .with_renderer(SentinelRenderer)
                .contains(RendererExpected(2))
                .contains_all([RendererExpected(1)])
                .contains_exactly_in_any_order([RendererExpected(2), RendererExpected(1)]);

            let failures = assert_that!([RendererActual(1)].as_slice())
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.contains(RendererExpected(2)));
            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
        }
    }

    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain(2);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2, 3].as_slice()), contains(4));
        }

        #[test]
        fn succeeds_when_value_is_present() {
            assert_that!([1, 2, 3].as_slice()).contains(2);
        }

        #[test]
        fn compiles_for_comparable_but_different_element_types() {
            assert_that!(["foo"].as_slice()).contains("foo".to_owned());
        }

        #[test]
        fn preserves_the_chain_subject_name() {
            let failures = assert_that!(vec![1, 2, 3])
                .with_subject_name("the elements")
                .capture(|it| it.contains(4));

            assert_that!(&failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| value.subject_name.as_deref())
                        .is_equal_to(Some("the elements"));
                },
            ]);
        }

        #[test]
        fn panics_when_value_is_missing() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains(4);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain

                    Expected: 4
                    -------- assertr --------
                "});
        }
    }

    mod contains_matching {
        use crate::{
            assertions::{collection::contains_matching, core::partial_eq::equal_to},
            expectation::{
                all_of,
                test_support::{assert_bounded_order, bounded_failures},
            },
            prelude::*,
            renderer::IntoRendered,
            test_support::{NoRenderer, UnorderedSet, rendered_text},
        };
        use core::{cell::Cell, fmt};
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .as_slice()
                .must()
                .contain_matching(crate::expectation::predicate(|it: &i32| *it % 2 == 0));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                contains_matching(crate::expectation::predicate(|it: &i32| *it > 7))
            );
        }

        #[test]
        fn succeeds_when_an_element_matches() {
            let calls = Cell::new(0);
            assert_that!([1, 2, 3].as_slice()).contains_matching(crate::expectation::predicate(
                |it: &i32| {
                    calls.set(calls.get() + 1);
                    *it % 2 == 0
                },
            ));
            assert_that!(calls.get()).is_equal_to(2);
        }

        #[test]
        fn empty_collection_retains_the_constraint_without_a_renderer() {
            let failures = assert_that!([] as [i32; 0])
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_matching(crate::expectation::anything()));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive_owned(|failure| {
                            failure
                                .constraint
                                .as_ref()
                                .unwrap()
                                .relation
                                .as_deref()
                                .unwrap()
                        })
                        .is_equal_to("contains a matching element");
                },
            ]);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_matching(crate::expectation::predicate(|it: &i32| *it > 7));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                does not match

                Nested failures:
                  - does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                  - does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                  - does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }
        struct ReverseRenderer<'a>(&'a Cell<usize>);

        impl ValueRenderer<i32> for ReverseRenderer<'_> {
            fn fmt(&self, value: &i32, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.set(self.0.get() + 1);
                write!(formatter, "{}", 10 - value)
            }
        }

        #[test]
        fn bounded_evidence_is_independent_of_iteration_order() {
            assert_bounded_order(&contains_matching(equal_to(9)));
        }

        #[test]
        fn sorts_nested_branches_before_limiting_them() {
            let matcher = contains_matching(all_of(matchers![equal_to(9), equal_to(0)]));
            let failures = bounded_failures(&[3, 2, 1], &matcher, 1);

            assert_that!(failures[0].children).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    assert_that!(failures[0].omitted_children).is_equal_to(5);
                    element
                        .derive_owned(|item| item.expected.as_ref())
                        .is_equal_to(Some(
                            &AssertionContext::default()
                                .render()
                                .value(&0)
                                .into_rendered(),
                        ));
                },
            ]);
            assert_bounded_order(&matcher);
        }

        #[test]
        fn selects_evidence_using_the_active_renderer_and_skips_zero_budget_rendering() {
            for limit in [0, 1] {
                let renders = Cell::new(0);
                let failures = assert_that!(UnorderedSet(vec![1, 2, 3]))
                    .with_renderer(ReverseRenderer(&renders))
                    .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
                    .capture(|it| it.matches(contains_matching(equal_to(9))));

                assert_that!(renders.get()).is_equal_to(if limit == 0 { 0 } else { 6 });
                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure
                            .derive(|failure| &failure.omitted_children)
                            .is_equal_to(3 - limit);
                        failure
                            .derive(|failure| &failure.children)
                            .contains_exactly_satisfying(vec![
                                |child: AssertThat<
                                    AssertionFailure,
                                    Capture,
                                >| {
                                    child.derive(|child| &child.actual).is_some_satisfying(
                                        |actual| {
                                            actual.derive_owned(rendered_text).is_equal_to("7");
                                        },
                                    );
                                };
                                limit
                            ]);
                    },
                ]);
            }
        }
    }

    mod contains_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain_satisfying(|it| {
                it.is_equal_to(2);
            });
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2].as_slice()),
                contains_satisfying(|it| {
                    it.is_equal_to(7);
                })
            );
        }

        #[test]
        fn succeeds_when_an_element_satisfies() {
            assert_that!([1, 2, 3].as_slice()).contains_satisfying(|it| {
                it.is_equal_to(2);
            });
        }

        #[test]
        fn panics_when_no_element_satisfies_and_lists_every_elements_failures() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].as_slice())
                    .with_location(false)
                    .contains_satisfying(|it| {
                        it.is_equal_to(7);
                    });
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].as_slice()`

                does not match

                Nested failures:
                  - Expected: 7

                      Actual: 1
                  - Expected: 7

                      Actual: 2
                -------- assertr --------
            "});
        }

        #[test]
        fn rendering_budget_limits_items_and_nested_failure_values() {
            let failures = assert_that!([123_456, 234_567, 345_678].as_slice())
                .with_rendering_budget(
                    RenderingBudget::default()
                        .with_max_items(1)
                        .with_max_leaf_characters(3),
                )
                .with_location(false)
                .capture(|it| {
                    it.contains_satisfying(|element| {
                        element.is_equal_to(99);
                    })
                });

            assert_that!(failures[0].children.as_slice()).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| value.actual.as_ref().map(rendered_text))
                        .is_equal_to(Some("123... 3 more characters ...".to_owned()));
                },
            ]);
            assert_that!(failures[0].omitted_children).is_equal_to(2);
        }

        #[test]
        fn opaque_callback_failures_preserve_custom_rendering_and_budgets() {
            use crate::test_support::{CustomValueRenderer, assert_custom_value};

            struct Opaque(usize);

            fn check(it: AssertThat<'_, Opaque, Capture, CustomValueRenderer>) {
                it.satisfies(
                    |value| &value.0,
                    |value| {
                        value.is_equal_to(9);
                    },
                );
            }

            let values = [Opaque(1), Opaque(2)];
            let failures = assert_that!(values)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .capture(|it| it.contains_satisfying(check));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].omitted_children).is_equal_to(1);
            let child = &failures[0].children[0];
            assert_custom_value(child.actual.as_ref().unwrap(), &1_usize);
            assert_custom_value(child.expected.as_ref().unwrap(), &9_usize);
        }
    }

    mod contains_all {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain_all([1, 3]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2].as_slice()), contains_all([1, 42]));
        }

        #[test]
        fn succeeds_when_all_expected_values_are_present() {
            assert_that!([1, 2, 3].as_slice()).contains_all([1, 3]);
        }

        #[test]
        fn succeeds_with_vec_input() {
            assert_that!([1, 2, 3].as_slice()).contains_all(vec![1, 3]);
        }

        #[test]
        fn ignores_duplicate_expectations_and_extra_actual_elements() {
            assert_that!([1, 2, 3].as_slice()).contains_all([1, 1, 1]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(["foo"].as_slice()).contains_all(["foo".to_owned()]);
            assert_that!(["foo".to_owned()].as_slice()).contains_all(["foo"]);
        }

        #[test]
        fn panics_when_any_expected_value_is_absent() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].as_slice())
                    .with_location(false)
                    .contains_all([1, 42]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2].as_slice()`

                    Actual: [
                        1,
                        2,
                    ]

                    does not contain all of

                    Expected: [
                        1,
                        42,
                    ]

                    Details:
                      - Elements not found: [
                            42,
                        ]
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .as_slice()
                .must()
                .not_contain_matching(crate::expectation::predicate(|it: &i32| *it > 7));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                does_not_contain_matching(crate::expectation::predicate(|it: &i32| *it % 2 == 1))
            );
        }

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!([1, 2, 3].as_slice())
                .does_not_contain_matching(crate::expectation::predicate(|it: &i32| *it > 7));
        }

        #[test]
        fn panics_when_elements_match_and_lists_them() {
            assert_that_panic_by(|| {
                assert_that!([-1, 7, 12].as_slice())
                    .with_location(false)
                    .does_not_contain_matching(crate::assertions::core::partial_ord::ge(5));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[-1, 7, 12].as_slice()`

                contains matching elements

                Nested failures:
                  - Actual: 7

                    matches the unwanted constraint

                    Constraint:
                        is greater than or equal to

                        Expected: 5
                  - Actual: 12

                    matches the unwanted constraint

                    Constraint:
                        is greater than or equal to

                        Expected: 5
                -------- assertr --------
            "});
        }
    }

    mod does_not_contain_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().not_contain_satisfying(|it| {
                it.is_equal_to(7);
            });
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                does_not_contain_satisfying(|it| {
                    it.is_greater_than(1);
                })
            );
        }

        #[test]
        fn succeeds_when_no_element_satisfies() {
            assert_that!([1, 2, 3].as_slice()).does_not_contain_satisfying(|it| {
                it.is_equal_to(7);
            });
        }

        #[test]
        fn panics_when_elements_satisfy_and_lists_them() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .does_not_contain_satisfying(|it| {
                        it.is_greater_than(1);
                    });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                contains matching elements

                Nested failures:
                  - Actual: 2

                    matches the unwanted constraint

                    Constraint:
                        satisfies the assertions
                  - Actual: 3

                    matches the unwanted constraint

                    Constraint:
                        satisfies the assertions
                -------- assertr --------
            "});
        }
    }

    mod does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().not_contain(4);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2, 3].as_slice()), does_not_contain(2));
        }

        #[test]
        fn succeeds_when_value_is_absent() {
            assert_that!([1, 2, 3].as_slice()).does_not_contain(4);
        }

        #[test]
        fn panics_when_value_is_present() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .does_not_contain(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    contains

                    Unexpected: 2
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order {
        use crate::prelude::*;

        use indoc::formatdoc;

        #[derive(Debug)]
        struct Actual(u8);

        #[derive(Debug)]
        struct Expected(u8);

        #[derive(Debug)]
        enum WildcardExpected {
            Any,
            Value(u8),
        }

        #[cfg(feature = "matchers")]
        #[derive(Debug)]
        struct DerivedActual {
            pub value: u8,
        }

        impl PartialEq<Expected> for Actual {
            fn eq(&self, other: &Expected) -> bool {
                self.0 == other.0
            }
        }

        impl<R> crate::Expectation<Actual, R> for WildcardExpected {
            type Success<'a>
                = ()
            where
                Self: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a;
            fn evaluate(&self, actual: &Actual, _: &AssertionContext<'_, R>) -> Result<(), ()> {
                match self {
                    Self::Any => Ok(()),
                    Self::Value(value) if actual.0 == *value => Ok(()),
                    Self::Value(_) => Err(()),
                }
            }
        }
        impl<R> ExpectationDiagnostics<Actual, R> for WildcardExpected {
            const KIND: crate::FailureKind = crate::FailureKind::Matching;
            fn explain<Target>(
                &self,
                rejected: Option<(&Actual, ())>,
                failure: crate::failure::FailureBuilder<Target>,
                context: &AssertionContext<'_, R>,
            ) -> crate::failure::FailureBuilder<Target> {
                match rejected {
                    None => failure.relation("matches the wildcard constraint"),
                    Some((_, ())) => failure.constraint(context.describe(&self)),
                }
            }
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .as_slice()
                .must()
                .contain_exactly_in_any_order([2, 3, 1]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1].as_slice()),
                contains_exactly_in_any_order([1, 1])
            );
        }

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order([2, 3, 1]);
        }

        #[test]
        fn supports_heterogeneous_partial_eq() {
            assert_that!([Actual(1), Actual(2)].as_slice())
                .contains_exactly_in_any_order([Expected(2), Expected(1)]);
        }

        #[test]
        fn supports_non_equivalence_matchers() {
            assert_that!([Actual(2), Actual(1)].as_slice()).contains_exactly_in_any_order_matching(
                [WildcardExpected::Any, WildcardExpected::Value(2)],
            );
        }

        #[test]
        #[cfg(feature = "matchers")]
        fn supports_structural_wildcards() {
            let actual = [DerivedActual { value: 2 }, DerivedActual { value: 1 }];
            assert_that!(actual.as_slice()).contains_exactly_in_any_order_matching(matchers![
                partial!(DerivedActual {
                    value: crate::expectation::anything()
                }),
                partial!(DerivedActual {
                    value: crate::matchers::eq(2)
                }),
            ]);
        }

        #[test]
        fn rejects_different_multiplicities() {
            assert_that_panic_by(|| {
                assert_that!([1].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order([1, 1]);
            })
            .has_type::<String>()
            .contains("Elements not found");
        }

        #[test]
        fn panics_when_slice_contains_unknown_data() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order([2, 3, 4]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain exactly in any order

                    Expected: [
                        2,
                        3,
                        4,
                    ]

                    Details:
                      - Elements not found: [
                            4,
                        ]
                      - Elements not expected: [
                            1,
                        ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .as_slice()
                .must()
                .contain_exactly_in_any_order_matching(crate::expectation::predicate_list(
                    [
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 2,
                        move |it: &i32| *it == 3,
                    ]
                    .as_slice(),
                ));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1]),
                contains_exactly_in_any_order_matching([
                    crate::assertions::core::partial_eq::equal_to(2)
                ])
            );
        }

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_matching(
                crate::expectation::predicate_list(
                    [
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 2,
                        move |it: &i32| *it == 3,
                    ]
                    .as_slice(),
                ),
            );
        }

        #[test]
        fn succeeds_when_slices_match_in_different_order() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_matching(
                crate::expectation::predicate_list(
                    [
                        move |it: &i32| *it == 3,
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 2,
                    ]
                    .as_slice(),
                ),
            );
        }

        #[test]
        fn succeeds_when_overlapping_predicates_have_an_exact_assignment() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it <= 2, |it| *it == 1];

            assert_that!([1, 2].as_slice()).contains_exactly_in_any_order_matching(
                crate::expectation::predicate_list(predicates),
            );
        }

        #[test]
        fn rejects_unmatched_predicates() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it == 1, |it| *it == 2];
            assert_that_panic_by(|| {
                assert_that!([1].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(crate::expectation::predicate_list(
                        predicates,
                    ));
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1].as_slice()`

                does not match

                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        satisfies the predicate

                    Details:
                      - at slot: 1
                    Nested failures:
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_slice_contains_non_matching_data() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(crate::expectation::predicate_list(
                        [
                            move |it: &i32| *it == 2,
                            move |it: &i32| *it == 3,
                            move |it: &i32| *it == 4,
                        ]
                        .as_slice(),
                    ));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                does not match

                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        satisfies the predicate

                    Details:
                      - at slot: 2
                    Nested failures:
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                  - has unexpected elements

                    Details:
                      - unexpected count: 1
                    Nested failures:
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn unordered_evaluates_each_visited_pair_once() {
            let count = core::cell::Cell::new(0);
            let first = crate::expectation::predicate(|a: &i32| {
                count.set(count.get() + 1);
                *a <= 2
            });
            let second = crate::expectation::predicate(|a: &i32| {
                count.set(count.get() + 1);
                *a == 1
            });
            assert_that!([1, 2]).contains_exactly_in_any_order_matching(matchers![first, second]);
            assert_that!(count.get()).is_less_or_equal_to(4);
        }
    }

    mod contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2]
                .as_slice()
                .must()
                .contain_exactly_in_any_order_satisfying([
                    |it: AssertThat<i32, Capture>| {
                        it.is_equal_to(2);
                    },
                    |it: AssertThat<i32, Capture>| {
                        it.is_equal_to(1);
                    },
                ]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1]),
                contains_exactly_in_any_order_satisfying([|it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                }])
            );
        }

        #[test]
        fn succeeds_when_assertions_are_satisfied_in_different_order() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(3);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
            ]);
        }

        #[test]
        fn succeeds_when_overlapping_assertions_have_an_exact_assignment() {
            assert_that!([1, 2].as_slice()).contains_exactly_in_any_order_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_less_than(3);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
            ]);
        }

        #[test]
        fn panics_when_elements_are_unmatched() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_satisfying([
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(2);
                        },
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(3);
                        },
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(4);
                        },
                    ]);
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                does not match

                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        satisfies the assertions

                    Details:
                      - at slot: 2
                    Nested failures:
                      - Expected: 4

                          Actual: 1
                      - Expected: 4

                          Actual: 2
                      - Expected: 4

                          Actual: 3
                  - has unexpected elements

                    Details:
                      - unexpected count: 1
                    Nested failures:
                      - Expected: 2

                          Actual: 1
                      - Expected: 3

                          Actual: 1
                -------- assertr --------
            "});
        }
    }

    #[cfg(feature = "std")]
    mod evaluation {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "std")]
        fn tracks_before_consuming_expected_values() {
            struct PanickingValues;
            impl IntoIterator for PanickingValues {
                type Item = i32;
                type IntoIter = core::iter::Empty<i32>;
                fn into_iter(self) -> Self::IntoIter {
                    panic!("expected iterator conversion");
                }
            }
            let failures = assert_that!([1, 2]).capture(|root| {
                let child = root.derive(|values| values);
                let panic = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                    child.contains_all(PanickingValues);
                }));
                assert_that!(panic).is_err();
                root
            });
            assert_that!(failures).is_empty();
        }
    }
}
