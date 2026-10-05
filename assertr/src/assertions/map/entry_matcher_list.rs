use super::entry::{explain_entry, record_entry};
use crate::borrow_for::{BorrowFor, borrow_for};
use crate::{
    __private::{Cons, Nil},
    AssertThat, AssertionContext, AssertionFailure, DebugRenderer, ExpectationDiagnostics,
    ValueRenderer,
    assertions::map::{Entry, Map, MapLookup, entry},
    expectation::{MatcherList, lists::sealed as list_sealed, satisfying},
    failure::{FailureBuilder, FailureKind},
    mode::Capture,
};
use alloc::vec::Vec;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Supported keyed matcher lists. Use `entries_are!` for heterogeneous entries, or arrays,
/// slices, and vectors of [`Entry`] values for homogeneous entries.
pub trait EntryMatcherList<MapType: Map + ?Sized, R = DebugRenderer>:
    MatcherList<MapType, R> + sealed::Sealed
{
    /// Evaluates one entry and retains its evidence, returning truth and the original stored key.
    /// A present key is returned even when its value rejects. The index must be less than `len()`.
    fn evaluate_entry_at<'a>(
        &'a self,
        index: usize,
        actual: &'a MapType,
        context: &mut AssertionContext<'_, R>,
    ) -> (bool, Option<&'a MapType::Key>);
}

impl sealed::Sealed for Nil {}
impl<MapType: Map + ?Sized, R> EntryMatcherList<MapType, R> for Nil {
    fn evaluate_entry_at<'a>(
        &'a self,
        _: usize,
        _: &'a MapType,
        _: &mut AssertionContext<'_, R>,
    ) -> (bool, Option<&'a MapType::Key>) {
        panic!("empty entry list")
    }
}

impl<K, M, T> sealed::Sealed for Cons<Entry<K, M>, T> {}
impl<MapType, StoredKey, R, K, M, T> EntryMatcherList<MapType, R> for Cons<Entry<K, M>, T>
where
    MapType: Map<Key = StoredKey> + MapLookup<K::View> + ?Sized,
    K: BorrowFor<StoredKey>,
    M: ExpectationDiagnostics<MapType::Value, R>,
    R: ValueRenderer<K::View>,
    T: EntryMatcherList<MapType, R>,
{
    fn evaluate_entry_at<'a>(
        &'a self,
        index: usize,
        actual: &'a MapType,
        context: &mut AssertionContext<'_, R>,
    ) -> (bool, Option<&'a MapType::Key>) {
        if index == 0 {
            self.0.evaluate_and_record(actual, context)
        } else {
            self.1.evaluate_entry_at(index - 1, actual, context)
        }
    }
}

macro_rules! homogeneous {
    ($type:ty $(, $size:ident)?) => {
        impl<K, M $(, const $size: usize)?> sealed::Sealed for $type {}

        impl<MapType, StoredKey, R, K, M $(, const $size: usize)?> EntryMatcherList<MapType, R> for $type
        where
            MapType: Map<Key = StoredKey> + MapLookup<K::View> + ?Sized,
            K: BorrowFor<StoredKey>,
            M: ExpectationDiagnostics<MapType::Value, R>,
            R: ValueRenderer<K::View>,
        {
            fn evaluate_entry_at<'a>(
                &'a self,
                index: usize,
                actual: &'a MapType,
                context: &mut AssertionContext<'_, R>,
            ) -> (bool, Option<&'a MapType::Key>) {
                self[index].evaluate_and_record(actual, context)
            }
        }
    };
}

homogeneous!([Entry<K, M>]);
homogeneous!(Vec<Entry<K, M>>);
homogeneous!([Entry<K, M>; N], N);

impl<L: ?Sized> sealed::Sealed for &L {}
impl<MapType: Map + ?Sized, R, L: EntryMatcherList<MapType, R> + ?Sized>
    EntryMatcherList<MapType, R> for &L
{
    fn evaluate_entry_at<'a>(
        &'a self,
        index: usize,
        actual: &'a MapType,
        context: &mut AssertionContext<'_, R>,
    ) -> (bool, Option<&'a MapType::Key>) {
        (**self).evaluate_entry_at(index, actual, context)
    }
}

/// Builds a homogeneous keyed list from key/matcher pairs.
pub fn entry_matchers<K, M>(entries: impl IntoIterator<Item = (K, M)>) -> Vec<Entry<K, M>> {
    entries
        .into_iter()
        .map(|(key, matcher)| entry(key, matcher))
        .collect()
}

/// A borrowed list of key/callback pairs that adapts only the entry being evaluated or described.
pub(crate) struct SatisfyingEntryList<'a, K, F>(pub(crate) &'a [(K, F)]);

impl<K, F> sealed::Sealed for SatisfyingEntryList<'_, K, F> {}

impl<K, F> list_sealed::Sealed for SatisfyingEntryList<'_, K, F> {}

impl<MapType, StoredKey, R, K, F> MatcherList<MapType, R> for SatisfyingEntryList<'_, K, F>
where
    MapType: Map<Key = StoredKey> + MapLookup<K::View> + ?Sized,
    K: BorrowFor<StoredKey>,
    F: for<'a> Fn(AssertThat<'a, MapType::Value, Capture, R>),
    R: ValueRenderer<K::View> + Clone,
{
    fn len(&self) -> usize {
        self.0.len()
    }

    fn describe_at(&self, index: usize, context: &AssertionContext<'_, R>) -> AssertionFailure {
        let (key, assertions) = &self.0[index];
        explain_entry::<MapType::Value, _, _, _, _>(
            borrow_for::<StoredKey, _>(key),
            &satisfying(assertions),
            None,
            FailureBuilder::detached::<MapType>(FailureKind::Matching),
            context,
        )
        .build()
    }

    fn evaluate_at(
        &self,
        index: usize,
        actual: &MapType,
        context: &mut AssertionContext<'_, R>,
    ) -> bool {
        self.evaluate_entry_at(index, actual, context).0
    }
}

impl<MapType, StoredKey, R, K, F> EntryMatcherList<MapType, R> for SatisfyingEntryList<'_, K, F>
where
    MapType: Map<Key = StoredKey> + MapLookup<K::View> + ?Sized,
    K: BorrowFor<StoredKey>,
    F: for<'a> Fn(AssertThat<'a, MapType::Value, Capture, R>),
    R: ValueRenderer<K::View> + Clone,
{
    fn evaluate_entry_at<'a>(
        &'a self,
        index: usize,
        actual: &'a MapType,
        context: &mut AssertionContext<'_, R>,
    ) -> (bool, Option<&'a MapType::Key>) {
        let (key, assertions) = &self.0[index];
        record_entry(key, &satisfying(assertions), actual, context)
    }
}
