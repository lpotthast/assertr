#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use assertr::matchers::{entry_matchers, predicate};
use assertr::prelude::*;

#[allow(dead_code)]
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
    assert_eq!(computed.actual().byte, 3);
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

    let fact = assertr::Fact::note("evidence");
    assert_that!(fact)
        .with_renderer(NoRenderer)
        .derive(assertr::Fact::value)
        .derive(assertr::renderer::Rendered::body)
        .is_same_instance_as(fact.value().body());
}

#[cfg(test)]
#[test]
fn projections_run_without_std() {
    projections_compile_without_renderer_support();
}

#[allow(dead_code)]
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
    assert!(failures.is_empty());
}

#[cfg(test)]
#[test]
fn unwind_safe_projections_run_without_std() {
    unwind_safe_projections_compile_without_std();
    let failures = assert_that!(1).capture(|root| {
        let result = std::panic::catch_unwind(|| {
            root.derive(|value| value).is_equal_to(2);
            panic!("after recording a failure");
        });
        assert!(result.is_err());
        root.is_equal_to(3)
    });
    assert_eq!(failures.len(), 2);
}

#[cfg(feature = "num")]
#[allow(dead_code)]
fn numeric_assertions_compile_without_std() {
    use assertr::assertions::num::NumericDistance;

    fn assert_close<T: NumericDistance + core::fmt::Debug>(actual: T, expected: T, deviation: T) {
        assert_that_owned!(actual).is_close_to(expected, deviation);
    }

    assert_close(i128::MIN, -1, i128::MAX);
    assert_close(0_u128, u128::MAX, u128::MAX);
    assert_close(-1.0_f32, 16_777_216.0, 16_777_216.0);
    assert_close(f64::INFINITY, f64::INFINITY, 0.0);
    let failures = assert_that!(9_007_199_254_740_992_f64)
        .capture(|it| it.is_close_to(9_007_199_254_740_994.0, 1.0));
    assert_that!(failures).has_length(1);
}

#[cfg(all(test, feature = "num"))]
#[test]
fn numeric_assertions_run_without_std_or_libm() {
    numeric_assertions_compile_without_std();
}

#[allow(dead_code)]
fn sensitive_value_policy_compiles_without_std() {
    use assertr::renderer::{RenderingContext, SensitiveValuePolicy};
    use core::fmt;

    struct BorrowedRenderer<'a>(&'a str);

    impl ValueRenderer<str> for BorrowedRenderer<'_> {
        fn fmt(&self, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}{value}", self.0)
        }

        fn sensitive_value_policy(&self) -> SensitiveValuePolicy {
            SensitiveValuePolicy::Reveal
        }
    }

    let prefix = alloc::string::String::from("value: ");
    let renderer = BorrowedRenderer(&prefix);
    let erased: &dyn ValueRenderer<str> = &renderer;
    assert_eq!(
        erased.sensitive_value_policy(),
        SensitiveValuePolicy::Reveal
    );
    let value = RenderingContext::new(&renderer, RenderingBudget::unlimited()).value("original");
    assert_eq!(alloc::format!("{value:?}"), "value: original");
}

