//! Retain the smallest k candidates without collecting and sorting the entire input.
//!
//! Diagnostic budgets limit output groups, but sorting everything before truncating still retains
//! every rendered candidate temporarily. This module replaces that full-sort-and-truncate step.
//! For example, a limit of two selects `[1, 2]` from `[5, 2, 4, 1]`, regardless of encounter order.
//!
//! [`select_smallest`] is the rendering-facing operation: cache each candidate's comparison key,
//! select by that key, and return payloads in stable sorted order. [`Smallest`] supports
//! incremental callers, such as evidence scopes that retain children while evaluation is still
//! running. It stores ordered items directly. Wrap a payload in [`Keyed`] when only its key should
//! be ordered.
//!
//! Groups that fit the limit stay in a vector and need only one sort, avoiding heap maintenance.
//! The first offer beyond a finite limit turns that vector into a max-heap. Its root is the largest
//! retained item, so each subsequent offer either replaces that root or is discarded. This keeps at
//! most k retained items plus the incoming candidate and uses O(n log(k + 1)) comparisons,
//! excluding key construction and comparison costs. Storage grows as needed. Neither construction
//! nor heap conversion reserves k entries upfront.
//!
//! A zero limit retains nothing. The rendering helper also skips input traversal and key
//! construction. `usize::MAX` means unlimited collection followed by sorting. These are item
//! bounds, not byte bounds: a candidate may own arbitrarily large data. Callers still own
//! rendering, truth, group membership, and omission accounting.

use alloc::{collections::BinaryHeap, vec::Vec};
use core::cmp::Ordering;

/// Associates an owned payload with a comparison key, without requiring the payload to be `Ord`.
///
/// Equality and ordering deliberately ignore `value`. Include original identity in `key` when
/// distinct payloads with equal display text must have a stable tie order.
pub(crate) struct Keyed<K, T> {
    pub(crate) key: K,
    pub(crate) value: T,
}

impl<K: Ord, T> PartialEq for Keyed<K, T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<K: Ord, T> Eq for Keyed<K, T> {}

impl<K: Ord, T> PartialOrd for Keyed<K, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, T> Ord for Keyed<K, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

enum Storage<T> {
    Buffer(Vec<T>),
    // Always nonempty and at the finite item limit. Conversion occurs on the first overflow.
    Heap(BinaryHeap<T>),
}

/// The smallest `maximum` items offered so far, with no cloning or eager allocation.
///
/// Use [`Self::into_sorted`] for final presentation. Equal items have no promised order.
/// [`select_smallest`] supplies encounter ranks when stable ordering by a separate key is needed.
pub(crate) struct Smallest<T> {
    maximum: usize,
    storage: Storage<T>,
}

impl<T: Ord> Smallest<T> {
    /// Starts empty. Zero disables retention and `usize::MAX` removes the limit.
    pub(crate) fn new(maximum: usize) -> Self {
        Self {
            maximum,
            storage: Storage::Buffer(Vec::new()),
        }
    }

    /// The item limit this selection was created with.
    pub(crate) fn maximum(&self) -> usize {
        self.maximum
    }

    /// Number of retained items, excluding discarded or replaced offers.
    pub(crate) fn len(&self) -> usize {
        match &self.storage {
            Storage::Buffer(buffer) => buffer.len(),
            Storage::Heap(heap) => heap.len(),
        }
    }

    /// Keeps the item if there is room or it is smaller than the largest retained item.
    /// Discards the losing item immediately, including any evicted payload.
    pub(crate) fn offer(&mut self, item: T) {
        if self.maximum == 0 {
            return;
        }
        if let Storage::Buffer(buffer) = &mut self.storage
            && buffer.len() == self.maximum
            && self.maximum != usize::MAX
        {
            self.storage = Storage::Heap(BinaryHeap::from(core::mem::take(buffer)));
        }
        match &mut self.storage {
            Storage::Buffer(buffer) => buffer.push(item),
            Storage::Heap(heap) => {
                let mut largest = heap.peek_mut().expect("a full nonempty heap");
                if item < *largest {
                    *largest = item;
                }
            }
        }
    }

    /// Moves the retained items out in ascending order, reusing the existing storage.
    pub(crate) fn into_sorted(self) -> Vec<T> {
        match self.storage {
            Storage::Buffer(mut buffer) => {
                buffer.sort_unstable();
                buffer
            }
            Storage::Heap(heap) => heap.into_sorted_vec(),
        }
    }
}

