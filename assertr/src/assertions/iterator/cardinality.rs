use super::{
    AssertThat, Borrow, FailureKind, Mode, PositionReporting, Preview, Tail, ValueRenderer, Vec,
    exact_size_hint,
};
use crate::Fact;

#[track_caller]
pub(crate) fn assert_is_empty<S, T, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    if let Some(item) = iterator.next() {
        let preview = Preview {
            items: alloc::vec![item],
            consumed: 1,
        };
        let failure = this
            .failure(FailureKind::Length)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("is not empty");
        preview
            .facts(failure, this.render(), positions.index(0))
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_is_not_empty<S, T, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    if iterator.next().is_none() {
        let preview: Preview<I::Item> = Preview {
            items: Vec::new(),
            consumed: 0,
        };
        this.failure(FailureKind::Length)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("is unexpectedly empty")
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_has_length<S, T, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
    expected: usize,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    if let Some(actual) = exact_size_hint(&iterator) {
        if actual == expected {
            return;
        }
        let preview: Preview<I::Item> = Preview {
            items: Vec::new(),
            consumed: 0,
        };
        let failure = this
            .failure(FailureKind::Length)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not have the expected length")
            .expected(this.render().value(&expected));
        preview
            .facts(failure, this.render(), None)
            .fact(Fact::labelled(
                "Actual length",
                this.render().value(&actual),
            ))
            .raise();
        return;
    }
    let mut tail = Tail::new();
    let mut exhausted = false;
    for _ in 0..=expected {
        let Some(item) = iterator.next() else {
            exhausted = true;
            break;
        };
        tail.push(item);
    }
    if tail.consumed == expected && exhausted {
        return;
    }
    let preview = tail.finish();
    let failure = this
        .failure(FailureKind::Length)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation("does not have the expected length")
        .expected(this.render().value(&expected));
    let failure = preview.facts(failure, this.render(), None);
    if preview.consumed > expected {
        failure.fact(Fact::labelled(
            "Minimum actual length",
            this.render().value(&preview.consumed),
        ))
    } else {
        failure.fact(Fact::labelled(
            "Actual length",
            this.render().value(&preview.consumed),
        ))
    }
    .raise();
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
                RenderingBudget::builder().max_leaf_characters(3).build(),
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

            assert_eq!(iterator.next(), Some(3));
            let fact = failures[0]
                .facts
                .iter()
                .find(|fact| fact.label == "Minimum actual length")
                .unwrap();
            assert_eq!(fact.value.type_name, Some("usize"));
            assert_eq!(
                fact.value.body,
                RenderedBody::Text {
                    text: "cus".into(),
                    omitted_characters: 6
                }
            );
        }
    }
}
