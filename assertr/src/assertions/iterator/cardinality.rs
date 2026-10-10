use super::{
    AssertionContext, Borrow, FailureBuilder, FailureKind, KnownLength, LengthBound,
    PREVIEW_CAPACITY, PhantomData, RenderingOrder, Scan, Tail, ValueRenderer, consumed_fact,
};
use crate::failure::Fact;

/// Requires an iterator to yield no element, consuming at most one.
pub(crate) struct IsEmptyScan<T>(PhantomData<fn() -> T>);

impl<T> IsEmptyScan<T> {
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T, I, R> Scan<I, R> for IsEmptyScan<T>
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    type Rejection = I::Item;

    fn kind(&self) -> FailureKind {
        FailureKind::Length
    }

    fn observe(
        &self,
        iterator: &mut I,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        iterator.next().map_or(Ok(()), Err)
    }

    fn explain(
        &self,
        item: Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let failure = failure
            .actual(render.borrowed_values::<T, _>(
                core::slice::from_ref(&item),
                RenderingOrder::PreserveIteration,
            ))
            .relation("is not empty");
        consumed_fact(failure, render, 1)
    }
}

/// Requires an iterator to yield at least one element, consuming at most one.
pub(crate) struct IsNotEmptyScan<T>(PhantomData<fn() -> T>);

impl<T> IsNotEmptyScan<T> {
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T, I, R> Scan<I, R> for IsNotEmptyScan<T>
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    type Rejection = ();

    fn kind(&self) -> FailureKind {
        FailureKind::Length
    }

    fn observe(&self, iterator: &mut I, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        iterator.next().map(drop).ok_or(())
    }

    fn explain(
        &self,
        (): (),
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let empty: &[I::Item] = &[];
        failure
            .actual(
                context
                    .render()
                    .borrowed_values::<T, _>(empty, RenderingOrder::PreserveIteration),
            )
            .relation("is unexpectedly empty")
    }
}

/// Why a length scan rejected its input.
pub(crate) enum LengthRejection<Item> {
    /// An exact size hint ruled out the expected length before consuming anything.
    Reported(KnownLength),
    /// Counting decided the rejection. The count is exact if the input ended, and a lower bound
    /// otherwise.
    Counted { tail: Tail<Item>, exact: bool },
}

/// Counts at most `expected + 1` elements. An exact size hint can only reject early. A hint that
/// agrees with the expected length is verified by counting, because hints are not trusted.
pub(crate) struct LengthScan<T> {
    expected: usize,
    item: PhantomData<fn() -> T>,
}

impl<T> LengthScan<T> {
    pub(crate) const fn new(expected: usize) -> Self {
        Self {
            expected,
            item: PhantomData,
        }
    }
}

