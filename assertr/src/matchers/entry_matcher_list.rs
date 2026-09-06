use super::{
    AssertrMatcher, Entry, MatcherList, entry,
    lists::{Cons, Nil},
};
use crate::{
    ValueRenderer,
    assertions::map::{Map, MapKeyQuery, MapLookup},
};
use alloc::vec::Vec;

pub(super) mod sealed {
    pub trait Sealed {}
}

/// Supported keyed matcher lists. Construct them with `entries_are!`.
pub trait EntryMatcherList<MapType, R>: MatcherList<MapType, R> + sealed::Sealed
where
    MapType: Map + ?Sized,
{
    /// The stored key found by one expected query, if it exists.
    fn found_key<'a>(&self, index: usize, actual: &'a MapType) -> Option<&'a MapType::Key>;
}

impl sealed::Sealed for Nil {}

impl<MapType, R> EntryMatcherList<MapType, R> for Nil
where
    MapType: Map + ?Sized,
{
    fn found_key<'a>(&self, _: usize, _: &'a MapType) -> Option<&'a MapType::Key> {
        None
    }
}

impl<K, M, T> sealed::Sealed for Cons<Entry<K, M>, T> {}

impl<MapType, R, K, M, T> EntryMatcherList<MapType, R> for Cons<Entry<K, M>, T>
where
    MapType: Map + MapLookup<<K as MapKeyQuery<<MapType as Map>::Key>>::Query> + ?Sized,
    K: MapKeyQuery<MapType::Key>,
    M: AssertrMatcher<MapType::Value, R>,
    R: ValueRenderer<K>,
    T: EntryMatcherList<MapType, R>,
{
    fn found_key<'a>(&self, index: usize, actual: &'a MapType) -> Option<&'a MapType::Key> {
        if index == 0 {
            actual.get_key_value(self.0.key.as_query()).map(|(k, _)| k)
        } else {
            self.1.found_key(index - 1, actual)
        }
    }
}

impl<K, M> sealed::Sealed for Vec<Entry<K, M>> {}

impl<MapType, R, K, M> EntryMatcherList<MapType, R> for Vec<Entry<K, M>>
where
    MapType: Map + MapLookup<<K as MapKeyQuery<<MapType as Map>::Key>>::Query> + ?Sized,
    K: MapKeyQuery<MapType::Key>,
    M: AssertrMatcher<MapType::Value, R>,
    R: ValueRenderer<K>,
{
    fn found_key<'a>(&self, index: usize, actual: &'a MapType) -> Option<&'a MapType::Key> {
        actual
            .get_key_value(self[index].key.as_query())
            .map(|(k, _)| k)
    }
}

impl<L: ?Sized> sealed::Sealed for &L {}

impl<MapType, R, L> EntryMatcherList<MapType, R> for &L
where
    MapType: Map + ?Sized,
    L: EntryMatcherList<MapType, R> + ?Sized,
{
    fn found_key<'a>(&self, index: usize, actual: &'a MapType) -> Option<&'a MapType::Key> {
        (**self).found_key(index, actual)
    }
}

/// Builds a homogeneous keyed list from key/matcher pairs.
pub fn entry_matchers<K, M>(entries: impl IntoIterator<Item = (K, M)>) -> Vec<Entry<K, M>> {
    entries
        .into_iter()
        .map(|(key, matcher)| entry(key, matcher))
        .collect()
}
