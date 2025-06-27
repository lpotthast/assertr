use crate::{AssertrPartialEq, EqContext};
use core::fmt::Debug;

/// PartialEq like comparison on slices, but with an `EqContext`, tracking human-readable differences.
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
///         compare_with = "::assertr::cmp::slice::compare"
///     )]
///     pub bars: Vec<Bar>,
/// }
/// ```
pub fn compare<V1, V2>(slice1: &[V1], slice2: &[V2], mut ctx: Option<&mut EqContext>) -> bool
where
    V1: AssertrPartialEq<V2>,
    V1: Debug,
    V2: Debug,
{
    let cmp_result = crate::util::slice::compare_with_context(slice1, slice2, ctx.as_deref_mut());

    if let Some(ctx) = ctx {
        if !cmp_result.strictly_equal {
            if !cmp_result.same_length {
                ctx.add_difference(format!(
                    "Slices are not of the same length. A:{} and B:{}",
                    slice1.len(),
                    slice2.len()
                ));
            }
            if cmp_result.only_differing_in_order() {
                ctx.add_difference(format!(
                    "Slices only differ in their element-order. A:{slice1:#?} and B:{slice2:#?}"
                ));
            }
            if !cmp_result.not_in_b.is_empty() {
                ctx.add_difference(format!("Elements not expected: {:#?}", cmp_result.not_in_b));
            }
            if !cmp_result.not_in_a.is_empty() {
                ctx.add_difference(format!("Elements not found: {:#?}", cmp_result.not_in_a));
            }
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
}
