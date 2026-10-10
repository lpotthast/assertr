#![cfg_attr(not(feature = "std"), no_std)]

//! Downstream compatibility checks for `core`, `alloc`, and optional features.
//! Keep representative capabilities, borrowed views, and observation types here. Assertion
//! behavior belongs in the owning unit tests. `cfg_attr(test, test)` runs compile checks on the
//! host while keeping their bodies available to embedded builds.

extern crate alloc;

use alloc::{boxed::Box, string::String};

use assertr::{
    matchers::{entry, predicate},
    prelude::*,
};

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn memory_assertions_compile_without_std() {
    use alloc::string::String;

    use assertr::{
        assertions::MemAssertions, matchers::memory::NeedsDrop,
        prelude::MemAssertions as PreludeMemAssertions,
    };

    struct NoRenderer;
    fn check<A: MemAssertions>(assertion: A) -> A {
        assertion.needs_drop()
    }

    check(assert_that_type::<String>().with_renderer(NoRenderer)).matches(NeedsDrop);
    // The assertions module and the prelude expose the same trait.
    PreludeMemAssertions::needs_drop(assert_that_type::<String>());

    let failures = assert_that_type::<u32>()
        .with_renderer(NoRenderer)
        .capture(|it| check(it).matches(NeedsDrop));
    assert_that!(failures).has_length(2);
    for failure in &failures {
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not need drop"));
    }
}

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn projections_compile_without_renderer_support() {
    struct Field {
        byte: u8,
    }
    #[derive(Clone)]
    struct NoRenderer;

    let subject = (Field { byte: 1 }, Field { byte: 2 });
    let root = assert_that!(subject).with_renderer(NoRenderer);
    let child: AssertThat<Field, Panic, NoRenderer> = root.derive(|subject| &subject.0);
    child.is_same_instance_as(&subject.0);
    root.derive(|subject| &subject.1)
        .is_same_instance_as(&subject.1);
    let computed: AssertThat<Field, Panic, NoRenderer> = root.derive_owned(|_| Field { byte: 3 });
    assert_that!(computed.actual().byte).is_equal_to(3);
    // Type-check the async projection on embedded targets without requiring an executor.
    let projection = root.derive_async(|_| core::future::ready(Field { byte: 4 }));
    drop(projection);

    let failures = assert_that_owned!(subject)
        .with_renderer(NoRenderer)
        .capture(|root| {
            root.derive(|subject| &subject.0)
                .is_same_instance_as(&root.actual().0);
            root
        });
    assert_that!(failures).is_empty();

    let fact = assertr::failure::Fact::note("evidence");
    assert_that!(fact)
        .with_renderer(NoRenderer)
        .derive(|fact| &fact.value.body)
        .is_same_instance_as(&fact.value.body);
}

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn unwind_safe_projections_compile_without_std() {
    use core::{
        cell::Cell,
        panic::{RefUnwindSafe, UnwindSafe},
    };
    fn both<T: UnwindSafe + RefUnwindSafe>(_: &T) {}
    struct NoRenderer;

    let failures = assert_that_owned!(Cell::new(1))
        .with_renderer(Cell::new(0))
        .capture(|root| {
            let child = root.derive_owned(Cell::get).with_renderer(NoRenderer);
            both(&child);
            child.track_assertion();
            root
        });
    assert_that!(failures).is_empty();
}

#[cfg(feature = "num")]
#[allow(dead_code)]
#[cfg_attr(test, test)]
fn numeric_assertions_compile_without_std() {
    use assertr::{assertions::NumericDistance, matchers::IsCloseTo};

    // The tolerance expectation needs only `NumericDistance`, not the `num` feature's traits.
    fn assert_close<T: NumericDistance + core::fmt::Debug>(actual: T, expected: T, deviation: T) {
        assert_that_owned!(actual).matches(IsCloseTo::new(expected, deviation));
    }

    assert_close(i128::MIN, -1, i128::MAX);
    assert_close(0_u128, u128::MAX, u128::MAX);
    assert_close(-1.0_f32, 16_777_216.0, 16_777_216.0);
    assert_close(f64::INFINITY, f64::INFINITY, 0.0);
    let failures = assert_that!(9_007_199_254_740_992_f64)
        .capture(|it| it.is_close_to(9_007_199_254_740_994.0, 1.0));
    assert_that!(failures).has_length(1);
    with_context(DebugRenderer, |context| {
        let close = IsCloseTo::new(2, 1);
        assert_that!(Expectation::evaluate(&close, &3, context).is_ok()).is_true();
    });
}

