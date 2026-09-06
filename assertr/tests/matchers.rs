//! Structural macro behavior at a downstream call site.
#![cfg(feature = "matchers")]

mod named_fields {
    use assertr::prelude::*;

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
            children: elements_are![partial!(Child { id: 1, .. }), partial!(Child { id: 2, .. })],
            ..
        });
        assert_that!(parent).matches(&matcher);
        assert_that!(parent).with_renderer(Scalar).matches(&matcher);
        let failures = assert_that!(parent).with_renderer(Scalar).capture(|it| {
            it.matches(partial!(Parent {
                children: elements_are![
                    partial!(Child { id: 3, .. }),
                    partial!(Child { id: 4, .. })
                ],
                ..
            }))
        });
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children[1].path).is_equal_to(vec![
            assertr::failure::PathSegment::Field("children"),
            assertr::failure::PathSegment::Index(1),
            assertr::failure::PathSegment::Field("id"),
        ]);
        assert_that!(parent)
            .with_renderer(Scalar)
            .does_not_match(partial!(Parent {
                children: elements_are_in_any_order![partial!(Child { id: 5, .. })],
                ..
            }));
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
                2
            },
            x: {
                calls.borrow_mut().push("x");
                1
            },
        });
        assert_that!(*calls.borrow()).is_equal_to(["y", "x"]);
        assert_that!(Pair { x: 1, y: 2 }).matches(&matcher);
        assert_that!(Pair { x: 1, y: 2 }).matches(&matcher);
        assert_that!(*calls.borrow()).is_equal_to(["y", "x"]);
    }
}

mod maps {
    use assertr::prelude::*;

    #[test]
    fn nested_maps_and_borrowed_values() {
        use std::collections::BTreeMap;

        struct Row<'a> {
            name: &'a str,
        }
        let name = String::from("Alice");
        let map = BTreeMap::from([(String::from("a"), Row { name: &name })]);
        assert_that!(map).matches(entries_are![("a", partial!(Row { name: "Alice" }))]);
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
                items: entries_are![("x", partial!(Item { id: 2 }))]
            }))
        });
        assert_that!(failures[0].children[0].path).has_length(3);
        let path = &failures[0].children[0].path;
        assert_that!(path[0]).is_equal_to(PathSegment::Field("items"));
        assert_that!(path[1]).is_matching(pattern!(PathSegment::Key(_)));
        assert_that!(path[2]).is_equal_to(PathSegment::Field("id"));
    }
}

mod tuple_structs {
    use assertr::prelude::*;

    #[allow(dead_code)]
    struct Pair(i32, i32);

    #[test]
    fn accepts_wildcard_fields() {
        assert_that!(Pair(1, 2)).matches(partial!(Pair(1, _)));
    }

    #[test]
    fn accepts_final_rest() {
        assert_that!(Pair(1, 2)).matches(partial!(Pair(1, ..)));
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
    use assertr::{failure::PathSegment, matchers::anything, prelude::*};

    enum Example {
        Named { value: i32 },
        Tuple(i32),
        Unit,
    }

    #[test]
    fn explicit_variant_marker_records_a_typed_path() {
        let failures = assert_that!(Example::Tuple(2))
            .capture(|it| it.matches(partial!(variant Example::Tuple(3))));

        assert_that!(failures[0].children[0].path).is_equal_to([
            PathSegment::Variant("Example::Tuple"),
            PathSegment::TupleIndex(0),
        ]);
    }

    #[test]
    fn mismatched_variants_require_no_renderer() {
        struct NoRenderer;

        assert_that!(Example::Unit)
            .with_renderer(NoRenderer)
            .does_not_match(partial!(Example::Tuple(anything())));
    }

    #[test]
    fn accepts_named_variants() {
        assert_that!(Example::Named { value: 1 }).matches(partial!(Example::Named { value: 1 }));
    }

    #[test]
    fn accepts_tuple_variants() {
        assert_that!(Example::Tuple(2)).matches(partial!(Example::Tuple(2)));
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
        assert_that!(Example::Tuple(1)).does_not_match(partial!(Unit));
    }

    #[test]
    fn accepts_unqualified_standard_constructors() {
        assert_that!(Some(1)).matches(partial!(Some(1)));
        assert_that!(Ok::<_, ()>(1)).matches(partial!(Ok(1)));
        assert_that!(None::<i32>).matches(partial!(None));
    }
}
