use std::collections::HashMap;
use std::hash::Hash;

pub fn compare<K, V1, V2>(map1: &HashMap<K, V1>, map2: &HashMap<K, V2>) -> bool
where
    K: Eq + Hash,
    V1: PartialEq<V2>,
{
    if map1.len() != map2.len() {
        return false;
    }

    map1.iter().all(|(k, v1)| {
        map2.get(k).map_or(false, |v2| v1 == v2)
    })
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::prelude::*;
    use crate::util::hashmap::compare;

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
            return self.id == other.id;
        }
    }

    #[test]
    fn can_check_equality_between_map_of_comparable_value_types() {
        let m1 = HashMap::from([("e1", Foo { id: 42 })]);
        let m2 = HashMap::from([("e1", Bar { id: 42 })]);
        let m3 = HashMap::from([("e1", Bar { id: 43 })]);

        assert_that(compare(&m1, &m2)).is_true();
        assert_that(compare(&m1, &m3)).is_false();
    }
}