#[allow(dead_code)]
fn capture_errors_compile_without_std() -> Result<(), Box<dyn core::error::Error + Send + Sync>> {
    use alloc::string::ToString;
    let failures = assert_that_owned!(0..).capture(|it| it.starts_with([1, 2]));
    let _report = failures.to_string();
    let single = assert_that!(failures).get_single().actual().clone();
    let _: Box<dyn core::error::Error + Send + Sync> = single.into();
    let result: Result<(), AssertionFailures> = Err(failures);
    result?;
    Ok(())
}

#[allow(dead_code)]
fn identity_assertions_compile_without_std() {
    struct Key {
        _byte: u8,
    }
    struct NoRenderer;
    let keys = [Key { _byte: 1 }, Key { _byte: 2 }, Key { _byte: 3 }];
    let candidates = [&keys[1], &keys[0]];
    assert_that!(candidates[0])
        .with_renderer(NoRenderer)
        .is_same_instance_as(&keys[1])
        .is_not_same_instance_as(&keys[0]);
    assert_that!(alloc::collections::LinkedList::from(candidates))
        .with_renderer(NumericRenderer)
        .contains_same_instance_as(&keys[1])
        .does_not_contain_same_instance_as(&keys[2])
        .contains_exactly_same_instances([&keys[1], &keys[0]])
        .contains_exactly_same_instances_in_any_order([&keys[0], &keys[1]]);
    let data = [1, 2];
    assert_that!([&data[..1], &data[..]])
        .with_renderer(NoRenderer)
        .contains_exactly_same_instances_in_any_order([&data[..], &data[..1]]);
}

#[allow(dead_code)]
fn panic_presentation_compiles_without_std() {
    use alloc::string::ToString;

    struct NoRenderer;

    let _assertion = assert_that!(1)
        .with_renderer(NoRenderer)
        .with_panic_presentation(ToString::to_string);
}

#[allow(dead_code)]
fn pattern_assertions_compile_without_std() {
    assert_that!(Some(42)).is_matching(pattern!(Some(42)));
}

#[allow(dead_code)]
fn iterator_assertions_compile_without_std() {
    fn positive(it: AssertThat<i32, Capture>) {
        it.is_greater_than(0);
    }

    assert_that_owned!(0..).contains(2);
    assert_that_owned!(1..).contains_satisfying(positive);
    assert_that_owned!(0..).contains_contiguous([2, 3]);
    assert_that_owned!([2, 1].into_iter()).contains_exactly_in_any_order([1, 2]);
    assert_that!([1, 2].into_iter()).has_remaining_count(2);

    assert_that!([1, 2])
        .into_iter_contains_all([2, 1])
        .starts_with([1])
        .ends_with([2])
        .contains_contiguous_satisfying([positive, positive])
        .into_iter_contains_exactly_in_any_order([2, 1]);
}

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn callback_assertions_compile_without_subject_renderers() {
    struct Secret;
    #[derive(Clone)]
    struct NoRenderer;

    fn is_some<R>(it: AssertThat<'_, Option<Secret>, Capture, R>) {
        it.is_some();
    }

    let values = [Some(Secret)];
    assert_that!(values)
        .with_renderer(NoRenderer)
        .contains_satisfying(is_some);
    assert_that!(values)
        .with_renderer(NumericRenderer)
        .contains_exactly_in_any_order_satisfying([is_some])
        .starts_with_satisfying([is_some])
        .ends_with_satisfying([is_some])
        .contains_contiguous_satisfying([is_some])
        .contains_exactly_satisfying([is_some])
        .into_iter_contains_satisfying(is_some)
        .into_iter_contains_exactly_in_any_order_satisfying([is_some]);
    assert_that_owned!([Some(Secret)].into_iter())
        .with_renderer(NumericRenderer)
        .contains_exactly_satisfying([is_some]);
    assert_that!(alloc::collections::BTreeMap::from([(
        1_usize,
        Some(Secret)
    )]))
    .with_renderer(NumericRenderer)
    .contains_entry_satisfying(&1_usize, is_some)
    .contains_exactly_entries_satisfying([(1_usize, is_some)]);

    let failures = assert_that!([None::<Secret>])
        .with_renderer(NoRenderer)
        .capture(|it| it.contains_satisfying(is_some));
    assert_that!(failures).has_length(1);
}

