use alloc::vec::Vec;
use core::marker::PhantomData;

use super::entry::{explain_entry, record_entry};
use crate::{
    __private::{Cons, Nil},
    AssertThat,
    assertions::map::{Entry, Map, MapLookup, entry},
    borrow_for::{BorrowFor, borrow_for},
    expectation::{AssertionContext, Expectation, lists::sealed as list_sealed},
    failure::{AssertionFailure, FailureBuilder, FailureKind},
    matchers::{MatcherList, satisfying},
    mode::Capture,
    renderer::{DebugRenderer, ValueRenderer},
};

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Supported keyed matcher lists. Use a `matchers!` list of [`entry`] values for heterogeneous
/// entries, or arrays, slices, and vectors of [`Entry`] values for homogeneous entries.
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
    M: Expectation<MapType::Value, R>,
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
            record_entry(&self.0.key, &self.0.matcher, actual, context)
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
            M: Expectation<MapType::Value, R>,
            R: ValueRenderer<K::View>,
        {
            fn evaluate_entry_at<'a>(
                &'a self,
                index: usize,
                actual: &'a MapType,
                context: &mut AssertionContext<'_, R>,
            ) -> (bool, Option<&'a MapType::Key>) {
                let entry = &self[index];
                record_entry(&entry.key, &entry.matcher, actual, context)
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
pub(crate) struct SatisfyingEntryList<L, K, F>(L, PhantomData<fn() -> (K, F)>);

impl<L: AsRef<[(K, F)]>, K, F> SatisfyingEntryList<L, K, F> {
    /// Stores the callbacks without accessing them, so tracking precedes the first borrow.
    pub(crate) fn new(callbacks: L) -> Self {
        Self(callbacks, PhantomData)
    }
}

impl<L, K, F> sealed::Sealed for SatisfyingEntryList<L, K, F> {}

impl<L, K, F> list_sealed::Sealed for SatisfyingEntryList<L, K, F> {}

impl<MapType, StoredKey, R, L, K, F> MatcherList<MapType, R> for SatisfyingEntryList<L, K, F>
where
    L: AsRef<[(K, F)]>,
    MapType: Map<Key = StoredKey> + MapLookup<K::View> + ?Sized,
    K: BorrowFor<StoredKey>,
    F: for<'a> Fn(AssertThat<'a, MapType::Value, Capture, R>),
    R: ValueRenderer<K::View> + Clone,
{
    fn len(&self) -> usize {
        self.0.as_ref().len()
    }

    fn describe_at(&self, index: usize, context: &AssertionContext<'_, R>) -> AssertionFailure {
        let (key, assertions) = &self.0.as_ref()[index];
        explain_entry::<MapType::Value, _, _, _>(
            || borrow_for::<StoredKey, _>(key),
            &satisfying(assertions),
            None,
            FailureBuilder::new::<MapType>(FailureKind::Matching),
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

impl<MapType, StoredKey, R, L, K, F> EntryMatcherList<MapType, R> for SatisfyingEntryList<L, K, F>
where
    L: AsRef<[(K, F)]>,
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
        let (key, assertions) = &self.0.as_ref()[index];
        record_entry(key, &satisfying(assertions), actual, context)
    }
}
