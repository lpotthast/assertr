//! Allocation regressions live in their own executable so the library can forbid unsafe code.

use assertr::{
    assertions::{collection, map},
    prelude::*,
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    collections::BTreeMap,
    hint::black_box,
};

struct CountingAllocator;

thread_local! {
    static ALLOCATED: Cell<Option<usize>> = const { Cell::new(None) };
}

fn record(bytes: usize) {
    let _ = ALLOCATED.try_with(|count| {
        if let Some(current) = count.get() {
            count.set(Some(current + bytes));
        }
    });
}

// SAFETY: Every allocation operation is forwarded unchanged to the system allocator.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(size);
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn allocated(f: impl FnOnce()) -> usize {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ALLOCATED.set(None);
        }
    }
    let _reset = Reset;
    ALLOCATED.set(Some(0));
    f();
    ALLOCATED.replace(None).unwrap()
}

#[test]
fn million_element_prefix_allocates_nothing() {
    let values = vec![1_u64; 1_000_000];
    let bytes = allocated(|| {
        black_box(assert_that!(black_box(&values)).starts_with(black_box(&values)));
    });
    assert_that!(bytes).is_equal_to(0);
}

#[test]
fn successful_collections_and_keys_use_existing_expected_storage() {
    let values = [1, 2, 3, 4];
    let keys = BTreeMap::from(values.map(|key| (key, key)));
    for reusable in [false, true] {
        for method in 0..5 {
            let bytes = allocated(|| {
                let it = assert_that!(black_box(&values));
                if reusable {
                    match method {
                        0 => {
                            black_box(it.matches(collection::StartsWith::new(black_box(&values))));
                        }
                        1 => {
                            black_box(it.matches(collection::EndsWith::new(black_box(&values))));
                        }
                        2 => {
                            black_box(
                                it.matches(collection::ContainsExactly::new(black_box(&values))),
                            );
                        }
                        3 => {
                            black_box(it.matches(collection::ContainsAll::new(black_box(&values))));
                        }
                        _ => {
                            black_box(
                                assert_that!(black_box(&keys))
                                    .matches(map::ContainsKeys::new(black_box(&values))),
                            );
                        }
                    }
                } else {
                    match method {
                        0 => {
                            black_box(it.starts_with(black_box(&values)));
                        }
                        1 => {
                            black_box(it.ends_with(black_box(&values)));
                        }
                        2 => {
                            black_box(it.contains_exactly(black_box(&values)));
                        }
                        3 => {
                            black_box(it.contains_all(black_box(&values)));
                        }
                        _ => {
                            black_box(
                                assert_that!(black_box(&keys)).contains_keys(black_box(&values)),
                            );
                        }
                    }
                }
            });
            assert_that!(bytes).is_equal_to(0);
        }
    }
}

#[test]
fn failing_million_element_prefix_renders_without_materializing_expected_views() {
    let actual = vec![1_u64; 1_000_000];
    let expected = vec![2_u64; 1_000_000];
    let mut failures = None;
    let bytes = allocated(|| {
        failures = Some(
            assert_that!(black_box(&actual))
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .capture(|it| it.starts_with(black_box(&expected))),
        );
    });
    assert_that!(failures.unwrap()).has_length(1);
    assert_that!(bytes).is_less_than(64 * 1024);
}

#[test]
fn iterator_prefix_and_exact_allocation_is_constant_beyond_preview_capacity() {
    let small = vec![1_u64; 32];
    let large = vec![1_u64; 4096];
    for exact in [false, true] {
        let measure = |values: &Vec<u64>| {
            allocated(|| {
                let iterator = black_box(values.iter().copied());
                let it = assert_that_owned!(iterator);
                if exact {
                    black_box(it.contains_exactly(black_box(values)));
                } else {
                    black_box(it.starts_with(black_box(values)));
                }
            })
        };
        assert_that!(measure(&large)).is_equal_to(measure(&small));
    }
}

mod working_storage {
    use super::*;
    use assertr::borrow_for::BorrowFor;
    use std::borrow::Borrow;