/// The set and map families live outside the `std` module, so `BTreeSet` and `BTreeMap` carry them
/// into `no_std` builds.
#[allow(dead_code)]
fn set_and_map_assertions_compile_without_std() {
    use alloc::collections::{BTreeMap, BTreeSet};

    use assertr::{expectation::Expectation, matchers::ContainsMatching};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_one(value: &i32) -> bool {
        *value == 1
    }

    fn satisfies_one(it: AssertThat<i32, Capture>) {
        it.is_equal_to(1);
    }

    // Evaluation needs no renderer when the element matcher needs none for its diagnostics.
    struct NoRenderer;
    let matcher = matchers::anything();
    let expected = ContainsMatching::new(&matcher);
    with_context(NoRenderer, |context| {
        assert_that!(expected.evaluate([1, 2].as_slice(), context).is_ok()).is_true();

        assert_that!(BTreeSet::from([1, 2, 3]))
            .contains(2)
            .does_not_contain(4)
            .contains_all([1, 3])
            .contains_matching(predicate(|it: &i32| *it > 2))
            .contains_exactly_in_any_order([3, 2, 1])
            .is_subset_of(BTreeSet::from([1, 2, 3, 4]))
            .is_superset_of(BTreeSet::from([1]))
            .is_disjoint_from(BTreeSet::from([9]))
            .has_length(3);

        assert_that!(BTreeMap::from([("a", 1)]))
            .contains_key("a")
            .does_not_contain_key("b")
            .contains_value(1)
            .contains_entry("a", 1)
            .contains_entry_satisfying("a", satisfies_one)
            .contains_keys(["a"])
            .contains_exactly_entries([("a", 1)])
            .contains_exactly_entries_matching([entry("a", predicate(is_one))])
            .contains_exactly_entries_satisfying([("a", satisfies_one)])
            .has_length(1);

        let expected = [entry("a", predicate(is_one))];
        assert_that!(BTreeMap::from([("a", 1)]))
            .contains_exactly_entries_matching(&expected[..])
            .matches(matchers::entries_are(expected));
    });
}

/// A `LinkedList` is an ordered collection, so it gets the order-sensitive assertions too.
#[allow(dead_code)]
fn linked_list_assertions_compile_without_std() {
    use alloc::collections::LinkedList;

    let list = [1, 2, 3].into_iter().collect::<LinkedList<_>>();
    assert_that!(list)
        .contains(2)
        .contains_exactly([1, 2, 3])
        .contains_exactly_in_any_order([3, 2, 1])
        .has_length(3);
}

#[cfg(all(test, not(feature = "std")))]
extern crate std;

#[cfg(all(test, not(feature = "std")))]
mod tests {
    use assertr::{assert_that, prelude::StrAssertions};

    use crate::NumericRenderer;

    mod matchers {
        use assertr::{
            matchers::{eq, ge},
            prelude::*,
        };

        #[cfg(feature = "partial")]
        #[test]
        fn structural_matchers_work_with_alloc() {
            crate::structural_matchers_without_std();
        }

        #[test]
        fn runtime_matchers_need_no_features() {
            assert_that!([1, 2]).matches(elements_are![eq(1), eq(2)]);
            assert_that!(3).matches(ge(2));
            crate::bounded_unordered_matching_without_std();
        }
    }

    use alloc::{
        string::{String, ToString},
        sync::Arc,
    };
    use core::sync::atomic::{AtomicUsize, Ordering};

