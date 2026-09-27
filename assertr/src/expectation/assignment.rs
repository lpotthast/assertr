//! Bounded rejection evidence for exact unordered assignment.
//!
//! Assignment compares actual occurrences with expectation slots before knowing which groups will
//! need diagnostics. Keeping every rejected pair's failure evidence would make retention approach
//! the Cartesian product. This module samples the already explained failures while the caller owns
//! evaluation, its scalar comparison cache, and the matching algorithm.
//!
//! One sampled item is an owned [`AssertionFailure`], including its rendered values and any
//! constraint description or nested child failures. Nested failures stay attached to that item
//! without consuming additional positions in the sample. The original observations have already
//! been released after explanation.
//!
//! The caller drives two phases:
//! - Offer each newly rejected pair to [`CandidateSamples`]. Each occurrence and slot retains at
//!   most its inherited allowance of these failure items. The two samples share an admitted child
//!   and its cached key through one `Rc` allocation.
//! - Once assignment is known, call [`CandidateSamples::resolve`]. This consumes the union of both
//!   sample families into [`AssignedEvidence`]. Rejections against missing slots belong there
//!   exclusively. Other rejections belong to unmatched occurrences. Completed comparisons can then
//!   offer new evidence directly to these destinations.
//!
//! Sorted scopes rank the existing child-report text. Original occurrence, slot, and child ordinal
//! break text ties and supply the entire order in iteration-preserving scopes. Routing can discard
//! children whose groups no longer need them. Finite samples can therefore remain underfilled when
//! eligible replacements have already been discarded. Neither observations nor leaf renderers are
//! revisited here.
//!
//! The caller keeps these collectors separate from assembly contexts until comparison completion,
//! then supplies eligible direct-failure totals from its scalar cache. Sampling omissions and later
//! context truncation are counted once each. This module has no collection or assertion-family
//! dependency. The caller still constructs missing constraints, surplus descriptions, and outer
//! groups through its normal assertion context.

use crate::{
    AssertionFailure,
    expectation::Evidence,
    failure::adapter::{HumanReadableText, ToHumanReadableText},
    renderer::RenderingOrder,
    util::{
        matching::BipartiteMatchResult,
        selection::{Keyed, Smallest},
    },
};
use alloc::{collections::BTreeMap, rc::Rc, vec::Vec};

// Original identities break report-text ties independently of insertion and heap drain order.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct Key {
    report: Option<HumanReadableText>,
    identity: (usize, usize, usize),
}

impl Key {
    fn new(pair: (usize, usize), ordinal: usize, failure: &AssertionFailure, sorted: bool) -> Self {
        Self {
            report: sorted.then(|| ToHumanReadableText::render_child(failure)),
            identity: (pair.0, pair.1, ordinal),
        }
    }
}

type Candidate = Keyed<Key, AssertionFailure>;
type Sample = Smallest<Rc<Candidate>>;
type Collector = Smallest<Candidate>;

/// Provisional rejection samples for each actual occurrence and expectation slot.
///
/// Retains at most `limit * (n + m)` memberships. A failure shared by two samples has two
/// memberships but one owned value. The caller's current pair evidence is additional. This is not
/// an absolute memory bound on failures or keys.
pub(crate) struct CandidateSamples {
    occurrences: Vec<Sample>,
    slots: Vec<Sample>,
    limit: usize,
    sorted: bool,
}

impl CandidateSamples {
    /// Creates independent samples using the effective allowance and order of the child context.
    pub(crate) fn new(n: usize, m: usize, limit: usize, order: RenderingOrder) -> Self {
        Self {
            occurrences: (0..n).map(|_| Smallest::new(limit)).collect(),
            slots: (0..m).map(|_| Smallest::new(limit)).collect(),
            limit,
            sorted: order == RenderingOrder::SortByRenderedText,
        }
    }

    /// Consumes one newly rejected pair's already scoped children, without retaining observations.
    /// Pair indexes must address the original occurrence and slot lists. Offer each pair only once.
    pub(crate) fn offer(&mut self, pair: (usize, usize), evidence: Evidence) {
        for (ordinal, failure) in evidence.children.into_iter().enumerate() {
            let candidate = Keyed {
                key: Key::new(pair, ordinal, &failure, self.sorted),
                value: failure,
            };
            let occurrence = self.occurrences[pair.0].would_retain(&candidate);
            let slot = self.slots[pair.1].would_retain(&candidate);
            if occurrence || slot {
                // Losing offers never allocate shared ownership. Both memberships share the
                // complete failure and its cached report, without cloning either one.
                let candidate = Rc::new(candidate);
                if occurrence {
                    self.occurrences[pair.0].offer(Rc::clone(&candidate));
                }
                if slot {
                    self.slots[pair.1].offer(candidate);
                }
            }
        }
    }

