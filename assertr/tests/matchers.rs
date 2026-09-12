//! Structural macro behavior at a downstream call site.
#![cfg(feature = "matchers")]

mod catalog {
    use assertr::{matchers::*, prelude::*};

    #[test]
    fn family_namespaces_compose_as_structural_fields() {
        struct User {
            name: &'static str,
            roles: [&'static str; 2],
            id: Option<u32>,
        }
        let expected = partial!(User {
            name: string::Contains::new("da"),
            roles: all_of((collection::Contains::new("reader"), HasLengthOf::new(2))),
            id: IsSome,
        });
        assert_that!(User {
            name: "Ada",
            roles: ["reader", "editor"],
            id: Some(1),
        })
        .matches(&expected);
    }
}

mod named_fields {
    use assertr::{matchers::eq, prelude::*};

    struct Secret;

    #[allow(dead_code)]
    struct Child {
        id: u32,
        secret: Secret,
    }

    #[allow(dead_code)]
    struct Parent {
        children: Vec<Child>,
        secret: Secret,
    }

    struct Scalar;

    impl ValueRenderer<usize> for Scalar {
        fn fmt(&self, v: &usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Debug::fmt(v, f)
        }
    }
    impl ValueRenderer<u32> for Scalar {
        fn fmt(&self, x: &u32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "id={x}")
        }
    }

    #[test]
    fn nested_omitted_fields_and_renderer_reuse() {
        let parent = Parent {
            children: vec![
                Child {
                    id: 1,
                    secret: Secret,
                },
                Child {
                    id: 2,
                    secret: Secret,
                },
            ],
            secret: Secret,
        };
        let matcher = partial!(Parent {
            children: elements_are![
                partial!(Child { id: eq(1), .. }),
                partial!(Child { id: eq(2), .. })
            ],
            ..
        });
        assert_that!(parent).matches(&matcher);
        assert_that!(parent).with_renderer(Scalar).matches(&matcher);
        let failures = assert_that!(parent).with_renderer(Scalar).capture(|it| {
            it.matches(partial!(Parent {
                children: elements_are![
                    partial!(Child { id: eq(3), .. }),
                    partial!(Child { id: eq(4), .. })
                ],
                ..
            }))
        });
        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive(|value| &value.children[1].path)
                    .is_equal_to(vec![
                        assertr::failure::PathSegment::Field("children"),
                        assertr::failure::PathSegment::Index(1),
                        assertr::failure::PathSegment::Field("id"),
                    ]);
            },
        ]);
        let failures = assert_that!(parent).with_renderer(Scalar).capture(|it| {
            it.matches(partial!(Parent {
                children: elements_are_in_any_order![partial!(Child { id: eq(5), .. })],
                ..
            }))
        });
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn expected_expressions_are_constructed_once_in_source_order() {
        struct Pair {
            x: i32,
            y: i32,
        }
        let calls = std::cell::RefCell::new(Vec::new());
        let matcher = partial!(Pair {
            y: {
                calls.borrow_mut().push("y");
                eq(2)
            },
            x: {
                calls.borrow_mut().push("x");
                eq(1)
            },
        });
        assert_that!(*calls.borrow()).is_equal_to(["y", "x"]);
        assert_that!(Pair { x: 1, y: 2 }).matches(&matcher);
        assert_that!(Pair { x: 1, y: 2 }).matches(&matcher);
        assert_that!(*calls.borrow()).is_equal_to(["y", "x"]);
    }
}

mod maps {
    use assertr::{matchers::eq, prelude::*};

    #[test]
    fn nested_maps_and_borrowed_values() {
        use std::collections::BTreeMap;

        struct Row<'a> {
            name: &'a str,
        }
        let name = String::from("Alice");
        let map = BTreeMap::from([(String::from("a"), Row { name: &name })]);
        assert_that!(map).matches(entries_are![("a", partial!(Row { name: eq("Alice") }))]);
    }

    #[test]
    fn nested_map_paths_compose_once() {
        use assertr::failure::PathSegment;

        struct Item {
            id: u32,
        }

        struct Root {
            items: std::collections::BTreeMap<&'static str, Item>,
        }
        let root = Root {
            items: std::collections::BTreeMap::from([("x", Item { id: 1 })]),
        };
        let failures = assert_that!(root).capture(|it| {
            it.matches(partial!(Root {
                items: entries_are![("x", partial!(Item { id: eq(2) }))]
            }))
        });
        assert_that!(failures[0].children[0].path).contains_exactly_satisfying([
            |segment: AssertThat<PathSegment, Capture>| {
                segment.is_equal_to(PathSegment::Field("items"));
            },
            |segment: AssertThat<PathSegment, Capture>| {
                segment.is_matching(pattern!(PathSegment::Key(_)));
            },
            |segment: AssertThat<PathSegment, Capture>| {
                segment.is_equal_to(PathSegment::Field("id"));
            },
        ]);
    }
}

mod tuple_structs {
    use assertr::{matchers::eq, prelude::*};

    #[allow(dead_code)]
    struct Pair(i32, i32);

    #[test]
    fn accepts_wildcard_fields() {
        assert_that!(Pair(1, 2)).matches(partial!(Pair(eq(1), _)));
    }

    #[test]
    fn accepts_final_rest() {
        assert_that!(Pair(1, 2)).matches(partial!(Pair(eq(1), ..)));
    }
}

mod unit_structs {
    use assertr::prelude::*;

    #[test]
    fn requires_no_renderer() {
        struct Unit;

        struct NoRenderer;

        assert_that!(Unit)
            .with_renderer(NoRenderer)
            .matches(partial!(Unit));
    }
}

mod enum_variants {
    use assertr::{
        failure::PathSegment,
        matchers::{anything, eq},
        prelude::*,
    };

    enum Example {
        Named { value: i32 },
        Tuple(i32),
        Unit,
    }

    #[test]
    fn explicit_variant_marker_records_a_typed_path() {
        let failures = assert_that!(Example::Tuple(2))
            .capture(|it| it.matches(partial!(variant Example::Tuple(eq(3)))));

        assert_that!(failures[0].children[0].path).is_equal_to([
            PathSegment::Variant("Example::Tuple"),
            PathSegment::TupleIndex(0),
        ]);
    }

    #[test]
    fn mismatched_variants_require_no_renderer() {
        struct NoRenderer;

        let failures = assert_that!(Example::Unit)
            .with_renderer(NoRenderer)
            .capture(|it| it.matches(partial!(Example::Tuple(anything()))));
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn accepts_named_variants() {
        assert_that!(Example::Named { value: 1 })
            .matches(partial!(Example::Named { value: eq(1) }));
    }

    #[test]
    fn accepts_tuple_variants() {
        assert_that!(Example::Tuple(2)).matches(partial!(Example::Tuple(eq(2))));
    }

    #[test]
    fn accepts_unit_variants() {
        assert_that!(Example::Unit).matches(partial!(Example::Unit));
    }

    #[test]
    fn accepts_imported_unit_variants() {
        use Example::Unit;

        assert_that!(Unit).matches(partial!(Unit));
        assert_that!(Unit).matches(partial!(variant Unit));
        let failures = assert_that!(Example::Tuple(1)).capture(|it| it.matches(partial!(Unit)));
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn accepts_unqualified_standard_constructors() {
        assert_that!(Some(1)).matches(partial!(Some(eq(1))));
        assert_that!(Ok::<_, ()>(1)).matches(partial!(Ok(eq(1))));
        assert_that!(None::<i32>).matches(partial!(None));
    }
}
