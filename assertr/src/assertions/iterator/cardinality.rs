use super::{
    AssertThat, Borrow, FailureKind, Mode, PositionReporting, Preview, Tail, ValueRenderer, Vec,
    exact_size_hint,
};

#[track_caller]
pub(crate) fn assert_is_empty<S, T, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T>,
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
        preview.facts(failure, positions.index(0)).raise();
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
    R: ValueRenderer<T>,
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
            .expected(expected);
        preview
            .facts(failure, None)
            .fact("Actual length", actual)
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
        .expected(expected);
    let failure = preview.facts(failure, None);
    if preview.consumed > expected {
        failure.fact("Actual length", format_args!("more than {expected}"))
    } else {
        failure.fact("Actual length", preview.consumed)
    }
    .raise();
}
