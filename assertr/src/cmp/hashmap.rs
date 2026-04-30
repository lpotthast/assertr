use crate::{AssertrPartialEq, EqContext};
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

/// Bound helper for `#[assertr_eq(compare_with = "::assertr::cmp::hashmap::compare")]`.
///
/// This trait means an actual map value can be compared with an expected map value. Users normally
/// only name this trait in `#[assertr_eq(compare_bounds = "...")]`.
pub trait CompareValue<Expected, R>: AssertrPartialEq<Expected, R> {}

impl<Actual, Expected, R> CompareValue<Expected, R> for Actual where
    Actual: AssertrPartialEq<Expected, R>
{
}

/// This function is supposed to be used when deriving `AssertrEq`:
/// ```
/// use std::collections::HashMap;
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
///         map_type = "HashMap<String, BarAssertrEq>",
///         compare_with = "::assertr::cmp::hashmap::compare",
///         compare_bounds = "Bar: ::assertr::cmp::hashmap::CompareValue<BarAssertrEq, R>"
///     )]
///     pub bars: HashMap<String, Bar>,
/// }
/// ```
pub fn compare<K, V1, V2, S1, S2, R>(
    map1: &HashMap<K, V1, S1>,
    map2: &HashMap<K, V2, S2>,
    mut ctx: Option<&mut EqContext<'_, R>>,
) -> bool
where
    K: Eq + Hash,
    V1: CompareValue<V2, R>,
    S1: BuildHasher,
    S2: BuildHasher,
{
    if map1.len() != map2.len() {
        return false;
    }

    map1.iter().all(|(k, v1)| {
        map2.get(k)
            .is_some_and(|v2| AssertrPartialEq::eq(v1, v2, ctx.as_deref_mut()))
    })
}

#[cfg(test)]
mod test {
    use crate::EqContext;
    use crate::cmp::hashmap::compare;
    use crate::prelude::*;
    use std::collections::HashMap;

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
    fn can_check_equality_between_map_of_comparable_value_types() {
        let m1 = HashMap::from([("e1", Foo { id: 42 })]);
        let m2 = HashMap::from([("e1", Bar { id: 42 })]);
        let m3 = HashMap::from([("e1", Bar { id: 43 })]);

        let mut ctx = EqContext::new();

        assert_that!(compare(&m1, &m2, Some(&mut ctx))).is_true();
        assert_that!(compare(&m1, &m3, Some(&mut ctx))).is_false();
    }
}
