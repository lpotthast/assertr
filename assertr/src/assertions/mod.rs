use ::core::fmt::Debug;
use ::core::ops::Range;
use ::std::ops::RangeInclusive;

pub mod alloc;
pub mod condition;
pub mod core;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "std")]
pub mod std;
#[cfg(feature = "tokio")]
pub mod tokio;

pub trait HasLength {
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    fn length(&self) -> usize;

    fn type_name_hint(&self) -> Option<&'static str> {
        None
    }
}

impl HasLength for &str {
    fn is_empty(&self) -> bool {
        str::is_empty(self)
    }

    fn length(&self) -> usize {
        str::len(self)
    }
}

impl HasLength for String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }

    fn length(&self) -> usize {
        String::len(self)
    }
}

impl HasLength for &String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }

    fn length(&self) -> usize {
        String::len(self)
    }
}

impl<T> HasLength for &[T] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, const S: usize> HasLength for [T; S] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn length(&self) -> usize {
        Vec::len(self)
    }
}

impl<T> HasLength for &Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn length(&self) -> usize {
        Vec::len(self)
    }
}

#[cfg(feature = "std")]
impl<K: Debug, V: Debug> HasLength for ::std::collections::HashMap<K, V> {
    fn is_empty(&self) -> bool {
        ::std::collections::HashMap::is_empty(self)
    }

    fn length(&self) -> usize {
        ::std::collections::HashMap::len(self)
    }

    fn type_name_hint(&self) -> Option<&'static str> {
        Some("HashMap")
    }
}

#[cfg(feature = "std")]
impl<K: Debug, V: Debug> HasLength for &::std::collections::HashMap<K, V> {
    fn is_empty(&self) -> bool {
        ::std::collections::HashMap::is_empty(self)
    }

    fn length(&self) -> usize {
        ::std::collections::HashMap::len(self)
    }

    fn type_name_hint(&self) -> Option<&'static str> {
        Some("&HashMap")
    }
}

impl HasLength for Range<i32> {
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn type_name_hint(&self) -> Option<&'static str> {
        Some("Range<i32>")
    }
}

impl HasLength for RangeInclusive<i32> {
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn type_name_hint(&self) -> Option<&'static str> {
        Some("RangeInclusive<i32>")
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn has_length_on_ranges() {
        assert_that(0..9).has_length(8);
        assert_that(0..=9).has_length(9);
    }
}
