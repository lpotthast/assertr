use crate::{AssertrPartialEq, EqContext};

pub(crate) struct CompareResult<'t, A, B> {
    pub(crate) strictly_equal: bool,
    pub(crate) only_differing_in_order: bool,
    pub(crate) not_in_a: Vec<&'t B>,
    pub(crate) not_in_b: Vec<&'t A>,
}

// TODO: Move to cmp module and rename.
pub(crate) fn compare<'t, A, B>(aa: &'t [A], bb: &'t [B]) -> CompareResult<'t, A, B>
where
    A: AssertrPartialEq<B>,
{
    if AssertrPartialEq::eq(aa, bb, None) {
        return CompareResult {
            strictly_equal: true,
            only_differing_in_order: false,
            not_in_a: Vec::new(),
            not_in_b: Vec::new(),
        };
    }

    let same_length = aa.len() == bb.len();

    let mut not_in_a = Vec::new();
    let mut not_in_b = Vec::new();

    let mut ctx = EqContext::new();

    for a in aa {
        if !bb.iter().any(|b| AssertrPartialEq::eq(a, b, Some(&mut ctx))) {
            not_in_b.push(a);
        }
    }

    for b in bb {
        if !aa.iter().any(|a| AssertrPartialEq::eq(a, b, Some(&mut ctx))) {
            not_in_a.push(b);
        }
    }

    CompareResult {
        strictly_equal: false,
        only_differing_in_order: same_length && not_in_a.is_empty() && not_in_b.is_empty(),
        not_in_a,
        not_in_b,
    }
}

pub(crate) struct TestMatchingResult<'t, A> {
    pub(crate) not_matched: Vec<&'t A>,
}

pub(crate) fn test_matching_any<'t, A, P>(
    aa: &'t [A],
    predicates: &'t [P],
) -> TestMatchingResult<'t, A>
where
    P: Fn(&'t A) -> bool,
{
    let mut not_matched = Vec::new();

    for a in aa {
        if !predicates.iter().any(|p| p(a)) {
            not_matched.push(a);
        }
    }

    TestMatchingResult { not_matched }
}

#[cfg(test)]
mod tests {
    mod compare {
        use crate::prelude::*;
        use crate::util::slice::compare;

        #[test]
        fn returns_equal_on_equal_input() {
            let result = compare(&[1, 2, 3], &[1, 2, 3]);

            assert_that(result.strictly_equal).is_true();
            assert_that(result.only_differing_in_order).is_false();
            assert_that(result.not_in_a).is_empty();
            assert_that(result.not_in_b).is_empty();
        }

        #[test]
        fn returns_not_equal_on_equal_but_rearranged_input() {
            let result = compare(&[1, 2, 3], &[3, 2, 1]);

            assert_that(result.strictly_equal).is_false();
            assert_that(result.only_differing_in_order).is_true();
            assert_that(result.not_in_a).is_empty();
            assert_that(result.not_in_b).is_empty();
        }

        #[test]
        fn returns_not_equal_and_lists_differences_on_differing_input() {
            let result = compare(&[1, 5, 7], &[5, 3, 4, 42]);

            assert_that(result.strictly_equal).is_false();
            assert_that(result.only_differing_in_order).is_false();
            assert_that(result.not_in_a).contains_exactly([&3, &4, &42]);
            assert_that(result.not_in_b).contains_exactly([&1, &7]);
        }
    }

    mod test_matching_any {
        use crate::prelude::*;
        use crate::util::slice::test_matching_any;

        #[test]
        fn returns_equal_on_matching_input() {
            let result = test_matching_any(
                &[1, 2, 3],
                [
                    move |it: &i32| *it == 1,
                    move |it: &i32| *it == 2,
                    move |it: &i32| *it == 3,
                ]
                    .as_slice(),
            );

            assert_that(result.not_matched).is_empty();
        }

        #[test]
        fn returns_not_equal_on_matching_but_rearranged_input() {
            let result = test_matching_any(
                &[1, 2, 3],
                [
                    move |it: &i32| *it == 3,
                    move |it: &i32| *it == 2,
                    move |it: &i32| *it == 1,
                ]
                    .as_slice(),
            );

            assert_that(result.not_matched).is_empty();
        }

        #[test]
        fn returns_not_equal_and_lists_differences_on_non_matching_input() {
            let result = test_matching_any(
                &[1, 5, 7],
                [
                    move |it: &i32| *it == 5,
                    move |it: &i32| *it == 3,
                    move |it: &i32| *it == 4,
                    move |it: &i32| *it == 42,
                ]
                    .as_slice(),
            );

            assert_that(result.not_matched).contains_exactly([&1, &7]);
        }
    }
}
