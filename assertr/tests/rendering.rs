use assertr::prelude::*;

#[derive(PartialEq)]
struct Secret(u32);

#[test]
fn non_debug_type_can_use_debug_format_closure() {
    let failures = assert_that!(Secret(1))
        .with_debug_format(|value, f| f.write_fmt(format_args!("Secret({})", value.0)))
        .with_capture()
        .with_location(false)
        .is_equal_to(Secret(2))
        .capture_failures();

    assert_that!(failures.as_slice()).contains_exactly([indoc::formatdoc! {"
        -------- assertr --------
        Expected: Secret(2)

          Actual: Secret(1)
        -------- assertr --------
    "}]);
}

#[test]
fn debug_format_closure_works_in_derived_chain() {
    // Regression: `with_debug_format` produces `CustomRenderer<F>`, which must be `Clone`
    // for any assertion that derives a child `AssertThat` (here, `derive`).
    assert_that!(Secret(7))
        .with_debug_format(|value: &Secret, f| f.write_fmt(format_args!("Secret({})", value.0)))
        .derive(|s| s.0 == 7)
        .is_true();
}

#[derive(Clone, Copy)]
struct SecretAndU32Renderer;

impl AssertionRenderer<Secret> for SecretAndU32Renderer {
    fn fmt(&self, value: &Secret, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Secret({})", value.0)
    }
}

impl AssertionRenderer<u32> for SecretAndU32Renderer {
    fn fmt(&self, value: &u32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "u32({value})")
    }
}

#[test]
fn custom_renderer_threads_through_satisfies() {
    // `satisfies` requires `R: Clone` and propagates the renderer to the child `AssertThat`.
    // The custom renderer must therefore stay live for the inner closure's assertions.
    assert_that!(Secret(7))
        .with_renderer(SecretAndU32Renderer)
        .satisfies(
            |s| s.0,
            |inner| {
                inner.is_equal_to(7u32);
            },
        );
}

#[test]
fn custom_renderer_renders_failures_inside_satisfies() {
    let failures = assert_that!(Secret(1))
        .with_renderer(SecretAndU32Renderer)
        .with_capture()
        .with_location(false)
        .satisfies(
            |s| s.0,
            |inner| {
                inner.is_equal_to(2u32);
            },
        )
        .capture_failures();

    assert_that!(failures.as_slice()).contains_exactly([indoc::formatdoc! {"
        -------- assertr --------
        Expected: u32(2)

          Actual: u32(1)
        -------- assertr --------
    "}]);
}

struct Actual(u32);
struct Expected(u32);

impl PartialEq<Expected> for Actual {
    fn eq(&self, other: &Expected) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Copy)]
struct NamedRenderer;

impl AssertionRenderer<Actual> for NamedRenderer {
    fn fmt(&self, value: &Actual, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Actual({})", value.0))
    }
}

impl AssertionRenderer<Expected> for NamedRenderer {
    fn fmt(&self, value: &Expected, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Expected({})", value.0))
    }
}

#[test]
fn named_renderer_can_render_heterogeneous_comparisons() {
    let failures = assert_that!(Actual(1))
        .with_renderer(NamedRenderer)
        .with_capture()
        .with_location(false)
        .is_equal_to(Expected(2))
        .capture_failures();

    assert_that!(failures.as_slice()).contains_exactly([indoc::formatdoc! {"
        -------- assertr --------
        Expected: Expected(2)
        
          Actual: Actual(1)
        -------- assertr --------
    "}]);
}

mod collection_renderer_equality {
    use super::*;
    use assertr::{AssertrPartialEq, EqContext};
    use std::collections::VecDeque;

    #[derive(Clone, Copy)]
    struct CollectionActual(u32);

    #[derive(Clone, Copy)]
    struct CollectionExpected(u32);

    #[derive(Clone, Copy)]
    struct CollectionRenderer;

    fn fmt_actuals<'a>(
        values: impl IntoIterator<Item = &'a CollectionActual>,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str("[")?;
        for (index, value) in values.into_iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            f.write_fmt(format_args!("Actual({})", value.0))?;
        }
        f.write_str("]")
    }

    fn fmt_expecteds<'a>(
        values: impl IntoIterator<Item = &'a CollectionExpected>,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str("[")?;
        for (index, value) in values.into_iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            f.write_fmt(format_args!("Expected({})", value.0))?;
        }
        f.write_str("]")
    }

    impl AssertrPartialEq<CollectionExpected, CollectionRenderer> for CollectionActual {
        fn eq(
            &self,
            other: &CollectionExpected,
            _ctx: Option<&mut EqContext<'_, CollectionRenderer>>,
        ) -> bool {
            self.0 == other.0
        }
    }

    impl AssertionRenderer<CollectionActual> for CollectionRenderer {
        fn fmt(
            &self,
            value: &CollectionActual,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            f.write_fmt(format_args!("Actual({})", value.0))
        }
    }

    impl AssertionRenderer<CollectionExpected> for CollectionRenderer {
        fn fmt(
            &self,
            value: &CollectionExpected,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            f.write_fmt(format_args!("Expected({})", value.0))
        }
    }

    impl AssertionRenderer<[CollectionActual]> for CollectionRenderer {
        fn fmt(
            &self,
            values: &[CollectionActual],
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            fmt_actuals(values, f)
        }
    }

    impl AssertionRenderer<[CollectionExpected]> for CollectionRenderer {
        fn fmt(
            &self,
            values: &[CollectionExpected],
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            fmt_expecteds(values, f)
        }
    }

    impl AssertionRenderer<Vec<CollectionActual>> for CollectionRenderer {
        fn fmt(
            &self,
            values: &Vec<CollectionActual>,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            <Self as AssertionRenderer<[CollectionActual]>>::fmt(self, values.as_slice(), f)
        }
    }

    impl<'a> AssertionRenderer<Vec<&'a CollectionActual>> for CollectionRenderer {
        fn fmt(
            &self,
            values: &Vec<&'a CollectionActual>,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            fmt_actuals(values.iter().copied(), f)
        }
    }

    impl<'a> AssertionRenderer<Vec<&'a CollectionExpected>> for CollectionRenderer {
        fn fmt(
            &self,
            values: &Vec<&'a CollectionExpected>,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            fmt_expecteds(values.iter().copied(), f)
        }
    }

    impl AssertionRenderer<VecDeque<CollectionActual>> for CollectionRenderer {
        fn fmt(
            &self,
            values: &VecDeque<CollectionActual>,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            fmt_actuals(values, f)
        }
    }

    #[test]
    fn slice_membership_uses_renderer_specific_equality() {
        let actual = [CollectionActual(1), CollectionActual(2)];

        assert_that!(actual.as_slice())
            .with_renderer(CollectionRenderer)
            .contains(CollectionExpected(2));
    }

    #[test]
    fn exact_slice_comparison_uses_renderer_specific_equality() {
        let actual = [CollectionActual(1), CollectionActual(2)];

        assert_that!(actual.as_slice())
            .with_renderer(CollectionRenderer)
            .contains_exactly([CollectionExpected(1), CollectionExpected(2)]);
    }

    #[test]
    fn iterator_membership_uses_renderer_specific_equality() {
        assert_that!(vec![CollectionActual(1), CollectionActual(2)].into_iter())
            .with_renderer(CollectionRenderer)
            .contains(CollectionExpected(2));
        assert_that!(vec![CollectionActual(1), CollectionActual(2)].into_iter())
            .with_renderer(CollectionRenderer)
            .contains_exactly([CollectionExpected(1), CollectionExpected(2)]);
    }

    #[test]
    fn into_iterator_membership_uses_renderer_specific_equality() {
        assert_that!(vec![CollectionActual(1), CollectionActual(2)])
            .with_renderer(CollectionRenderer)
            .into_iter_contains(CollectionExpected(2));
    }

    #[test]
    fn vec_deque_comparison_uses_renderer_specific_equality() {
        assert_that!(VecDeque::from([CollectionActual(1), CollectionActual(2),]))
            .with_renderer(CollectionRenderer)
            .contains(CollectionExpected(2));
    }
}

mod wrapper_renderer {
    use super::*;
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;

    #[derive(PartialEq, Eq, Hash)]
    struct Secret(u32);

    #[derive(Clone, Copy)]
    struct SecretRenderer;

    impl AssertionRenderer<Secret> for SecretRenderer {
        fn fmt(&self, value: &Secret, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("Secret({})", value.0))
        }
    }

    impl AssertionRenderer<RefCell<Secret>> for SecretRenderer {
        fn fmt(
            &self,
            value: &RefCell<Secret>,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            match value.try_borrow() {
                Ok(inner) => f.write_fmt(format_args!("RefCell({})", inner.0)),
                Err(_) => f.write_str("RefCell(<borrowed>)"),
            }
        }
    }

    impl AssertionRenderer<HashSet<Secret>> for SecretRenderer {
        fn fmt(
            &self,
            value: &HashSet<Secret>,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            let mut entries: std::vec::Vec<u32> = value.iter().map(|s| s.0).collect();
            entries.sort_unstable();
            f.write_str("{")?;
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?;
                }
                f.write_fmt(format_args!("Secret({entry})"))?;
            }
            f.write_str("}")
        }
    }

    impl AssertionRenderer<HashMap<&'static str, Secret>> for SecretRenderer {
        fn fmt(
            &self,
            value: &HashMap<&'static str, Secret>,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            let mut entries: std::vec::Vec<(&&'static str, &Secret)> = value.iter().collect();
            entries.sort_by_key(|(k, _)| **k);
            f.write_str("{")?;
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?;
                }
                f.write_fmt(format_args!("{:?}: Secret({})", k, v.0))?;
            }
            f.write_str("}")
        }
    }

    impl AssertionRenderer<&'static str> for SecretRenderer {
        fn fmt(&self, value: &&'static str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("{value:?}"))
        }
    }

    impl AssertionRenderer<Mutex<Secret>> for SecretRenderer {
        fn fmt(
            &self,
            _value: &Mutex<Secret>,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            f.write_str("Mutex<Secret>")
        }
    }

    #[test]
    fn ref_cell_assertion_uses_custom_renderer() {
        let cell = RefCell::new(Secret(7));
        let failures = assert_that!(&cell)
            .with_renderer(SecretRenderer)
            .with_capture()
            .with_location(false)
            .is_borrowed()
            .capture_failures();

        assert_that!(failures.as_slice()).contains_exactly([indoc::formatdoc! {"
            -------- assertr --------
            Actual: RefCell(7) is not borrowed.

            Expected: RefCell to be borrowed (immutably) at least once.
            -------- assertr --------
        "}]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hashset_assertion_uses_custom_renderer() {
        let actual: HashSet<Secret> = HashSet::from([Secret(1)]);
        let failures = assert_that!(actual)
            .with_renderer(SecretRenderer)
            .with_capture()
            .with_location(false)
            .contains(Secret(2))
            .capture_failures();

        assert_that!(failures.as_slice()).contains_exactly([indoc::formatdoc! {"
            -------- assertr --------
            Actual: HashSet {{Secret(1)}}

            does not contain expected: Secret(2)
            -------- assertr --------
        "}]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hashmap_contains_value_uses_custom_renderer() {
        let mut map: HashMap<&'static str, Secret> = HashMap::new();
        map.insert("alpha", Secret(1));

        let failures = assert_that!(map)
            .with_renderer(SecretRenderer)
            .with_capture()
            .with_location(false)
            .contains_value(Secret(2))
            .capture_failures();

        assert_that!(failures.as_slice()).contains_exactly([indoc::formatdoc! {r#"
            -------- assertr --------
            Actual: HashMap {{"alpha": Secret(1)}}

            does not contain expected value: Secret(2)
            -------- assertr --------
        "#}]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn mutex_is_locked_uses_custom_renderer() {
        let mutex = Mutex::new(Secret(11));
        let failures = assert_that!(mutex)
            .with_renderer(SecretRenderer)
            .with_capture()
            .with_location(false)
            .is_locked()
            .capture_failures();

        assert_that!(failures.as_slice()).contains_exactly([indoc::formatdoc! {"
            -------- assertr --------
            Expected: Mutex {{ data: Secret(11), poisoned: false }}

            to be locked, but it wasn't!
            -------- assertr --------
        "}]);
    }
}

#[cfg(feature = "derive")]
mod derive {
    use super::*;
    use std::collections::HashMap;

    #[derive(PartialEq)]
    pub struct Hidden(u32);

    #[derive(AssertrEq)]
    pub struct Subject {
        pub hidden: Hidden,
    }

    impl AssertionRenderer<Subject> for NamedRenderer {
        fn fmt(&self, value: &Subject, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("Subject({})", value.hidden.0))
        }
    }

    impl AssertionRenderer<SubjectAssertrEq> for NamedRenderer {
        fn fmt(
            &self,
            _value: &SubjectAssertrEq,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            f.write_str("SubjectAssertrEq(..)")
        }
    }

    impl AssertionRenderer<Hidden> for NamedRenderer {
        fn fmt(&self, value: &Hidden, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("Hidden({})", value.0))
        }
    }

    #[test]
    fn derive_reports_differences_for_non_debug_fields_with_renderer() {
        let failures = assert_that!(Subject { hidden: Hidden(1) })
            .with_renderer(NamedRenderer)
            .with_capture()
            .with_location(false)
            .is_equal_to(SubjectAssertrEq {
                hidden: eq(Hidden(2)),
            })
            .capture_failures();

        assert_that!(failures.as_slice()).contains_exactly([indoc::formatdoc! {"
            -------- assertr --------
            Expected: SubjectAssertrEq(..)
            
              Actual: Subject(1)

            Details: [
                Differences: [
                    \"hidden\": expected Hidden(2), but was Hidden(1),
                ],
            ]
            -------- assertr --------
        "}]);
    }

    #[derive(Debug, AssertrEq)]
    pub struct Child {
        pub id: i32,
    }

    #[derive(Debug, AssertrEq)]
    pub struct NestedParent {
        #[assertr_eq(map_type = "ChildAssertrEq")]
        pub child: Child,
    }

    #[test]
    fn debug_renderer_renders_nested_generated_matchers() {
        let failures = assert_that!(NestedParent {
            child: Child { id: 1 },
        })
        .with_capture()
        .with_location(false)
        .is_equal_to(NestedParentAssertrEq {
            child: eq(ChildAssertrEq { id: eq(2) }),
        })
        .capture_failures();

        assert_that!(failures[0].as_str()).contains(indoc::indoc! {r"
            Expected: NestedParentAssertrEq {
                child: Eq::Eq(ChildAssertrEq {
                    id: Eq::Eq(2),
                }),
            }
        "});
    }

    #[derive(Debug, AssertrEq)]
    pub struct VecParent {
        #[assertr_eq(
            map_type = "Vec<ChildAssertrEq>",
            compare_with = "::assertr::cmp::slice::compare",
            compare_bounds = "Child: ::assertr::cmp::slice::CompareElement<ChildAssertrEq, R>"
        )]
        pub children: Vec<Child>,
    }

    #[test]
    fn debug_renderer_renders_vec_of_generated_matchers() {
        let failures = assert_that!(VecParent {
            children: vec![Child { id: 1 }],
        })
        .with_capture()
        .with_location(false)
        .is_equal_to(VecParentAssertrEq {
            children: eq(vec![ChildAssertrEq { id: eq(2) }]),
        })
        .capture_failures();

        assert_that!(failures[0].as_str()).contains(indoc::indoc! {r"
            Expected: VecParentAssertrEq {
                children: Eq::Eq([
                    ChildAssertrEq {
                        id: Eq::Eq(2),
                    },
                ]),
            }
        "});
    }

    #[derive(Debug, AssertrEq)]
    pub struct MapParent {
        #[assertr_eq(
            map_type = "HashMap<String, ChildAssertrEq>",
            compare_with = "::assertr::cmp::hashmap::compare",
            compare_bounds = "Child: ::assertr::cmp::hashmap::CompareValue<ChildAssertrEq, R>"
        )]
        pub children: HashMap<String, Child>,
    }

    #[test]
    fn debug_renderer_renders_hashmap_of_generated_matchers() {
        let failures = assert_that!(MapParent {
            children: HashMap::from([("first".to_string(), Child { id: 1 })]),
        })
        .with_capture()
        .with_location(false)
        .is_equal_to(MapParentAssertrEq {
            children: eq(HashMap::from([(
                "first".to_string(),
                ChildAssertrEq { id: eq(2) },
            )])),
        })
        .capture_failures();

        assert_that!(failures[0].as_str()).contains(indoc::indoc! {r#"
            Expected: MapParentAssertrEq {
                children: Eq::Eq({
                    "first": ChildAssertrEq {
                        id: Eq::Eq(2),
                    },
                }),
            }
        "#});
    }
}
