use crate::{AssertrPartialEq, EqContext};

/// PartialEq like comparison on slices, but with an `EqContext`, tracking exact differences.
pub fn compare<V1, V2>(slice1: &[V1], slice2: &[V2], mut ctx: Option<&mut EqContext>) -> bool
where
    V1: AssertrPartialEq<V2>,
{
    if slice1.len() != slice2.len() {
        // TODO: Add difference? Remove this, as we always want to see exact differences?
        return false;
    }

    slice1.iter().enumerate().all(|(i, v1)| {
        // TODO: Handle absence with ctx.
        slice2
            .get(i)
            .map_or(false, |v2| AssertrPartialEq::eq(v1, v2, ctx.as_deref_mut()))
    })
}

#[cfg(test)]
mod test {
    use crate::cmp::slice::compare;
    use crate::prelude::*;
    use crate::EqContext;

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

        assert_that(compare(&slice1, &slice2, Some(&mut ctx))).is_true();
        assert_that(compare(&slice1, &slice3, Some(&mut ctx))).is_false();
    }
}
