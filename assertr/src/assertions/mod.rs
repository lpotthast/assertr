use ::core::fmt::Debug;
use ::core::ops::Range;
use ::std::ops::RangeInclusive;

pub mod alloc;
pub mod condition;
pub mod core;
#[cfg(feature = "jiff")]
pub mod jiff;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "std")]
pub mod std;
#[cfg(feature = "tokio")]
pub mod tokio;

pub trait HasLength {
    fn length(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}

impl HasLength for &str {
    fn length(&self) -> usize {
        str::len(self)
    }

    fn is_empty(&self) -> bool {
        str::is_empty(self)
    }
}

impl HasLength for String {
    fn length(&self) -> usize {
        String::len(self)
    }

    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }
}

impl HasLength for &String {
    fn length(&self) -> usize {
        String::len(self)
    }

    fn is_empty(&self) -> bool {
        String::is_empty(self)
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
    fn length(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}

impl<T> HasLength for &Vec<T> {
    fn length(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<K: Debug, V: Debug> HasLength for ::std::collections::HashMap<K, V> {
    fn length(&self) -> usize {
        ::std::collections::HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashMap::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<K: Debug, V: Debug> HasLength for &::std::collections::HashMap<K, V> {
    fn length(&self) -> usize {
        ::std::collections::HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashMap::is_empty(self)
    }
}

impl HasLength for Range<usize> {
    fn length(&self) -> usize {
        (self.end - 1) - self.start
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<usize> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for Range<u8> {
    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<u8> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for Range<u16> {
    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<u16> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for Range<u32> {
    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<u32> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for Range<u64> {
    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<u64> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for Range<i8> {
    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<i8> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for Range<i16> {
    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<i16> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for Range<i32> {
    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<i32> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for Range<i64> {
    fn length(&self) -> usize {
        let s = (self.end - 1) - self.start;
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl HasLength for RangeInclusive<i64> {
    fn length(&self) -> usize {
        let s = self.end() - self.start();
        s as usize
    }

    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

#[cfg(test)]
mod tests {
    mod has_length {

        mod on_usize_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_usize..9_usize).has_length(8);
                assert_that(0_usize..=9_usize).has_length(9);
            }
        }

        mod on_u8_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_u8..9_u8).has_length(8);
                assert_that(0_u8..=9_u8).has_length(9);
            }
        }

        mod on_u16_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_u16..9_u16).has_length(8);
                assert_that(0_u16..=9_u16).has_length(9);
            }
        }

        mod on_u32_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_u32..9_u32).has_length(8);
                assert_that(0_u32..=9_u32).has_length(9);
            }
        }

        mod on_u64_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_u64..9_u64).has_length(8);
                assert_that(0_u64..=9_u64).has_length(9);
            }
        }

        mod on_i8_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_i8..9_i8).has_length(8);
                assert_that(0_i8..=9_i8).has_length(9);
            }
        }

        mod on_i16_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_i16..9_i16).has_length(8);
                assert_that(0_i16..=9_i16).has_length(9);
            }
        }

        mod on_i32_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_i32..9_i32).has_length(8);
                assert_that(0_i32..=9_i32).has_length(9);
            }
        }

        mod on_i64_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(0_i64..9_i64).has_length(8);
                assert_that(0_i64..=9_i64).has_length(9);
            }
        }
    }
}
