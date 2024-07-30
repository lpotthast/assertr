use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{BuildHasher, Hash, RandomState};

/// HashMap wrapper implementing PartialEq for all other HashMaps where the value types of
/// both HashMaps are comparable.
pub struct HashMapExt<K, V, S = RandomState> {
    pub inner: HashMap<K, V, S>,
}

impl<K, V, S> Debug for HashMapExt<K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.inner.iter()).finish()
    }
}

impl<K, V, S> HashMapExt<K, V, S> {
    pub fn new(map: HashMap<K, V, S>) -> Self {
        Self { inner: map }
    }
}

impl<K, V1, V2, S> PartialEq<HashMapExt<K, V2, S>> for HashMapExt<K, V1, S>
where
    K: Eq + Hash,
    V1: PartialEq<V2>,
    S: BuildHasher,
{
    fn eq(&self, other: &HashMapExt<K, V2, S>) -> bool {
        if self.inner.len() != other.inner.len() {
            return false;
        }

        self.inner.iter().all(|(key, value)| other.inner.get(key).map_or(false, |v| *value == *v))
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::prelude::*;
    use crate::util::hashmap_ext::HashMapExt;

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
        let m1 = HashMapExt::new(HashMap::from([("e1", Foo { id: 42 })]));
        let m2 = HashMapExt::new(HashMap::from([("e1", Bar { id: 42 })]));
        let m3 = HashMapExt::new(HashMap::from([("e1", Bar { id: 43 })]));

        assert_that::<HashMapExt<&'static str, Foo, _>>(&m1).is_equal_to(m2);
        assert_that::<HashMapExt<&'static str, Foo, _>>(&m1).is_not_equal_to(m3);
    }
}