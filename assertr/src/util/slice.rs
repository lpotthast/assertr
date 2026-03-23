use alloc::vec::Vec;

use crate::{AssertrPartialEq, EqContext};

pub(crate) struct CompareResult<'t, A, B> {
    pub(crate) strictly_equal: bool,
    pub(crate) same_length: bool,
    pub(crate) not_in_a: Vec<&'t B>,
    pub(crate) not_in_b: Vec<&'t A>,
}

impl<A, B> CompareResult<'_, A, B> {
    pub fn only_differing_in_order(&self) -> bool {
        !self.strictly_equal
            && self.same_length
            && self.not_in_a.is_empty()
            && self.not_in_b.is_empty()
    }
}

pub(crate) fn compare<'t, A, B>(aa: &'t [A], bb: &'t [B]) -> CompareResult<'t, A, B>
where
    A: AssertrPartialEq<B>,
{
    compare_with_context(aa, bb, None)
}

// TODO: Move to cmp module and rename.
pub(crate) fn compare_with_context<'t, A, B>(
    aa: &'t [A],
    bb: &'t [B],
    mut ctx: Option<&mut EqContext>,
) -> CompareResult<'t, A, B>
where
    A: AssertrPartialEq<B>,
{
    if AssertrPartialEq::eq(aa, bb, ctx.as_deref_mut()) {
        return CompareResult {
            strictly_equal: true,
            same_length: true,
            not_in_a: Vec::new(),
            not_in_b: Vec::new(),
        };
    }

    let same_length = aa.len() == bb.len();

    let mut not_in_a = Vec::new();
    let mut not_in_b = Vec::new();

    for a in aa {
        if !bb
            .iter()
            .any(|b| AssertrPartialEq::eq(a, b, ctx.as_deref_mut()))
        {
            not_in_b.push(a);
        }
    }

    for b in bb {
        if !aa
            .iter()
            .any(|a| AssertrPartialEq::eq(a, b, ctx.as_deref_mut()))
        {
            not_in_a.push(b);
        }
    }

    CompareResult {
        strictly_equal: false,
        same_length,
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
        fn returns_equal_on_equal_input_using_refs() {
            let result = compare(&[&1, &2, &3], &[&1, &2, &3]);

            result.only_differing_in_order().must().be_false();
            result.strictly_equal.must().be_true();
            result.same_length.must().be_true();
            result.not_in_a.must().be_empty();
            result.not_in_b.must().be_empty();
        }

        #[test]
        fn returns_equal_on_equal_input() {
            let result = compare(&[1, 2, 3], &[1, 2, 3]);

            result.only_differing_in_order().must().be_false();
            result.strictly_equal.must().be_true();
            result.same_length.must().be_true();
            result.not_in_a.must().be_empty();
            result.not_in_b.must().be_empty();
        }

        #[test]
        fn returns_not_equal_on_equal_but_rearranged_input() {
            let result = compare(&[1, 2, 3], &[3, 2, 1]);

            result.only_differing_in_order().must().be_true();
            result.strictly_equal.must().be_false();
            result.same_length.must().be_true();
            result.not_in_a.must().be_empty();
            result.not_in_b.must().be_empty();
        }

        #[test]
        fn returns_not_equal_and_lists_differences_on_differing_input() {
            let result = compare(&[1, 5, 7], &[5, 3, 4, 42]);

            result.only_differing_in_order().must().be_false();
            result.strictly_equal.must().be_false();
            result.same_length.must().be_false();
            result.not_in_a.must().contain_exactly([&3, &4, &42]);
            result.not_in_b.must().contain_exactly([&1, &7]);
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

            result.not_matched.must().be_empty();
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

            result.not_matched.must().be_empty();
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

            result.not_matched.must().contain_exactly([&1, &7]);
        }
    }
}