    use assertr::{
        failure::AssertionFailure,
        prelude::{
            CollectionAssertions, IdentityAssertions, LengthAssertions, PartialEqAssertions,
            StableOrderAssertions,
        },
    };

    fn counting(
        count: &Arc<AtomicUsize>,
    ) -> impl Fn(&AssertionFailure) -> String + core::panic::RefUnwindSafe + Send + Sync + 'static
    {
        let count = Arc::clone(count);
        move |failure| {
            count.fetch_add(1, Ordering::Relaxed);
            failure.to_string()
        }
    }

    #[test]
    fn opaque_identity_assertions_capture_and_panic_without_std() {
        struct Key {
            _byte: u8,
        }
        struct NoRenderer;
        super::identity_assertions_compile_without_std();
        let keys = [Key { _byte: 1 }, Key { _byte: 1 }, Key { _byte: 1 }];
        let failures = assertr::assert_that!(keys[0])
            .with_renderer(NoRenderer)
            .capture(|it| {
                it.is_same_instance_as(&keys[1])
                    .is_not_same_instance_as(&keys[0])
            });
        assert_that!(failures).has_length(2);
        let failures = assertr::assert_that!([&keys[0], &keys[1]])
            .with_renderer(NumericRenderer)
            .capture(|it| {
                it.contains_same_instance_as(&keys[2])
                    .does_not_contain_same_instance_as(&keys[0])
                    .contains_exactly_same_instances([&keys[1], &keys[0]])
                    .contains_exactly_same_instances_in_any_order([&keys[0], &keys[0]])
            });
        assert_that!(failures).has_length(4);
        let panic = std::panic::catch_unwind(|| {
            assertr::assert_that!(keys[0])
                .with_renderer(NoRenderer)
                .is_same_instance_as(&keys[1]);
        })
        .unwrap_err();
        assert_that!(panic.downcast_ref::<String>().unwrap())
            .contains("is not the same instance as");
    }

    #[test]
    fn a_presentation_runs_only_in_panic_mode_without_std() {
        let count = Arc::new(AtomicUsize::new(0));
        let failures = assertr::assert_that!(1)
            .with_panic_presentation(counting(&count))
            .capture(|it| it.is_equal_to(2));
        assert_that!(failures).has_length(1);
        assert_that!(count.load(Ordering::Relaxed)).is_equal_to(0);

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assertr::assert_that!(1)
                .with_panic_presentation(counting(&count))
                .is_equal_to(2);
        }))
        .unwrap_err();

        assert_that!(panic.downcast_ref::<String>().unwrap())
            .contains("Expected: 2\n\n  Actual: 1");
        assert_that!(count.load(Ordering::Relaxed)).is_equal_to(1);
    }

    #[test]
    fn a_panicking_presentation_propagates_without_std() {
        crate::panic_presentation_compiles_without_std();
        let panic = std::panic::catch_unwind(|| {
            assertr::assert_that!(1)
                .with_panic_presentation(|_| panic!("presentation exploded"))
                .is_equal_to(2);
        })
        .unwrap_err();
        assert_that!(*panic.downcast_ref::<&str>().unwrap()).is_equal_to("presentation exploded");
    }

    #[test]
    // The `if` around the panic keeps the closure's return type inferable; an `assert!` would
    // change the panic payload.
    #[allow(clippy::manual_assert)]
    fn a_panic_inside_a_capture_closure_preserves_the_original_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _failures = assertr::assert_that!(42).capture(|it| {
                let it = it.is_equal_to(43);
                if it.actual() == &42 {
                    panic!("original panic");
                }
                it
            });
        })
        .expect_err("the closure should panic");

        assertr::assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(&"original panic"));
    }
}

/// Exercises shared candidate selection and routing on hosted and embedded alloc targets.
#[allow(dead_code)]
fn bounded_unordered_matching_without_std() {
    use assertr::matchers::{eq, ge};
    for limit in [0, 1, 2, usize::MAX] {
        let failures = assert_that!([1, 2, 99])
            .with_rendering_budget(RenderingBudget::unlimited().with_max_items(limit))
            .capture(|it| it.matches(elements_are_in_any_order![ge(0), eq(1), eq(3)]));
        assert_that!(failures).has_length(1);
        assert_that!([1, 2])
            .with_rendering_budget(RenderingBudget::unlimited().with_max_items(limit))
            .matches(elements_are_in_any_order![ge(0), eq(1)]);
    }
}

