//! Shared streaming implementation for direct and borrowed iterator assertions.
//!
//! Per-failure diagnostics are accumulated in local `Vec<String>` buffers and handed to
//! [`AssertThat::fail_with_details`], never stored on the assertion itself.

use alloc::{collections::VecDeque, format, string::String, vec::Vec};
use core::{borrow::Borrow, fmt::Write};
use indoc::writedoc;

use crate::{
    AssertThat, AssertrPartialEq, Mode, ValueRenderer, mode::Capture, util::failure::join_failures,
    util::matching::match_bipartite,
};

const PREVIEW_CAPACITY: usize = 16;

struct Preview<Item> {
    items: Vec<Item>,
    consumed: usize,
}

impl<Item> Preview<Item> {
    fn omitted(&self) -> usize {
        self.consumed.saturating_sub(self.items.len())
    }

    fn start_index(&self) -> usize {
        self.omitted()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PositionReporting {
    YieldOrder,
    Unavailable,
}

impl PositionReporting {
    const fn index(self, index: usize) -> Option<usize> {
        match self {
            Self::YieldOrder => Some(index),
            Self::Unavailable => None,
        }
    }
}

struct Tail<Item> {
    items: VecDeque<Item>,
    consumed: usize,
}

impl<Item> Tail<Item> {
    fn new() -> Self {
        Self {
            items: VecDeque::new(),
            consumed: 0,
        }
    }
    fn push(&mut self, item: Item) {
        self.consumed += 1;
        if self.items.len() == PREVIEW_CAPACITY {
            let _ = self.items.pop_front();
        }
        self.items.push_back(item);
    }
    fn finish(self) -> Preview<Item> {
        Preview {
            items: self.items.into_iter().collect(),
            consumed: self.consumed,
        }
    }
}

fn exact_size_hint<I: Iterator>(iterator: &I) -> Option<usize> {
    let (lower, upper) = iterator.size_hint();
    (upper == Some(lower)).then_some(lower)
}

fn push_preview_details<Item>(
    details: &mut Vec<String>,
    preview: &Preview<Item>,
    decisive_index: Option<usize>,
) {
    details.push(format!("Consumed {} element(s).", preview.consumed));
    if preview.omitted() > 0 {
        details.push(format!(
            "Actual preview contains the last {} consumed elements; {} earlier element(s) omitted.",
            preview.items.len(),
            preview.omitted()
        ));
    }
    if let Some(index) = decisive_index {
        details.push(format!("Decisive element is at zero-based index {index}."));
    }
}

mod cardinality;
mod membership;
mod positional;
mod unordered;

pub(crate) use cardinality::{assert_has_length, assert_is_empty, assert_is_not_empty};
pub(crate) use membership::{
    assert_contains, assert_contains_all, assert_contains_matching, assert_contains_satisfying,
    assert_does_not_contain, assert_does_not_contain_matching, assert_does_not_contain_satisfying,
};
pub(crate) use positional::{
    assert_contains_contiguous, assert_contains_contiguous_matching,
    assert_contains_contiguous_satisfying, assert_contains_exactly,
    assert_contains_exactly_matching, assert_contains_exactly_satisfying, assert_ends_with,
    assert_ends_with_matching, assert_ends_with_satisfying, assert_starts_with,
    assert_starts_with_matching, assert_starts_with_satisfying,
};
pub(crate) use unordered::{
    assert_contains_exactly_in_any_order, assert_contains_exactly_in_any_order_matching,
    assert_contains_exactly_in_any_order_satisfying,
};
