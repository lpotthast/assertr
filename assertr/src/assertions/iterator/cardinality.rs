use super::{
    AssertThat, AssertionContext, Borrow, EqualToRef, Expectation, FailureBuilder, FailureKind,
    GroupStyle, Mode, PhantomData, PositionReporting, Preview, Scan, Tail, ValueRenderer, Vec,
    exact_size_hint, execute,
};
use crate::Fact;

struct IsEmpty<T> {
    positions: PositionReporting,
    item: PhantomData<fn() -> T>,
}

impl<T, I, R> Scan<I, R> for IsEmpty<T>
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    type Rejection = I::Item;
    const KIND: FailureKind = FailureKind::Length;

    fn observe(
        &self,
        iterator: &mut I,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        match iterator.next() {
            Some(item) => Err(item),
            None => Ok(()),
        }
    }

    fn explain<Target>(
        &self,
        item: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = failure
            .actual(render.borrowed_values::<T, _>(core::slice::from_ref(&item), GroupStyle::List))
            .relation("is not empty")
            .fact(Fact::labelled("Consumed elements", render.value(&1_usize)));
        match self.positions.index(0) {
            Some(index) => failure.fact(Fact::labelled("Decisive index", index)),
            None => failure,
        }
    }
}

struct IsNotEmpty<T>(PhantomData<fn() -> T>);

impl<T, I, R> Scan<I, R> for IsNotEmpty<T>
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    type Rejection = ();
    const KIND: FailureKind = FailureKind::Length;

    fn observe(&self, iterator: &mut I, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if iterator.next().is_some() {
            Ok(())
        } else {
            Err(())
        }
    }

    fn explain<Target>(
        &self,
        (): (),
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let empty: &[I::Item] = &[];
        failure
            .actual(
                context
                    .render()
                    .borrowed_values::<T, _>(empty, GroupStyle::List),
            )
            .relation("is unexpectedly empty")
    }
}

struct LengthObservation<Item> {
    preview: Preview<Item>,
    length: usize,
    exact: bool,
}

struct HasLength<T> {
    expected: usize,
    item: PhantomData<fn() -> T>,
}

impl<T, I, R> Scan<I, R> for HasLength<T>
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    type Rejection = LengthObservation<I::Item>;
    const KIND: FailureKind = FailureKind::Length;

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let observation = observe_length(iterator, self.expected);
        if observation.exact
            && EqualToRef(&self.expected)
                .evaluate(&observation.length, context)
                .is_ok()
        {
            Ok(())
        } else {
            Err(observation)
        }
    }

    fn explain<Target>(
        &self,
        actual: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = failure
            .expected(render.value(&self.expected))
            .actual(render.borrowed_values::<T, _>(&actual.preview.items, GroupStyle::List))
            .relation("does not have the expected length");
        actual
            .preview
            .facts(failure, render, None)
            .fact(Fact::labelled(
                if actual.exact {
                    "Actual length"
                } else {
                    "Minimum actual length"
                },
                render.value(&actual.length),
            ))
    }
}

#[track_caller]
pub(crate) fn assert_is_empty<S, T, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &IsEmpty::<T> {
            positions,
            item: PhantomData,
        },
    );
}

#[track_caller]
pub(crate) fn assert_is_not_empty<S, T, I, M: Mode, R>(this: &AssertThat<'_, S, M, R>, iterator: I)
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    execute(this, iterator, &IsNotEmpty::<T>(PhantomData));
}

fn observe_length<I: Iterator>(iterator: &mut I, expected: usize) -> LengthObservation<I::Item> {
    if let Some(length) = exact_size_hint(&iterator) {
        return LengthObservation {
            preview: Preview {
                items: Vec::new(),
                consumed: 0,
            },
            length,
            exact: true,
        };
    }
    let mut tail = Tail::new();
    let mut exact = false;
    for _ in 0..=expected {
        let Some(item) = iterator.next() else {
            exact = true;
            break;
        };
        tail.push(item);
    }
    LengthObservation {
        length: tail.consumed,
        preview: tail.finish(),
        exact,
    }
}

#[track_caller]
pub(crate) fn assert_has_length<S, T, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: usize,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &HasLength::<T> {
            expected,
            item: PhantomData,
        },
    );
}

#[cfg(test)]
mod tests {
    mod assert_has_length {
        use super::super::assert_has_length;
        use crate::{
            AssertionFailures,
            prelude::*,
            renderer::RenderedBody,
            test_support::{CustomValueRenderer, assert_custom_fact, assert_custom_value},
        };
        use indoc::formatdoc;

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
                    it.track_assertion();
                    assert_has_length(
                        &it,
                        ObservedIterator {
                            hints: &hints,
                            drops: &drops,
                        },
                        3,
                    );
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
                    it.track_assertion();
                    assert_has_length(&it, iterator, expected);
                    it
                })
        }
        #[test]
        fn reports_exact_size_hints_without_consuming_elements() {
            let failures = length([1, 2].into_iter(), 3, RenderingBudget::default());
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `()`

                Actual: []

                does not have the expected length

                Expected: custom(3)

                Details:
                  - Consumed elements: custom(0)
                  - Actual length: custom(2)
                -------- assertr --------
            "});

                    assert_custom_value(element.actual().expected.as_ref().unwrap(), &3_usize);
                    assert_custom_fact(element.actual(), "Actual length", 2);
                    assert_custom_fact(element.actual(), "Consumed elements", 0);
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
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `()`

                Actual: [
                    custom(1),
                    custom(2),
                ]

                does not have the expected length

                Expected: custom(1)

                Details:
                  - Consumed elements: custom(2)
                  - Minimum actual length: custom(2)
                -------- assertr --------
            "});

                    assert_custom_fact(element.actual(), "Minimum actual length", 2);
                    assert_custom_fact(element.actual(), "Consumed elements", 2);
                },
            ]);
        }
        #[test]
        fn limits_evidence_without_changing_consumption() {
            use indoc::formatdoc;

            let mut iterator = [1, 2, 3].into_iter().filter(|_| true);
            let failures = length(
                &mut iterator,
                1,
                RenderingBudget::default().with_max_leaf_characters(3),
            );
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `()`

                Actual: [
                    cus... 6 more characters ...,
                    cus... 6 more characters ...,
                ]

                does not have the expected length

                Expected: cus... 6 more characters ...

                Details:
                  - Consumed elements: cus... 6 more characters ...
                  - Minimum actual length: cus... 6 more characters ...
                -------- assertr --------
            "});
                },
            ]);

            assert_that!(iterator.next()).is_equal_to(Some(3));
            let fact = failures[0]
                .facts
                .iter()
                .find(|fact| fact.label == "Minimum actual length")
                .unwrap();
            assert_that!(fact.value.type_name).is_equal_to(Some("usize"));
            assert_that!(fact.value.body).is_equal_to(RenderedBody::Text {
                text: "cus".into(),
                omitted_characters: 6,
            });
        }
    }
}
