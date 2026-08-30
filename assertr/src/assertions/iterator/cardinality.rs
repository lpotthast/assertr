use super::{
    AssertThat, Borrow, CollectionStyle, Mode, Preview, String, Tail, ValueRenderer, Vec, Write,
    exact_size_hint, format, push_preview_details, writedoc,
};

#[track_caller]
pub(crate) fn assert_is_empty<S, T, I, M: Mode, R>(this: &AssertThat<'_, S, M, R>, mut iterator: I)
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    if let Some(item) = iterator.next() {
        let preview = Preview {
            items: alloc::vec![item],
            consumed: 1,
        };
        let mut details = Vec::new();
        push_preview_details(&mut details, &preview, Some(0));
        let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
    Actual: {actual:#?}

    is not empty.
"}
        });
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
        let preview: Vec<I::Item> = Vec::new();
        let actual = this.render_borrowed_values::<T, _>(&preview, CollectionStyle::List);
        this.fail(|w: &mut String| {
            writedoc! {w,r"
    Actual: {actual:#?}

    is empty.
"}
        });
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
        let details = alloc::vec![format!(
            "Iterator reported an exact remaining length of {actual}; no elements were consumed."
        )];
        let preview: Vec<I::Item> = Vec::new();
        let rendered = this.render_borrowed_values::<T, _>(&preview, CollectionStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {rendered:#?}

                does not have the correct length

                Expected: {expected}
                Observed: {actual}
            "}
        });
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
    let mut details = Vec::new();
    push_preview_details(&mut details, &preview, None);
    let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
    let observed = if preview.consumed > expected {
        format!("more than {expected}")
    } else {
        format!("{}", preview.consumed)
    };
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w,r"
        Actual: {actual:#?}

        does not have the correct length

        Expected: {expected}
        Observed: {observed}
    "}
    });
}