/// Structural matching remains available with alloc and no std.
#[cfg(feature = "partial")]
pub fn structural_matchers_without_std() {
    use assertr::{matchers::eq, prelude::*};
    struct Hidden;
    #[allow(dead_code)]
    struct Child {
        id: u32,
        hidden: Hidden,
    }
    let children = alloc::vec![Child {
        id: 1,
        hidden: Hidden
    }];
    assert_that!(children).matches(elements_are![partial!(Child { id: eq(1), .. })]);
    assert_that!(children)
        .with_renderer(NumericRenderer)
        .into_iter_contains_matching(partial!(Child {
            id: matchers::anything(),
            ..
        }));
    let map = alloc::collections::BTreeMap::from([(
        "child",
        Child {
            id: 1,
            hidden: Hidden,
        },
    )]);
    assert_that!(map).matches(entries_are![("child", partial!(Child { id: eq(1), .. }))]);
}

#[derive(Clone)]
struct NumericRenderer;
impl ValueRenderer<usize> for NumericRenderer {
    fn fmt(&self, value: &usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(value, f)
    }
}

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn typed_rejections_and_numeric_evidence_compile_without_std() {
    use assertr::{
        expectation::{AssertionContext, Expectation},
        failure::{Fact, FailureBuilder, FailureKind},
        matchers::each,
    };

    struct OpaqueError(u32);
    struct Reject;
    impl<R: ValueRenderer<OpaqueError>> Expectation<u32, R> for Reject {
        type Success<'a> = ();
        type Rejection<'a> = OpaqueError;

        fn evaluate(&self, value: &u32, _: &AssertionContext<'_, R>) -> Result<(), OpaqueError> {
            Err(OpaqueError(*value))
        }

        const KIND: FailureKind = FailureKind::Predicate;

        fn explain(
            &self,
            rejected: Option<(&u32, OpaqueError)>,
            failure: FailureBuilder,
            context: &AssertionContext<'_, R>,
        ) -> FailureBuilder {
            match rejected {
                None => failure.relation("is accepted"),
                Some((_, error)) => failure
                    .relation("is rejected")
                    .fact(Fact::note(context.render().value(&error))),
            }
        }
    }

    struct ErrorRenderer;
    impl ValueRenderer<OpaqueError> for ErrorRenderer {
        fn fmt(&self, error: &OpaqueError, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "error({})", error.0)
        }
    }
    fn iterator_trait<A: ExactSizeIteratorAssertions<NumericRenderer>>() {}
    iterator_trait::<AssertThat<'static, core::array::IntoIter<u32, 1>, Panic, NumericRenderer>>();
    let assertion = Reject;
    with_context(ErrorRenderer, |context| {
        let error = assertion.evaluate(&7, context).unwrap_err();
        assert_that!(error.0).is_equal_to(7);
        let failures = assert_that!(7_u32)
            .with_renderer(ErrorRenderer)
            .capture(|it| it.matches(&assertion));
        assert_that!(failures[0].facts[0].value.type_name)
            .is_equal_to(Some(core::any::type_name::<OpaqueError>()));
        let failures = assert_that!([7_u32, 8])
            .with_renderer(ErrorRenderer)
            .capture(|it| it.matches(each(&assertion)));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children).has_length(2);
        for child in &failures[0].children {
            assert_that!(child.facts[0].value.type_name)
                .is_equal_to(Some(core::any::type_name::<OpaqueError>()));
        }
        let failures = assert_that!(7_u32)
            .with_renderer(ErrorRenderer)
            .capture(|it| it.matches(&assertion));
        assert_that!(failures).has_length(1);
        let failures = assert_that!([7_u32].into_iter())
            .with_renderer(NumericRenderer)
            .capture(|it| it.has_remaining_count(2));
        assert_that!(failures[0].expected.as_ref().unwrap().type_name).is_equal_to(Some("usize"));
    });
}

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn reusable_checks_compile_without_std() {
    use assertr::matchers::{field, predicate};

    struct Person {
        name: String,
    }
    let has_name = field(
        "name",
        |person: &Person| &person.name,
        predicate(|name: &String| !name.is_empty())
            .described_as("is not empty")
            .rejected_as("is empty"),
    );
    assert_that!(Person { name: "Ada".into() }).matches(&has_name);
    let failures = assert_that!(Person {
        name: String::new()
    })
    .capture(|it| it.matches(&has_name));
    assert_that!(failures[0].children[0].relation.as_deref()).is_equal_to(Some("is empty"));
}

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn assertion_definitions_compile_without_std() {
    use alloc::string::String;

    use assertr::{
        expectation::Expectation,
        matchers::{
            EqualTo, GreaterOrEqual, HasDebugString, IsOfType, IsOk, IsReady, IsSome,
            cell::{IsBorrowed, IsNotMutablyBorrowed},
            string::StartsWith,
        },
    };
    with_context(DebugRenderer, |context| {
        // One definition per leaf capability.
        assert_that!(EqualTo::new(3).evaluate(&3, context).is_ok()).is_true();
        assert_that!(GreaterOrEqual::new(0.0).evaluate(&1.0, context).is_ok()).is_true();
        let prefix = StartsWith::new("hel");
        assert_that!(prefix.evaluate("hello", context).is_ok()).is_true();
        assert_that!(HasDebugString::new("123").evaluate(&123, context).is_ok()).is_true();

        // Rejection and success can retain different guard types.
        let cell = core::cell::RefCell::new(7);
        let rejection: core::cell::RefMut<'_, i32> =
            IsBorrowed.evaluate(&cell, context).err().unwrap();
        drop(rejection);
        let assertion = assert_that!(cell);
        let guard: core::cell::Ref<'_, i32> =
            assertion.test_assertion(&IsNotMutablyBorrowed).unwrap();
        drop(guard);

        // Borrowed payloads include an optional value, an allocated value, and a nested reference.
        let optional = Some(3);
        let assertion = assert_that!(optional);
        let value: Option<&i32> = assertion.test_assertion(&IsSome);
        assert_that!(value).is_equal_to(Some(&3));
        let result = Ok::<_, ()>(String::from("borrowed"));
        let value = IsOk.evaluate(&result, context).unwrap();
        assert_that!(value).is_same_instance_as(result.as_ref().unwrap());
        let ready = core::task::Poll::Ready(value);
        let assertion = assert_that!(ready);
        let observed = assertion.test_assertion(&IsReady).unwrap();
        assert_that!(*observed).is_same_instance_as(value);
        let boxed: Box<dyn core::any::Any> = Box::new(123);
        assert_that!(boxed).matches(IsOfType::<i32>::new());
    });
}

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn collection_assertion_definitions_compile_without_std() {
    use assertr::{
        expectation::Expectation,
        matchers::{HasLengthOf, collection, iterator, map, set},
    };

    with_context(DebugRenderer, |context| {
        let values = [1, 2];
        assert_that!(HasLengthOf::new(2).evaluate(&values, context).is_ok()).is_true();
        assert_that!(
            collection::ContainsExactly::new([1, 2])
                .evaluate(&values, context)
                .is_ok()
        )
        .is_true();
        let first = collection::HasFirst.evaluate(&values, context).unwrap();
        assert_that!(first).is_same_instance_as(&values[0]);
        let index = collection::HasElementAt::new(1);
        let second = index.evaluate(&values, context).unwrap();
        assert_that!(second).is_same_instance_as(&values[1]);
        let map = alloc::collections::BTreeMap::from([(1, 2)]);
        assert_that!(
            map::ContainsEntry::new(&1, 2)
                .evaluate(&map, context)
                .is_ok()
        )
        .is_true();
        assert_that!(
            map::ContainsExactlyEntries::new([(1, 2)])
                .evaluate(&map, context)
                .is_ok()
        )
        .is_true();
        let set = alloc::collections::BTreeSet::from([1]);
        assert_that!(set::IsSubsetOf::new(&set).evaluate(&set, context).is_ok()).is_true();
        assert_that!(
            iterator::HasRemainingCount::new(2)
                .evaluate(&values.iter(), context)
                .is_ok()
        )
        .is_true();
        assert_that!([&values[0]]).matches(collection::ContainsSameInstanceAs::new(&values[0]));
    });
}

