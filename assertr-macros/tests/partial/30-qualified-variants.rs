use renamed_assertr::{failure::PathSegment, matchers::eq, prelude::*};

enum Message {
    Named { value: i32 },
    Tuple(i32),
    Unit,
}

impl Message {
    fn check_self_path() {
        assert_that!(Self::Tuple(3)).matches(partial!(variant Self::Tuple(eq(3))));
    }
}

mod nested {
    use super::*;

    pub fn check_super_path() {
        assert_that!(Message::Unit).matches(partial!(variant super::Message::Unit));
    }
}

mod variant {
    pub struct Value(pub i32);
}

mod constructors {
    use super::*;

    pub fn check_marker_named_constructors() {
        #[allow(non_camel_case_types)]
        struct variant {
            value: i32,
        }
        assert_that!(variant { value: 3 }).matches(partial!(variant { value: eq(3) }));
        {
            #[allow(non_camel_case_types)]
            struct variant(i32);
            assert_that!(variant(3)).matches(partial!(variant(eq(3))));
        }
        {
            #[allow(non_camel_case_types)]
            struct variant;
            assert_that!(variant).matches(partial!(variant));
        }
    }
}

fn main() {
    assert_that!(Message::Named { value: 3 })
        .matches(partial!(variant crate::Message::Named { value: eq(3) }));
    assert_that!(Message::Tuple(3)).matches(partial!(variant self::Message::Tuple(eq(3))));
    assert_that!(Some(3)).matches(partial!(variant::core::option::Option::Some(eq(3))));
    assert_that!(None::<i32>).matches(partial!(variant::core::option::Option::None));
    Message::check_self_path();
    nested::check_super_path();
    constructors::check_marker_named_constructors();

    let failures = assert_that!(Message::Named { value: 3 })
        .capture(|it| it.matches(partial!(variant crate::Message::Named { value: eq(9) })));
    assert_that!(failures[0].children[0].path).contains_exactly([
        PathSegment::Variant("crate::Message::Named"),
        PathSegment::Field("value"),
    ]);
    let failures = assert_that!(Some(3))
        .capture(|it| it.matches(partial!(variant::core::option::Option::Some(eq(9)))));
    assert_that!(failures[0].children[0].path).contains_exactly([
        PathSegment::Variant("core::option::Option::Some"),
        PathSegment::TupleIndex(0),
    ]);
    let failures = assert_that!(Message::Tuple(3))
        .capture(|it| it.matches(partial!(variant crate::Message::Unit)));
    assert_that!(failures[0].children[0].path)
        .contains_exactly([PathSegment::Variant("crate::Message::Unit")]);

    // The raw identifier escapes the marker when the constructor's module is named variant.
    assert_that!(variant::Value(3)).matches(partial!(r#variant::Value(eq(3))));
    let failures =
        assert_that!(variant::Value(3)).capture(|it| it.matches(partial!(r#variant::Value(eq(9)))));
    assert_that!(failures[0].children[0].path).contains_exactly([PathSegment::TupleIndex(0)]);
}