    /// Consumes all surviving candidates exactly once and routes them by the final assignment.
    /// `result` addresses the same occurrence and slot lists. The effective allowance is preserved.
    pub(crate) fn resolve(self, result: &BipartiteMatchResult) -> AssignedEvidence {
        let mut destinations = AssignedEvidence {
            missing: result
                .unmatched_expected
                .iter()
                .map(|&slot| (slot, Smallest::new(self.limit)))
                .collect(),
            unexpected: result
                .unmatched_actual
                .iter()
                .map(|&index| (index, Smallest::new(self.limit)))
                .collect(),
            sorted: self.sorted,
        };
        // Consume the union, including samples whose own group is irrelevant after assignment.
        // Only the last membership recovers the owned failure. No additional union collection is
        // needed.
        for sample in self.occurrences.into_iter().chain(self.slots) {
            for candidate in sample.into_unsorted() {
                if let Some(candidate) = Rc::into_inner(candidate) {
                    destinations.route(candidate);
                }
            }
        }
        destinations
    }
}

/// Rejection evidence routed to missing slots and unmatched occurrences after assignment.
///
/// Separate from assembly contexts so retained failures cannot reduce later evaluation budgets.
pub(crate) struct AssignedEvidence {
    missing: BTreeMap<usize, Collector>,
    unexpected: BTreeMap<usize, Collector>,
    sorted: bool,
}

impl AssignedEvidence {
    /// Routes a newly completed rejection directly, without competing in provisional samples.
    pub(crate) fn offer(&mut self, pair: (usize, usize), evidence: Evidence) {
        for (ordinal, failure) in evidence.children.into_iter().enumerate() {
            self.route(Keyed {
                key: Key::new(pair, ordinal, &failure, self.sorted),
                value: failure,
            });
        }
    }

    fn route(&mut self, candidate: Candidate) {
        let (index, slot, _) = candidate.key.identity;
        // A missing slot owns every rejection against it, exclusively.
        let destination = self
            .missing
            .get_mut(&slot)
            .or_else(|| self.unexpected.get_mut(&index));
        if let Some(collector) = destination {
            collector.offer(candidate);
        }
    }

    /// Removes a missing slot's ordered sample. `total` counts all its eligible direct failures,
    /// including omitted children from pair evidence and candidates discarded during sampling.
    pub(crate) fn take_missing(&mut self, slot: usize, total: usize) -> Evidence {
        finish(self.missing.remove(&slot).expect("missing slot"), total)
    }

    /// Removes an unmatched occurrence's ordered sample. `total` excludes missing-slot rejections,
    /// which belong exclusively to those slots. The caller's assembly context applies its budget.
    pub(crate) fn take_unexpected(&mut self, index: usize, total: usize) -> Evidence {
        finish(
            self.unexpected
                .remove(&index)
                .expect("unmatched occurrence"),
            total,
        )
    }

    /// Drops rejections for an occurrence that instead needs an occupied-expectation description.
    pub(crate) fn discard_surplus(&mut self, index: usize) {
        self.unexpected.remove(&index);
    }
}

