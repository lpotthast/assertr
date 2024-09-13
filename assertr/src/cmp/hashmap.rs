use crate::{AssertrPartialEq, EqContext};
use std::collections::HashMap;
use std::hash::Hash;

pub fn compare<K, V1, V2>(
    map1: &HashMap<K, V1>,
    map2: &HashMap<K, V2>,
    mut ctx: Option<&mut EqContext>,
) -> bool
where
    K: Eq + Hash,
    V1: AssertrPartialEq<V2>,
{
    if map1.len() != map2.len() {
        return false;
    }

    map1.iter().all(|(k, v1)| {
        map2.get(k)
            .map_or(false, |v2| AssertrPartialEq::eq(v1, v2, ctx.as_deref_mut()))
    })
}

#[cfg(test)]
mod test {
    use crate::cmp::hashmap::compare;
    use crate::prelude::*;
    use crate::EqContext;
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

        assert_that(compare(&m1, &m2, Some(&mut ctx))).is_true();
        assert_that(compare(&m1, &m3, Some(&mut ctx))).is_false();
    }
}
