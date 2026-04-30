use crate::{AssertionRenderer, AssertrPartialEq, EqContext};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Bound helper for `#[assertr_eq(compare_with = "::assertr::cmp::slice::compare")]`.
///
/// This trait means an actual slice element can be compared with an expected slice element while
/// both can be rendered by the active assertion renderer. Users normally only name this trait in
/// `#[assertr_eq(compare_bounds = "...")]`.
pub trait CompareElement<Expected, R>: AssertrPartialEq<Expected, R> {
    fn render_actual_values(ctx: &EqContext<'_, R>, values: &[&Self]) -> String;

    fn render_expected_values(ctx: &EqContext<'_, R>, values: &[&Expected]) -> String;
}

impl<Actual, Expected, R> CompareElement<Expected, R> for Actual
where
    Actual: AssertrPartialEq<Expected, R>,
    R: AssertionRenderer<Actual> + AssertionRenderer<Expected>,
{
    fn render_actual_values(ctx: &EqContext<'_, R>, values: &[&Self]) -> String {
        format!("{:#?}", ctx.render_values(values))
    }

    fn render_expected_values(ctx: &EqContext<'_, R>, values: &[&Expected]) -> String {
        format!("{:#?}", ctx.render_values(values))
    }
}

/// `PartialEq` like comparison on slices, but with an `EqContext`, tracking human-readable differences.
///
/// This function is supposed to be used when deriving `AssertrEq` and having a slice-like type:
/// ```
/// use assertr::prelude::*;
///
/// #[derive(Debug, AssertrEq)]
/// pub struct Bar {
///     pub id: i32,
/// }
///
/// #[derive(Debug, AssertrEq)]
/// pub struct Foo {
///     pub id: i32,
///
///     #[assertr_eq(
///         map_type = "Vec<BarAssertrEq>",
///         compare_with = "::assertr::cmp::slice::compare",
///         compare_bounds = "Bar: ::assertr::cmp::slice::CompareElement<BarAssertrEq, R>"
///     )]
///     pub bars: Vec<Bar>,
/// }
/// ```
pub fn compare<V1, V2, R>(
    slice1: &[V1],
    slice2: &[V2],
    mut ctx: Option<&mut EqContext<'_, R>>,
) -> bool
where
    V1: CompareElement<V2, R>,
{
    let cmp_result = crate::util::slice::compare_with_context(slice1, slice2, ctx.as_deref_mut());

    if let Some(ctx) = ctx
        && !cmp_result.strictly_equal
    {
        if !cmp_result.same_length {
            ctx.add_difference(format!(
                "Slices are not of the same length. A:{} and B:{}",
                slice1.len(),
                slice2.len()
            ));
        }
        if cmp_result.only_differing_in_order() {
            let slice1_values = slice1.iter().collect::<Vec<_>>();
            let slice2_values = slice2.iter().collect::<Vec<_>>();
            ctx.add_difference(format!(
                "Slices only differ in their element-order. A:{} and B:{}",
                V1::render_actual_values(ctx, slice1_values.as_slice()),
                V1::render_expected_values(ctx, slice2_values.as_slice())
            ));
        }
        if !cmp_result.not_in_b.is_empty() {
            ctx.add_difference(format!(
                "Elements not expected: {}",
                V1::render_actual_values(ctx, cmp_result.not_in_b.as_slice())
            ));
        }
        if !cmp_result.not_in_a.is_empty() {
            ctx.add_difference(format!(
                "Elements not found: {}",
                V1::render_expected_values(ctx, cmp_result.not_in_a.as_slice())
            ));
        }
    }

    cmp_result.strictly_equal
}

#[cfg(test)]
mod test {
    use crate::EqContext;
    use crate::cmp::slice::compare;
    use crate::prelude::*;

    #[derive(Debug, PartialEq)]
    struct Foo {
        id: u32,
    }

    #[derive(Debug, PartialEq)]
    struct Bar {
        id: u32,
    }

    impl PartialEq<Bar> for Foo {
        fn eq(&self, other: &Bar) -> bool {
            self.id == other.id
        }
    }

    #[test]
    fn can_check_equality_between_slices_of_comparable_value_types() {
        let slice1 = [Foo { id: 42 }];
        let slice2 = [Bar { id: 42 }];
        let slice3 = [Bar { id: 43 }];

        let mut ctx = EqContext::new();

        assert_that!(compare(&slice1, &slice2, Some(&mut ctx))).is_true();
        assert_that!(compare(&slice1, &slice3, Some(&mut ctx))).is_false();
    }

    #[test]
    fn reports_no_differences_on_equal_slices() {
        let slice1 = [1, 2, 3];
        let slice2 = [1, 2, 3];

        let mut ctx = EqContext::new();

        let result = compare(&slice1, &slice2, Some(&mut ctx));

        assert_that!(result).is_true();
        assert_that!(ctx.differences.differences).is_empty();
    }

    #[test]
    fn reports_differences_on_unequal_slices_of_same_length() {
        let slice1 = [1, 2, 3];
        let slice2 = [2, 3, 4];

        let mut ctx = EqContext::new();

        let result = compare(&slice1, &slice2, Some(&mut ctx));

        assert_that!(result).is_false();
        assert_that!(ctx.differences.differences).contains_exactly(&[
            "Elements not expected: [\n    1,\n]".to_string(),
            "Elements not found: [\n    4,\n]".to_string(),
        ]);
    }

    #[test]
    fn reports_differences_on_unequal_slices_of_same_length_only_differing_in_order() {
        let slice1 = [1, 2, 3];
        let slice2 = [1, 3, 2];

        let mut ctx = EqContext::new();

        let result = compare(&slice1, &slice2, Some(&mut ctx));

        assert_that!(result).is_false();
        assert_that!(ctx.differences.differences).contains_exactly(&[
            "Slices only differ in their element-order. A:[\n    1,\n    2,\n    3,\n] and B:[\n    1,\n    3,\n    2,\n]".to_string(),
        ]);
    }

    #[test]
    fn reports_differences_on_unequal_slices_of_different_length() {
        let slice1 = [1, 2, 3];
        let slice2 = [1, 2, 3, 4];

        let mut ctx = EqContext::new();

        let result = compare(&slice1, &slice2, Some(&mut ctx));

        assert_that!(result).is_false();
        assert_that!(ctx.differences.differences).contains_exactly(&[
            "Slices are not of the same length. A:3 and B:4".to_string(),
            "Elements not found: [\n    4,\n]".to_string(),
        ]);
    }

    #[test]
    fn reports_formatted_elements_with_custom_renderer() {
        #[derive(PartialEq)]
        struct Actual(u32);

        #[derive(PartialEq)]
        struct Expected(u32);

        impl PartialEq<Expected> for Actual {
            fn eq(&self, other: &Expected) -> bool {
                self.0 == other.0
            }
        }

        struct Renderer;

        impl AssertionRenderer<Actual> for Renderer {
            fn fmt(&self, value: &Actual, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("Actual({})", value.0))
            }
        }

        impl AssertionRenderer<Expected> for Renderer {
            fn fmt(&self, value: &Expected, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("Expected({})", value.0))
            }
        }

        let renderer = Renderer;
        let mut ctx = EqContext::with_renderer(&renderer);

        let result = compare(
            &[Actual(1), Actual(2), Actual(3)],
            &[Expected(2), Expected(3), Expected(4)],
            Some(&mut ctx),
        );

        assert_that!(result).is_false();
        assert_that!(ctx.differences.differences).contains_exactly(&[
            "Elements not expected: [\n    Actual(1),\n]".to_string(),
            "Elements not found: [\n    Expected(4),\n]".to_string(),
        ]);
    }

    #[test]
    fn reports_reordered_elements_with_custom_renderer() {
        #[derive(PartialEq)]
        struct Value(u32);

        struct Renderer;

        impl AssertionRenderer<Value> for Renderer {
            fn fmt(&self, value: &Value, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("Value({})", value.0))
            }
        }

        let renderer = Renderer;
        let mut ctx = EqContext::with_renderer(&renderer);

        let result = compare(
            &[Value(1), Value(2), Value(3)],
            &[Value(1), Value(3), Value(2)],
            Some(&mut ctx),
        );

        assert_that!(result).is_false();
        assert_that!(ctx.differences.differences).contains_exactly(&[
            "Slices only differ in their element-order. A:[\n    Value(1),\n    Value(2),\n    Value(3),\n] and B:[\n    Value(1),\n    Value(3),\n    Value(2),\n]".to_string(),
        ]);
    }
}