// Tests renderer-independent evaluation through the same boundary available downstream.
struct InContext<F>(F);

impl<R, F: Fn(&assertr::expectation::AssertionContext<'_, R>)> Expectation<(), R> for InContext<F> {
    type Success<'a>
        = ()
    where
        Self: 'a,
        (): 'a;
    type Rejection<'a>
        = core::convert::Infallible
    where
        Self: 'a,
        (): 'a;

    fn evaluate<'a>(
        &'a self,
        (): &'a (),
        context: &assertr::expectation::AssertionContext<'_, R>,
    ) -> Result<(), core::convert::Infallible> {
        (self.0)(context);
        Ok(())
    }

    const KIND: assertr::failure::FailureKind = assertr::failure::FailureKind::Other;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a (), core::convert::Infallible)>,
        failure: assertr::failure::FailureBuilder,
        _: &assertr::expectation::AssertionContext<'_, R>,
    ) -> assertr::failure::FailureBuilder {
        match rejected {
            None => failure.relation("evaluates with the supplied context"),
            Some(((), never)) => match never {},
        }
    }
}

fn with_context<R, F: Fn(&assertr::expectation::AssertionContext<'_, R>)>(renderer: R, f: F) {
    assert_that!(())
        .with_renderer(renderer)
        .matches(InContext(f));
}