impl<T, I, R> Scan<I, R> for LengthScan<T>
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    type Rejection = LengthRejection<I::Item>;

    fn kind(&self) -> FailureKind {
        FailureKind::Length
    }

    fn observe(
        &self,
        iterator: &mut I,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        if let Some(known) = KnownLength::mismatch(iterator, self.expected, LengthBound::Exact) {
            return Err(LengthRejection::Reported(known));
        }
        let mut tail = Tail::new(PREVIEW_CAPACITY);
        let mut exact = false;
        for _ in 0..=self.expected {
            let Some(item) = iterator.next() else {
                exact = true;
                break;
            };
            tail.push(item);
        }
        if exact && tail.consumed == self.expected {
            Ok(())
        } else {
            Err(LengthRejection::Counted {
                tail: tail.finish(),
                exact,
            })
        }
    }

    fn explain(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let failure = failure
            .relation("does not have the expected length")
            .expected(render.value(&self.expected));
        match rejection {
            LengthRejection::Reported(known) => known.reported_fact(failure, render),
            LengthRejection::Counted { tail, exact } => {
                let failure = failure
                    .actual(tail.rendered::<T, _>(render))
                    .fact(Fact::labelled(
                        if exact {
                            "Actual length"
                        } else {
                            "Minimum actual length"
                        },
                        render.value(&tail.consumed),
                    ));
                tail.omission(failure)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod length_scan {
        use indoc::formatdoc;

        use super::super::LengthScan;
        use crate::{
            assertions::iterator::run,
            failure::AssertionFailures,
            prelude::*,
            renderer::RenderedBody,
            test_support::{CustomValueRenderer, assert_custom_fact, assert_custom_value},
        };

        #[test]
        fn the_iterator_stays_alive_through_explanation_without_repeating_its_hint() {
            use core::{cell::Cell, fmt};

            struct ObservedIterator<'a> {
                hints: &'a Cell<usize>,
                drops: &'a Cell<usize>,
            }
            impl Iterator for ObservedIterator<'_> {
                type Item = i32;
                fn next(&mut self) -> Option<i32> {
                    panic!("the size hint is decisive")
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.hints.set(self.hints.get() + 1);
                    (2, Some(2))
                }
            }
            impl Drop for ObservedIterator<'_> {
                fn drop(&mut self) {
                    self.drops.set(self.drops.get() + 1);
                }
            }
            struct Renderer<'a>(&'a Cell<usize>);
            impl ValueRenderer<i32> for Renderer<'_> {
                fn fmt(&self, _: &i32, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                    panic!("no items were consumed")
                }
            }
            impl ValueRenderer<usize> for Renderer<'_> {
                fn fmt(&self, value: &usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    assert_that!(self.0.get()).is_equal_to(0);
                    fmt::Debug::fmt(value, f)
                }
            }

            let hints = Cell::new(0);
            let drops = Cell::new(0);
            let failures = assert_that!(())
                .with_renderer(Renderer(&drops))
                .capture(|it| {
                    let iterator = ObservedIterator {
                        hints: &hints,
                        drops: &drops,
                    };
                    run(&it, || (iterator, LengthScan::<i32>::new(3)));
                    it
                });
            assert_that!(hints.get()).is_equal_to(1);
            assert_that!(drops.get()).is_equal_to(1);
            assert_that!(failures).has_length(1);
        }

        fn length<I: Iterator<Item = i32>>(
            iterator: I,
            expected: usize,
            budget: RenderingBudget,
        ) -> AssertionFailures {
            assert_that!(())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .with_rendering_budget(budget)
                .capture(|it| {
                    run(&it, || (iterator, LengthScan::<i32>::new(expected)));
                    it
                })
        }
        #[test]
        fn reports_exact_size_hints_without_consuming_elements() {
            let failures = length([1, 2].into_iter(), 3, RenderingBudget::default());
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    let element = element.has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `()`

                does not have the expected length

                Expected: custom(3)

                Details:
                  - Reported length: custom(2)
                -------- assertr --------
            "});

                    assert_custom_value(element.actual().expected.as_ref().unwrap(), &3_usize);
                    assert_custom_fact(element.actual(), "Reported length", 2);
                    element.derive(|failure| &failure.actual).is_none();
                },
            ]);
        }
        #[test]
        fn reports_a_minimum_length_when_the_scan_stops_early() {
            let failures = length(
                [1, 2].into_iter().filter(|_| true),
                1,
                RenderingBudget::default(),
            );
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    let element = element.has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `()`

                Actual: [
                    custom(1),
                    custom(2),
                ]

                does not have the expected length

                Expected: custom(1)

                Details:
                  - Minimum actual length: custom(2)
                -------- assertr --------
            "});

                    assert_custom_fact(element.actual(), "Minimum actual length", 2);
                },
            ]);
        }
        #[test]
        fn limits_evidence_without_changing_consumption() {
            let mut iterator = [1, 2, 3].into_iter().filter(|_| true);
            let failures = length(
                &mut iterator,
                1,
                RenderingBudget::default().with_max_leaf_characters(3),
            );
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `()`

                Actual: [
                    cus... 6 more characters ...,
                    cus... 6 more characters ...,
                ]

                does not have the expected length

                Expected: cus... 6 more characters ...

                Details:
                  - Minimum actual length: cus... 6 more characters ...
                -------- assertr --------
            "});
                },
            ]);

            assert_that!(iterator.next()).is_equal_to(Some(3));
            let fact = failures[0]
                .facts
                .iter()
                .find(|fact| fact.label.as_deref() == Some("Minimum actual length"))
                .unwrap();
            assert_that!(fact.value.type_name).is_equal_to(Some("usize"));
            assert_that!(fact.value.body).is_equal_to(RenderedBody::Text {
                text: "cus".into(),
                omitted_characters: 6,
            });
        }

        /// Yields `remaining` items while reporting `hint` as its exact length.
        struct Lying<'a> {
            remaining: i32,
            hint: usize,
            next_calls: &'a core::cell::Cell<usize>,
        }

        impl Iterator for Lying<'_> {
            type Item = i32;
            fn next(&mut self) -> Option<i32> {
                self.next_calls.set(self.next_calls.get() + 1);
                (self.remaining > 0).then(|| {
                    self.remaining -= 1;
                    self.remaining
                })
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.hint, Some(self.hint))
            }
        }

        #[test]
        fn verifies_an_agreeing_size_hint_by_consuming_at_most_one_extra_element() {
            for (remaining, expected, next_calls, failed) in
                [(3, 0, 1, true), (3, 2, 3, true), (2, 2, 3, false)]
            {
                let calls = core::cell::Cell::new(0);
                let iterator = Lying {
                    remaining,
                    hint: expected,
                    next_calls: &calls,
                };
                let failures = length(iterator, expected, RenderingBudget::default());
                assert_that!(calls.get()).is_equal_to(next_calls);
                assert_that!(failures.len()).is_equal_to(usize::from(failed));
            }

            let calls = core::cell::Cell::new(0);
            let failures = length(
                Lying {
                    remaining: 3,
                    hint: 0,
                    next_calls: &calls,
                },
                0,
                RenderingBudget::default(),
            );
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `()`

                Actual: [
                    custom(2),
                ]

                does not have the expected length

                Expected: custom(0)

                Details:
                  - Minimum actual length: custom(1)
                -------- assertr --------
            "});
                },
            ]);
        }

        #[test]
        fn borrowed_length_and_emptiness_agree_on_a_lying_size_hint() {
            struct LyingSlice<'a>(core::slice::Iter<'a, i32>);
            impl<'a> Iterator for LyingSlice<'a> {
                type Item = &'a i32;
                fn next(&mut self) -> Option<&'a i32> {
                    self.0.next()
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    (0, Some(0))
                }
            }
            struct Source([i32; 3]);
            impl<'a> IntoIterator for &'a Source {
                type Item = &'a i32;
                type IntoIter = LyingSlice<'a>;
                fn into_iter(self) -> LyingSlice<'a> {
                    LyingSlice(self.0.iter())
                }
            }

            let failures = assert_that!(Source([1, 2, 3]))
                .with_location(false)
                .capture(|it| it.into_iter_has_length(0).into_iter_is_empty());
            assert_that!(failures).has_length(2);
        }
    }
}
