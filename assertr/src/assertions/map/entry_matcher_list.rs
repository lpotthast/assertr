use crate::{
    __private::{Cons, Nil},
    AssertionContext, ExpectationDiagnostics, ValueRenderer,
    assertions::map::{Entry, Map, MapKeyQuery, MapLookup, entry},
    expectation::MatcherList,
};
use alloc::vec::Vec;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Supported keyed matcher lists. Use `entries_are!` for heterogeneous entries, or arrays,
/// slices, and vectors of [`Entry`] values for homogeneous entries.
pub trait EntryMatcherList<MapType: Map + ?Sized, R>:
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
impl<MapType, R, K, M, T> EntryMatcherList<MapType, R> for Cons<Entry<K, M>, T>
where
    MapType: Map + MapLookup<<K as MapKeyQuery<<MapType as Map>::Key>>::Query> + ?Sized,
    K: MapKeyQuery<MapType::Key>,
    M: ExpectationDiagnostics<MapType::Value, R>,
    R: ValueRenderer<K>,
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

        impl<MapType, R, K, M $(, const $size: usize)?> EntryMatcherList<MapType, R> for $type
        where
            MapType: Map + MapLookup<<K as MapKeyQuery<<MapType as Map>::Key>>::Query> + ?Sized,
            K: MapKeyQuery<MapType::Key>,
            M: ExpectationDiagnostics<MapType::Value, R>,
            R: ValueRenderer<K>,
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