#[allow(dead_code)]
fn capture_errors_compile_without_std()
-> Result<(), alloc::boxed::Box<dyn core::error::Error + Send + Sync>> {
    use alloc::string::ToString;
    let failures = assert_that_owned!(0..).capture(|it| it.starts_with([1, 2]));
    let _report = failures.to_string();
    let single = assert_that!(failures).get_single().actual().clone();
    let _: alloc::boxed::Box<dyn core::error::Error + Send + Sync> = single.into();
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
fn failure_adapters_compile_without_std() {
    use alloc::string::{String, ToString};
    use core::convert::Infallible;

    use assertr::failure::adapter::{Adapter, AdapterExt, HumanReadableText, ToHumanReadableText};

    struct Sink;

    impl Adapter<HumanReadableText> for Sink {
        type Output = ();
        type Error = Infallible;

        fn adapt(&self, _input: &HumanReadableText) -> Result<Self::Output, Self::Error> {
            Ok(())
        }
    }

    fn accepts_failure_adapter<A: Adapter<assertr::AssertionFailure>>(_adapter: A) {}

    struct NoRenderer;

    accepts_failure_adapter(ToHumanReadableText.then(Sink));
    let _assertion = assert_that!(1)
        .with_renderer(NoRenderer)
        .with_panic_presentation(ToHumanReadableText);
    let presentation = ToHumanReadableText.map_err(|error: Infallible| error.to_string());
    let adapter: &dyn Adapter<assertr::AssertionFailure, Output = HumanReadableText, Error = String> =
        &presentation;
    accepts_failure_adapter(adapter);
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

/// The set and map families live outside the `std` module, so `BTreeSet` and `BTreeMap` carry them
/// into `no_std` builds.
#[allow(dead_code)]
fn set_and_map_assertions_compile_without_std() {
    use alloc::collections::{BTreeMap, BTreeSet};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_one(value: &i32) -> bool {
        *value == 1
    }

    fn satisfies_one(it: AssertThat<i32, Capture>) {
        it.is_equal_to(1);
    }

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
        .contains_entry::<i32, _>("a", 1)
        .contains_entry_satisfying("a", satisfies_one)
        .contains_keys(["a"])
        .contains_exactly_entries([("a", 1)])
        .contains_exactly_entries_matching(entry_matchers([("a", predicate(is_one))]))
        .contains_exactly_entries_satisfying([("a", satisfies_one)])
        .has_length(1);
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
    use crate::NumericRenderer;
    #[test]
    fn borrowed_renderers_support_sensitivity_policy_without_std() {
        crate::sensitive_value_policy_compiles_without_std();
    }

    mod matchers {
        use assertr::matchers::ge;
        use assertr::prelude::*;

        #[cfg(feature = "matchers")]
        #[test]
        fn structural_matchers_work_with_alloc() {
            crate::structural_matchers_without_std();
        }

        #[test]
        fn runtime_matchers_need_no_features() {
            assert_that!([1, 2]).matches(elements_are![1, 2]);
            assert_that!(3).matches(ge(2));
        }
    }

    use alloc::{rc::Rc, string::String};
    use core::{
        convert::Infallible,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use assertr::prelude::{
        BoolAssertions, CollectionAssertions, IdentityAssertions, IteratorAssertions,
        LengthAssertions, PartialEqAssertions, StableOrderAssertions,
    };
    use assertr::{
        AssertionFailure,
        failure::adapter::{Adapter, HumanReadableText, ToHumanReadableText},
    };

    struct CountsPresentations(Rc<AtomicUsize>);

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
        assert_eq!(failures.len(), 2);
        let failures = assertr::assert_that!([&keys[0], &keys[1]])
            .with_renderer(NumericRenderer)
            .capture(|it| {
                it.contains_same_instance_as(&keys[2])
                    .does_not_contain_same_instance_as(&keys[0])
                    .contains_exactly_same_instances([&keys[1], &keys[0]])
                    .contains_exactly_same_instances_in_any_order([&keys[0], &keys[0]])
            });
        assert_eq!(failures.len(), 4);
        let panic = std::panic::catch_unwind(|| {
            assertr::assert_that!(keys[0])
                .with_renderer(NoRenderer)
                .is_same_instance_as(&keys[1]);
        })
        .unwrap_err();
        assert!(
            panic
                .downcast_ref::<String>()
                .unwrap()
                .contains("is not the same instance as")
        );
    }

    impl Adapter<AssertionFailure> for CountsPresentations {
        type Output = HumanReadableText;
        type Error = Infallible;

        fn adapt(&self, failure: &AssertionFailure) -> Result<Self::Output, Self::Error> {
            self.0.fetch_add(1, Ordering::Relaxed);
            ToHumanReadableText.adapt(failure)
        }
    }

    #[test]
    fn a_non_sync_presentation_runs_only_in_panic_mode_without_std() {
        let count = Rc::new(AtomicUsize::new(0));
        let failures = assertr::assert_that!(1)
            .with_panic_presentation(CountsPresentations(Rc::clone(&count)))
            .capture(|it| it.is_equal_to(2));
        assert_eq!(failures.len(), 1);
        assert_eq!(count.load(Ordering::Relaxed), 0);

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assertr::assert_that!(1)
                .with_panic_presentation(CountsPresentations(Rc::clone(&count)))
                .is_equal_to(2);
        }))
        .unwrap_err();

        assert!(
            panic
                .downcast_ref::<String>()
                .unwrap()
                .contains("Expected: 2\n\n  Actual: 1")
        );
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn a_presentation_error_falls_back_without_std() {
        struct ReturnsError;

        impl Adapter<AssertionFailure> for ReturnsError {
            type Output = HumanReadableText;
            type Error = &'static str;

            fn adapt(&self, _: &AssertionFailure) -> Result<HumanReadableText, &'static str> {
                Err("presentation unavailable")
            }
        }

        let panic = std::panic::catch_unwind(|| {
            assertr::assert_that!(1)
                .with_panic_presentation(ReturnsError)
                .is_equal_to(2);
        })
        .unwrap_err();
        let message = panic.downcast_ref::<String>().unwrap();
        assert!(message.contains("Expected: 2\n\n  Actual: 1"));
        assert!(
            message
                .contains("The failure presentation returned an error: presentation unavailable")
        );
    }

    #[test]
    fn streaming_iterator_assertions_run_in_the_hosted_no_std_fixture() {
        let failures = assertr::assert_that_owned!(0..20).capture(|it| it.does_not_contain(19));

        assertr::assert_that!(failures).has_length(1);
    }

    #[test]
    fn dropping_an_unused_assertion_does_not_panic() {
        let result = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42);
        });

        assertr::assert_that!(result.is_ok()).is_true();
    }

    #[test]
    fn dropping_an_unused_assertion_during_unwinding_preserves_the_original_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42);
            panic!("original panic");
        })
        .expect_err("the closure should panic");

        assertr::assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(&"original panic"));
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