fn finish(collector: Collector, total: usize) -> Evidence {
    let children = collector.into_values();
    Evidence {
        omitted: total - children.len(),
        children,
        // Assembly appends the already scoped paths and supplies its own prefix afterwards.
        path_prefix_len: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        failure::{FailureBuilder, FailureKind},
        prelude::*,
    };
    use alloc::collections::BTreeSet;

    fn branch(values: &[usize]) -> Evidence {
        let context = AssertionContext::default();
        Evidence {
            children: values
                .iter()
                .map(|value| {
                    FailureBuilder::detached::<usize>(FailureKind::Matching)
                        .actual(context.render().value(value))
                        .child(
                            FailureBuilder::detached::<()>(FailureKind::Other)
                                .relation("nested rejection")
                                .omitted_children(7)
                                .build(),
                        )
                        .build()
                })
                .collect(),
            ..Evidence::default()
        }
    }

    fn roots(samples: &CandidateSamples) -> BTreeSet<*const Candidate> {
        samples
            .occurrences
            .iter()
            .chain(&samples.slots)
            .flat_map(Smallest::retained)
            .map(Rc::as_ptr)
            .collect()
    }

    #[test]
    fn memberships_share_roots_and_reports_and_release_replaced_candidates() {
        let (n, m, k) = (4, 3, 2);
        let mut samples = CandidateSamples::new(n, m, k, RenderingOrder::SortByRenderedText);
        let mut old_roots = Vec::new();
        for index in (0..n).rev() {
            for slot in (0..m).rev() {
                let branch = branch(&[index * 10 + slot, index * 10 + slot + 1]);
                let current_branch = branch.children.len();
                let sampled_roots = roots(&samples).len();
                assert_that!(current_branch).is_equal_to(k);
                assert_that!(sampled_roots + current_branch).is_less_or_equal_to(k * (n + m) + k);
                samples.offer((index, slot), branch);
                let mut memberships = 0;
                for sample in samples.occurrences.iter().chain(&samples.slots) {
                    assert_that!(sample.retained().len()).is_less_or_equal_to(k);
                    for candidate in sample.retained() {
                        memberships += 1;
                        assert_that!(Rc::strong_count(candidate)).is_less_or_equal_to(2);
                        old_roots.push(Rc::downgrade(candidate));
                    }
                }
                assert_that!(memberships).is_less_or_equal_to(k * (n + m));
                assert_that!(roots(&samples).len()).is_less_or_equal_to(memberships);
            }
        }
        assert_that!(old_roots.iter().any(|root| root.strong_count() == 0)).is_true();
        let retained_roots = roots(&samples).len();
        let result = BipartiteMatchResult {
            matched_pairs: vec![(0, 1), (1, 2)],
            unmatched_actual: vec![2, 3],
            unmatched_expected: vec![0],
        };
        let destinations = samples.resolve(&result);
        // Every Rc is consumed. Final collectors own the moved failures and cached report strings.
        assert_that!(old_roots.iter().all(|root| root.strong_count() == 0)).is_true();
        let final_roots = destinations
            .missing
            .values()
            .chain(destinations.unexpected.values())
            .map(|collector| collector.retained().len())
            .sum::<usize>();
        assert_that!(final_roots).is_less_or_equal_to(retained_roots);
    }

    #[test]
    fn union_and_completion_recover_candidates_excluded_from_the_occurrence_sample() {
        for completed in [false, true] {
            let mut samples = CandidateSamples::new(3, 3, 1, RenderingOrder::PreserveIteration);
            if completed {
                samples.offer((0, 1), branch(&[1])); // Assigned occurrence and occupied slot.
            }
            samples.offer((1, 0), branch(&[10])); // Missing slot owns the occurrence's sample.
            if !completed {
                samples.offer((1, 1), branch(&[11])); // Only the occupied slot's sample admits it.
            }
            let result = BipartiteMatchResult {
                matched_pairs: vec![(0, 2), (2, 1)],
                unmatched_actual: vec![1],
                unmatched_expected: vec![0],
            };
            let mut destinations = samples.resolve(&result);
            if completed {
                destinations.offer((1, 1), branch(&[11])); // Bypass both irrelevant samples.
            }
            let unexpected = destinations.take_unexpected(1, 1);
            assert_that!(destinations.take_missing(0, 1).children).has_length(1);
            assert_that!(unexpected.children).has_length(1);
            assert_that!(unexpected.children[0].actual.as_ref().unwrap().text(false))
                .is_equal_to("11");
            assert_that!(unexpected.omitted).is_equal_to(0);
            assert_that!(unexpected.children[0].children[0].omitted_children).is_equal_to(7);
        }
    }

    #[test]
    fn report_ties_use_original_identity_after_heap_draining_and_union_recovery() {
        for order in [
            RenderingOrder::PreserveIteration,
            RenderingOrder::SortByRenderedText,
        ] {
            let mut samples = CandidateSamples::new(3, 2, 2, order);
            for index in (0..3).rev() {
                for slot in (0..2).rev() {
                    samples.offer((index, slot), branch(&[1, 1]));
                }
            }
            let result = BipartiteMatchResult {
                matched_pairs: vec![],
                unmatched_actual: vec![0, 1, 2],
                unmatched_expected: vec![0, 1],
            };
            let destinations = samples.resolve(&result);
            for (slot, collector) in destinations.missing {
                let identities: Vec<_> = collector
                    .into_sorted()
                    .into_iter()
                    .map(|candidate| candidate.key.identity)
                    .collect();
                assert_that!(identities).is_equal_to([(0, slot, 0), (0, slot, 1)]);
            }
        }
    }
}
