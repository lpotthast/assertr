use ::core::fmt::Debug;
use ::core::ops::Range;
use ::std::ops::RangeInclusive;

pub mod alloc;
pub mod condition;
pub mod core;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "jiff")]
pub mod jiff;
#[cfg(feature = "num")]
pub mod num;
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

#[cfg(feature = "std")]
impl<V: Debug> HasLength for ::std::collections::HashSet<V> {
    fn length(&self) -> usize {
        ::std::collections::HashSet::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashSet::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<V: Debug> HasLength for &::std::collections::HashSet<V> {
    fn length(&self) -> usize {
        ::std::collections::HashSet::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashSet::is_empty(self)
    }
}

impl HasLength for Range<usize> {
    fn length(&self) -> usize {
        if self.start < self.end {
            self.end - self.start
        } else {
            self.start - self.end
        }
    }
}

impl HasLength for RangeInclusive<usize> {
    fn length(&self) -> usize {
        let diff = if self.start() < self.end() {
            self.end() - self.start()
        } else {
            self.start() - self.end()
        };
        diff + 1
    }
}

impl HasLength for Range<u8> {
    fn length(&self) -> usize {
        (self.end as i16 - self.start as i16).unsigned_abs() as usize
    }
}

impl HasLength for RangeInclusive<u8> {
    fn length(&self) -> usize {
        (*self.end() as i16 - *self.start() as i16).unsigned_abs() as usize + 1
    }
}

impl HasLength for Range<u16> {
    fn length(&self) -> usize {
        (self.end as i32 - self.start as i32).unsigned_abs() as usize
    }
}

impl HasLength for RangeInclusive<u16> {
    fn length(&self) -> usize {
        (*self.end() as i32 - *self.start() as i32).unsigned_abs() as usize + 1
    }
}

impl HasLength for Range<u32> {
    fn length(&self) -> usize {
        (self.end as i64 - self.start as i64).unsigned_abs() as usize
    }
}

impl HasLength for RangeInclusive<u32> {
    fn length(&self) -> usize {
        (*self.end() as i64 - *self.start() as i64).unsigned_abs() as usize + 1
    }
}

impl HasLength for Range<u64> {
    fn length(&self) -> usize {
        (self.end as i128 - self.start as i128).unsigned_abs() as usize
    }
}

impl HasLength for RangeInclusive<u64> {
    fn length(&self) -> usize {
        (*self.end() as i128 - *self.start() as i128).unsigned_abs() as usize + 1
    }
}

impl HasLength for Range<i8> {
    fn length(&self) -> usize {
        (self.end - self.start).unsigned_abs() as usize
    }
}

impl HasLength for RangeInclusive<i8> {
    fn length(&self) -> usize {
        (*self.end() - *self.start()).unsigned_abs() as usize + 1
    }
}

impl HasLength for Range<i16> {
    fn length(&self) -> usize {
        (self.end - self.start).unsigned_abs() as usize
    }
}

impl HasLength for RangeInclusive<i16> {
    fn length(&self) -> usize {
        (*self.end() - *self.start()).unsigned_abs() as usize + 1
    }
}

impl HasLength for Range<i32> {
    fn length(&self) -> usize {
        (self.end - self.start).unsigned_abs() as usize
    }
}

impl HasLength for RangeInclusive<i32> {
    fn length(&self) -> usize {
        (*self.end() - *self.start()).unsigned_abs() as usize + 1
    }
}

impl HasLength for Range<i64> {
    fn length(&self) -> usize {
        (self.end - self.start).unsigned_abs() as usize
    }
}

impl HasLength for RangeInclusive<i64> {
    fn length(&self) -> usize {
        (*self.end() - *self.start()).unsigned_abs() as usize + 1
    }
}

#[cfg(test)]
mod tests {
    mod has_length {

        mod on_usize_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_usize..9_usize).has_length(8);
                assert_that(1_usize..=9_usize).has_length(9);

                // inverted range
                assert_that(9_usize..1_usize).has_length(8);
                assert_that(9_usize..=1_usize).has_length(9);
            }
        }

        mod on_u8_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_u8..9_u8).has_length(8);
                assert_that(1_u8..=9_u8).has_length(9);

                // inverted range
                assert_that(9_u8..1_u8).has_length(8);
                assert_that(9_u8..=1_u8).has_length(9);
            }
        }

        mod on_u16_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_u16..9_u16).has_length(8);
                assert_that(1_u16..=9_u16).has_length(9);

                // inverted range
                assert_that(9_u16..1_u16).has_length(8);
                assert_that(9_u16..=1_u16).has_length(9);
            }
        }

        mod on_u32_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_u32..9_u32).has_length(8);
                assert_that(1_u32..=9_u32).has_length(9);

                // inverted range
                assert_that(9_u32..1_u32).has_length(8);
                assert_that(9_u32..=1_u32).has_length(9);
            }
        }

        mod on_u64_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_u64..9_u64).has_length(8);
                assert_that(1_u64..=9_u64).has_length(9);

                // inverted range
                assert_that(9_u64..1_u64).has_length(8);
                assert_that(9_u64..=1_u64).has_length(9);
            }
        }

        mod on_i8_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_i8..9_i8).has_length(8);
                assert_that(1_i8..=9_i8).has_length(9);

                // inverted range
                assert_that(9_i8..1_i8).has_length(8);
                assert_that(9_i8..=1_i8).has_length(9);

                // negative range
                assert_that(-9_i8..-1_i8).has_length(8);
                assert_that(-9_i8..=-1_i8).has_length(9);

                // across zero
                assert_that(-4_i8..4_i8).has_length(8);
                assert_that(-4_i8..=4_i8).has_length(9);
            }
        }

        mod on_i16_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_i16..9_i16).has_length(8);
                assert_that(1_i16..=9_i16).has_length(9);

                // inverted range
                assert_that(9_i16..1_i16).has_length(8);
                assert_that(9_i16..=1_i16).has_length(9);

                // negative range
                assert_that(-9_i16..-1_i16).has_length(8);
                assert_that(-9_i16..=-1_i16).has_length(9);

                // across zero
                assert_that(-4_i16..4_i16).has_length(8);
                assert_that(-4_i16..=4_i16).has_length(9);
            }
        }

        mod on_i32_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_i32..9_i32).has_length(8);
                assert_that(1_i32..=9_i32).has_length(9);

                // inverted range
                assert_that(9_i32..1_i32).has_length(8);
                assert_that(9_i32..=1_i32).has_length(9);

                // negative range
                assert_that(-9_i32..-1_i32).has_length(8);
                assert_that(-9_i32..=-1_i32).has_length(9);

                // across zero
                assert_that(-4_i32..4_i32).has_length(8);
                assert_that(-4_i32..=4_i32).has_length(9);
            }
        }

        mod on_i64_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that(1_i64..9_i64).has_length(8);
                assert_that(1_i64..=9_i64).has_length(9);

                // inverted range
                assert_that(9_i64..1_i64).has_length(8);
                assert_that(9_i64..=1_i64).has_length(9);

                // negative range
                assert_that(-9_i64..-1_i64).has_length(8);
                assert_that(-9_i64..=-1_i64).has_length(9);

                // across zero
                assert_that(-4_i64..4_i64).has_length(8);
                assert_that(-4_i64..=4_i64).has_length(9);
            }
        }
    }
}