// Keep the alloc-only API boundary here. Detailed rendering behavior is tested in the renderer.
#[allow(dead_code)]
mod structural_rendering {
    use alloc::collections::{BTreeMap, BTreeSet};
    use core::{cell::RefCell, fmt};

    use assertr::{
        failure::{Fact, FailureKind},
        prelude::*,
        renderer::{Rendered, RenderingContext, RenderingOrder},
    };

    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    struct Token(u8);
    struct LeafRenderer;
    struct NoRenderer;

    impl ValueRenderer<Token> for LeafRenderer {
        fn fmt(&self, value: &Token, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "token({})", value.0)
        }
    }

    fn verify() {
        let map = BTreeMap::from([(Token(1), Token(10))]);
        let set = BTreeSet::from([Token(1)]);
        let failures = assert_that!(map).with_renderer(LeafRenderer).capture(|it| {
            it.track_assertion();
            it.raise(
                it.failure(FailureKind::Length)
                    .actual(it.render().map(it.actual()))
                    .relation("is not empty")
                    .fact(Fact::labelled("Keys", it.render().collection(&set))),
            );
            it
        });
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].actual).is_some();
        assert_that!(failures[0].facts).has_length(1);

        let render = RenderingContext::new(&LeafRenderer, RenderingBudget::default());
        let tokens = [Token(1)];
        let borrowed = [&tokens[0]];
        let entries = [(&tokens[0], &tokens[0])];
        let owner = Some(Token(3));
        let cell = RefCell::new(Token(4));
        // Every structural method must compile with alloc and leaf-only rendering.
        let _: [Rendered; 7] = [
            render.collection(&tokens),
            render.stable_collection(&tokens),
            render.map(&BTreeMap::from([(Token(2), Token(20))])),
            render.borrowed_values::<Token, _>(&borrowed, RenderingOrder::SortByRenderedText),
            render
                .entry_list::<Token, Token, _, _, _>(&entries, RenderingOrder::SortByRenderedText),
            render.variant(&owner, "Some", owner.as_ref().unwrap()),
            render.struct_field(&cell, "RefCell", "value", &*cell.borrow()),
        ];
        let _: Rendered = RenderingContext::new(&NoRenderer, RenderingBudget::default())
            .unavailable_struct_field(&cell, "RefCell", "value", "<borrowed>");
    }

    #[test]
    fn public_structural_rendering_works_with_alloc() {
        verify();
    }
}

struct TextOperand<'a>(&'a str);
impl core::borrow::Borrow<str> for TextOperand<'_> {
    fn borrow(&self) -> &str {
        self.0
    }
}
impl assertr::borrow_for::BorrowFor<String> for TextOperand<'_> {
    type View = str;
}