    #[derive(Clone, Copy, Debug)]
    struct Item(u64);
    impl PartialEq<u64> for Item {
        fn eq(&self, expected: &u64) -> bool {
            self.0 == *expected
        }
    }
    impl PartialEq<[u64]> for Item {
        fn eq(&self, expected: &[u64]) -> bool {
            expected == [self.0]
        }
    }
    struct Thin(u64);
    impl Borrow<u64> for Thin {
        fn borrow(&self) -> &u64 {
            &self.0
        }
    }
    impl BorrowFor<Item> for Thin {
        type View = u64;
    }
    struct Wide([u64; 1]);
    impl Borrow<[u64]> for Wide {
        fn borrow(&self) -> &[u64] {
            &self.0
        }
    }
    impl BorrowFor<Item> for Wide {
        type View = [u64];
    }

    fn measure<E: BorrowFor<Item>>(actual: &[Item; 128], expected: &[E], method: usize) -> usize
    where
        Item: PartialEq<E::View>,
        DebugRenderer: ValueRenderer<E::View>,
    {
        allocated(|| match method {
            0 => {
                black_box(
                    StableOrderAssertions::<Item, DebugRenderer>::contains_contiguous(
                        assert_that!(actual),
                        black_box(expected),
                    ),
                );
            }
            1 => {
                black_box(
                    CollectionAssertions::<Item, DebugRenderer>::contains_exactly_in_any_order(
                        assert_that!(actual),
                        black_box(expected),
                    ),
                );
            }
            2 => {
                black_box(IteratorAssertions::<Item, _, DebugRenderer>::ends_with(
                    assert_that_owned!(actual.iter().copied()),
                    black_box(expected),
                ));
            }
            3 => {
                black_box(
                    IteratorAssertions::<Item, _, DebugRenderer>::contains_contiguous(
                        assert_that_owned!(actual.iter().copied()),
                        black_box(expected),
                    ),
                );
            }
            4 => {
                black_box(
                    IteratorAssertions::<Item, _, DebugRenderer>::contains_exactly_in_any_order(
                        assert_that_owned!(actual.iter().copied()),
                        black_box(expected),
                    ),
                );
            }
            5 => {
                black_box(
                    IntoIteratorAssertions::<Item, DebugRenderer>::into_iter_contains_all(
                        assert_that!(actual),
                        black_box(expected),
                    ),
                );
            }
            6 => {
                black_box(
                    assert_that!(actual)
                        .matches(collection::ContainsContiguous::new(black_box(expected))),
                );
            }
            _ => {
                black_box(assert_that!(actual).matches(
                    collection::ContainsExactlyInAnyOrder::new(black_box(expected)),
                ));
            }
        })
    }

    #[test]
    fn required_working_storage_is_independent_of_expected_view_width() {
        let actual = [Item(1); 128];
        let thin = std::array::from_fn::<_, 128, _>(|_| Thin(1));
        let wide = std::array::from_fn::<_, 128, _>(|_| Wide([1]));
        // A full vector of selected views costs more for wide references. Equal allocation
        // totals catch that buffer while permitting the algorithms' necessary working storage.
        for method in 0..8 {
            let thin_bytes = measure(&actual, &thin, method);
            let wide_bytes = measure(&actual, &wide, method);
            assert_that!(wide_bytes)
                .is_equal_to(thin_bytes)
                .is_greater_than(0);
        }
    }

    #[test]
    fn exact_map_working_storage_is_independent_of_expected_view_width() {
        let actual = (0..128)
            .map(|key| (key, Item(1)))
            .collect::<BTreeMap<_, _>>();
        let thin = (0..128).map(|key| (key, Thin(1))).collect::<Vec<_>>();
        let wide = (0..128).map(|key| (key, Wide([1]))).collect::<Vec<_>>();
        for reusable in [false, true] {
            let thin_bytes = allocated(|| {
                if reusable {
                    black_box(
                        assert_that!(black_box(&actual))
                            .matches(map::ContainsExactlyEntries::new(black_box(&thin))),
                    );
                } else {
                    black_box(
                        assert_that!(black_box(&actual)).contains_exactly_entries(black_box(&thin)),
                    );
                }
            });
            let wide_bytes = allocated(|| {
                if reusable {
                    black_box(
                        assert_that!(black_box(&actual))
                            .matches(map::ContainsExactlyEntries::new(black_box(&wide))),
                    );
                } else {
                    black_box(
                        assert_that!(black_box(&actual)).contains_exactly_entries(black_box(&wide)),
                    );
                }
            });
            assert_that!(wide_bytes)
                .is_equal_to(thin_bytes)
                .is_greater_than(0);
        }
    }
}