/// Structural matching remains available with alloc and no std.
#[cfg(feature = "matchers")]
pub fn structural_matchers_without_std() {
    use assertr::prelude::*;
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
    assert_that!(children).matches(elements_are![partial!(Child { id: 1, .. })]);
    assert_that!(children)
        .with_renderer(NumericRenderer)
        .into_iter_contains_matching(partial!(Child {
            id: assertr::matchers::anything(),
            ..
        }));
    let map = alloc::collections::BTreeMap::from([(
        "child",
        Child {
            id: 1,
            hidden: Hidden,
        },
    )]);
    assert_that!(map).matches(entries_are![("child", partial!(Child { id: 1, .. }))]);
}

struct NumericRenderer;
impl assertr::ValueRenderer<usize> for NumericRenderer {
    fn fmt(&self, value: &usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(value, f)
    }
}

#[allow(dead_code)]
fn typed_condition_and_numeric_evidence_compile_without_std() {
    struct OpaqueError(u32);
    struct Reject;
    impl AssertrCondition<u32> for Reject {
        type Error = OpaqueError;
        fn test(&self, value: &u32) -> Result<(), OpaqueError> {
            Err(OpaqueError(*value))
        }
    }
    #[derive(Clone, Copy)]
    struct ErrorRenderer;
    impl ValueRenderer<OpaqueError> for ErrorRenderer {
        fn fmt(&self, error: &OpaqueError, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "error({})", error.0)
        }
    }
    fn condition_trait<A: ConditionAssertions<u32, ErrorRenderer>>() {}
    fn iterable_trait<A: IterableConditionAssertions<u32, [u32; 1], ErrorRenderer>>() {}
    fn iterator_trait<A: ExactSizeIteratorAssertions<NumericRenderer>>() {}
    condition_trait::<AssertThat<'static, u32, Panic, ErrorRenderer>>();
    iterable_trait::<AssertThat<'static, [u32; 1], Panic, ErrorRenderer>>();
    iterator_trait::<AssertThat<'static, core::array::IntoIter<u32, 1>, Panic, NumericRenderer>>();
    let failures = assert_that!(7_u32)
        .with_renderer(ErrorRenderer)
        .capture(|it| it.is(Reject));
    assert_eq!(
        failures[0].facts[0].value.type_name,
        Some(core::any::type_name::<OpaqueError>())
    );
    let failures = assert_that!([7_u32])
        .with_renderer(ErrorRenderer)
        .capture(|it| it.are(Reject));
    assert_eq!(failures.len(), 1);
    let failures = assert_that!(7_u32)
        .with_renderer(ErrorRenderer)
        .capture(|it| it.matches(assertr::matchers::condition(Reject)));
    assert_eq!(failures.len(), 1);
    let failures = assert_that!([7_u32].into_iter())
        .with_renderer(NumericRenderer)
        .capture(|it| it.has_remaining_count(2));
    assert_eq!(
        failures[0].expected.as_ref().unwrap().type_name,
        Some("usize")
    );
}

#[cfg(test)]
#[test]
fn typed_condition_and_numeric_evidence_run_without_std() {
    typed_condition_and_numeric_evidence_compile_without_std();
}
