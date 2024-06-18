pub(crate) struct CompareResult<'t, A, B> {
    pub(crate) strictly_equal: bool,
    pub(crate) only_differing_in_order: bool,
    pub(crate) not_in_a: Vec<&'t B>,
    pub(crate) not_in_b: Vec<&'t A>,
}

pub(crate) fn compare<'t, A, B>(aa: &'t [A], bb: &'t [B]) -> CompareResult<'t, A, B>
where
    A: PartialEq<B>,
{
    if aa == bb {
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

    for a in aa {
        if !bb.iter().any(|b| a == b) {
            not_in_b.push(a);
        }
    }

    for b in bb {
        if !aa.iter().any(|a| a == b) {
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::compare;

    #[test]
    fn compare_returns_equal_on_equal_input() {
        let result = compare(&[1, 2, 3], &[1, 2, 3]);

        assert_that(result.strictly_equal).is_true();
        assert_that(result.only_differing_in_order).is_false();
        assert_that(result.not_in_a).is_empty();
        assert_that(result.not_in_b).is_empty();
    }

    #[test]
    fn compare_returns_not_equal_on_equal_but_rearranged_input() {
        let result = compare(&[1, 2, 3], &[3, 2, 1]);

        assert_that(result.strictly_equal).is_false();
        assert_that(result.only_differing_in_order).is_true();
        assert_that(result.not_in_a).is_empty();
        assert_that(result.not_in_b).is_empty();
    }

    #[test]
    fn compare_returns_not_equal_and_lists_differences_on_differing_input() {
        let result = compare(&[1, 5, 7], &[5, 3, 4, 42]);

        assert_that(result.strictly_equal).is_false();
        assert_that(result.only_differing_in_order).is_false();
        assert_that(result.not_in_a).contains_exactly([&3, &4, &42]);
        assert_that(result.not_in_b).contains_exactly([&1, &7]);
    }
}