impl<K: Ord, T> Smallest<Keyed<K, T>> {
    /// Moves payloads out in ascending key order and releases their cached keys.
    pub(crate) fn into_values(self) -> Vec<T> {
        self.into_sorted()
            .into_iter()
            .map(|item| item.value)
            .collect()
    }
}

/// Equivalent to stable sorting by `key` and taking the first `maximum` payloads.
///
/// Each key is computed once and cached with the item's encounter rank. Neither `Ord` nor `Clone`
/// is required on payloads. A nonzero limit consumes the full input, even for losing candidates.
/// Zero does not advance the input or call `key`. Unlimited selection retains every candidate.
pub(crate) fn select_smallest<T, K: Ord>(
    input: impl Iterator<Item = T>,
    maximum: usize,
    mut key: impl FnMut(&T) -> K,
) -> Vec<T> {
    let mut selection = Smallest::new(maximum);
    if maximum > 0 {
        for (rank, value) in input.enumerate() {
            selection.offer(Keyed {
                key: (key(&value), rank),
                value,
            });
        }
    }
    selection.into_values()
}

#[cfg(test)]
mod tests {
    use alloc::rc::Rc;
    use core::cell::Cell;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn matches_stable_full_sort_with_duplicates_at_every_limit() {
        for length in 0..9 {
            for seed in 0..32 {
                let input: Vec<_> = (0..length).map(|i| ((i * 7 + seed) % 5, i)).collect();
                let mut reference = input.clone();
                reference.sort_by_key(|&(key, _)| key);
                for limit in (0..=length + 1).chain([usize::MAX]) {
                    let selected = select_smallest(input.iter().copied(), limit, |&(key, _)| key);
                    assert_that!(selected).is_equal_to(&reference[..limit.min(length)]);
                }
            }
        }
    }

    #[test]
    fn zero_neither_advances_input_nor_computes_keys() {
        let input = core::iter::from_fn(|| -> Option<()> { panic!("advanced the input") });
        assert_that!(select_smallest(input, 0, |()| -> usize {
            panic!("computed a key")
        }))
        .is_empty();
    }

    #[test]
    fn buffers_until_overflow_without_reserving_the_limit() {
        for limit in [0, 2, usize::MAX] {
            let mut selection = Smallest::new(limit);
            let Storage::Buffer(buffer) = &selection.storage else {
                panic!("allocated a heap before overflow")
            };
            assert_that!(buffer.capacity()).is_equal_to(0);
            for key in [3, 2] {
                selection.offer(key);
                assert_that!(matches!(selection.storage, Storage::Buffer(_))).is_true();
            }
            selection.offer(9);
            assert_that!(matches!(selection.storage, Storage::Heap(_))).is_equal_to(limit == 2);
            selection.offer(1);
            let values = selection.into_sorted();
            let reference = [1, 2, 3, 9];
            assert_that!(values).is_equal_to(&reference[..limit.min(4)]);
        }
    }

    // Deliberately neither Clone nor Ord. Count payload roots independently of storage capacity.
    struct Live(Rc<Cell<usize>>);

    impl Live {
        fn new(live: &Rc<Cell<usize>>) -> Self {
            live.set(live.get() + 1);
            Self(Rc::clone(live))
        }
    }

    impl Drop for Live {
        fn drop(&mut self) {
            self.0.set(self.0.get() - 1);
        }
    }

    #[test]
    fn replacements_and_rejected_offers_keep_only_the_limit_plus_one_live_payloads() {
        for limit in [0, 1, 2, 17] {
            let live = Rc::new(Cell::new(0));
            let mut selection = Smallest::new(limit);
            for key in (0..100).rev().chain(0..100) {
                let payload = Live::new(&live);
                assert_that!(live.get()).is_less_or_equal_to(limit + 1);
                selection.offer(Keyed {
                    key,
                    value: payload,
                });
                assert_that!(live.get()).is_equal_to(selection.len());
                assert_that!(live.get()).is_less_or_equal_to(limit);
            }
            // Finalization moves the payloads without cloning them or requiring payload ordering.
            let values = selection.into_values();
            assert_that!(live.get()).is_equal_to(values.len());
            drop(values);
            assert_that!(live.get()).is_equal_to(0);
        }
    }
}