fn reusable_bulk_views_compile_without_std() {
    use alloc::{collections::BTreeMap, string::String, vec};

    // Borrow non-Clone custom operands as a Vec and a slice, and reuse their matcher.
    let operands = vec![TextOperand("hello")];
    let expected_list = matchers::collection::ContainsAll::new(&operands);
    assert_that!([String::from("hello")])
        .into_iter_contains_all(operands.as_slice())
        .matches(&expected_list);
    assert_that!(vec![String::from("hello")]).matches(&expected_list);
    let keys = vec![TextOperand("key")];
    let entries = vec![(TextOperand("key"), TextOperand("hello"))];
    let actual = BTreeMap::from([(String::from("key"), String::from("hello"))]);
    assert_that!(actual)
        .contains_keys(&keys)
        .matches(matchers::map::ContainsExactlyEntries::new(
            entries.as_slice(),
        ));
}

#[allow(dead_code)]
#[cfg_attr(test, test)]
fn borrowed_views_compile_without_std() {
    use alloc::{collections::BTreeMap, string::String, vec};

    use assertr::matchers::{
        EqualTo, all_of, dereferenced, each,
        range::{ContainsElement, DoesNotContainElement},
    };

    // Native map lookup accepts both built-in slice views and downstream text views.
    let bytes = BTreeMap::from([(vec![1_u8, 2], 3)]);
    let query = &[1_u8, 2][..];
    assert_that!(bytes)
        .contains_key(query)
        .matches(assertr::entries_are![(query, matchers::eq(3))]);
    assert_that!(BTreeMap::from([(String::from("key"), 1)]))
        .matches(entry(TextOperand("key"), matchers::eq(1)));

    reusable_bulk_views_compile_without_std();

    // Cover borrowed and unsized operands across collection, iterator, map, and ordering bounds.
    let expected = String::from("hello");
    assert_that!(String::from("hello")).is_less_or_equal_to(&expected);
    assert_that!([String::from("hello")])
        .contains("hello")
        .contains_exactly([&expected]);
    assert_that_owned!([String::from("hello")].into_iter()).contains(&expected);
    assert_that!(BTreeMap::from([(1, String::from("hello"))])).contains_entry(&1, "hello");

    // Range bounds can be owned or borrowed.
    let lower = String::from("a");
    let upper = String::from("z");
    assert_that!(&lower..&upper).contains_element(&expected);
    let range_matcher = all_of(matchers![
        ContainsElement::<String>::borrowing(&expected),
        DoesNotContainElement::<String>::borrowing(&upper)
    ]);
    assert_that!(String::from("a")..String::from("z")).matches(&range_matcher);
    assert_that!([&lower..&upper]).matches(each(&range_matcher));
    assert_that!(&1..&3).matches(ContainsElement::new(&2));

    // Reusable matchers borrow non-Copy operands and accept sized or unsized subjects.
    assert_that_owned!(&expected).matches(dereferenced(EqualTo::new("hello")));
    let matcher = EqualTo::new(&expected);
    assert_that!(String::from("hello")).matches(&matcher);
    let literal_matcher = EqualTo::new("hello");
    assert_that!(expected).matches(&literal_matcher);
    assert_that!("hello").matches(&literal_matcher);

    let expected_values = vec![1, 2];
    assert_that!(expected_values).is_equal_to([1, 2]);
    assert_that!([1, 2].as_slice())
        .is_equal_to(vec![1, 2])
        .is_equal_to(&expected_values)
        .matches(dereferenced(EqualTo::new(&expected_values)));

    assert_that!(expected).is_equal_to(TextOperand("hello"));
    #[cfg(feature = "partial")]
    {
        struct Message {
            text: String,
        }
        assert_that!(Message {
            text: expected.clone()
        })
        .matches(assertr::partial!(Message {
            text: EqualTo::new("hello")
        }));
    }
    #[cfg(feature = "num")]
    {
        let value = 10;
        let deviation = 2;
        assert_that!(11).is_close_to(&value, &deviation);
    }
}
