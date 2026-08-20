use ::alloc::{collections::VecDeque, string::String, vec::Vec};
use ::core::ops::{Range, RangeInclusive};
#[cfg(feature = "std")]
use ::std::hash::BuildHasher;

pub mod alloc;
pub(crate) mod collection;
pub mod condition;
pub mod core;
#[cfg(feature = "http")]
pub mod http;
pub(crate) mod iterator;
#[cfg(feature = "jiff")]
pub mod jiff;
#[cfg(feature = "num")]
pub mod num;
#[cfg(feature = "program")]
pub mod program;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "rootcause")]
pub mod rootcause;
#[cfg(feature = "std")]
pub mod std;
#[cfg(feature = "tokio")]
pub mod tokio;

pub trait HasLength {
    fn length(&self) -> usize;

    #[must_use]
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    #[must_use]
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

impl<T> HasLength for VecDeque<T> {
    fn length(&self) -> usize {
        VecDeque::len(self)
    }

    fn is_empty(&self) -> bool {
        VecDeque::is_empty(self)
    }
}

impl<T> HasLength for &VecDeque<T> {
    fn length(&self) -> usize {
        VecDeque::len(self)
    }

    fn is_empty(&self) -> bool {
        VecDeque::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<K, V, S: BuildHasher> HasLength for ::std::collections::HashMap<K, V, S> {
    fn length(&self) -> usize {
        ::std::collections::HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashMap::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<K, V, S: BuildHasher> HasLength for &::std::collections::HashMap<K, V, S> {
    fn length(&self) -> usize {
        ::std::collections::HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashMap::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<V, S: BuildHasher> HasLength for ::std::collections::HashSet<V, S> {
    fn length(&self) -> usize {
        ::std::collections::HashSet::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashSet::is_empty(self)
    }
}

#[cfg(feature = "std")]
impl<V, S: BuildHasher> HasLength for &::std::collections::HashSet<V, S> {
    fn length(&self) -> usize {
        ::std::collections::HashSet::len(self)
    }

    fn is_empty(&self) -> bool {
        ::std::collections::HashSet::is_empty(self)
    }
}

/// Converts a range's mathematical length to `usize`, panicking when it does not fit.
fn range_length<D>(difference: D) -> usize
where
    D: TryInto<usize>,
{
    difference
        .try_into()
        .unwrap_or_else(|_| panic!("range length exceeds usize::MAX"))
}

/// Length of a non-empty inclusive range, `difference + 1`, panicking when it does not fit.
fn inclusive_range_length<D>(difference: D) -> usize
where
    D: TryInto<usize>,
{
    match range_length(difference).checked_add(1) {
        Some(length) => length,
        None => panic!("range length exceeds usize::MAX"),
    }
}

macro_rules! impl_has_length_for_unsigned_ranges {
    ($($type:ty),+ $(,)?) => {
        $(
            impl HasLength for Range<$type> {
                fn length(&self) -> usize {
                    if self.start < self.end {
                        range_length(self.end - self.start)
                    } else {
                        0
                    }
                }
            }

            impl HasLength for RangeInclusive<$type> {
                fn length(&self) -> usize {
                    if self.is_empty() {
                        0
                    } else {
                        inclusive_range_length(*self.end() - *self.start())
                    }
                }
            }
        )+
    };
}

macro_rules! impl_has_length_for_signed_ranges {
    ($($type:ty),+ $(,)?) => {
        $(
            impl HasLength for Range<$type> {
                fn length(&self) -> usize {
                    if self.start < self.end {
                        range_length(self.end.abs_diff(self.start))
                    } else {
                        0
                    }
                }
            }

            impl HasLength for RangeInclusive<$type> {
                fn length(&self) -> usize {
                    if self.is_empty() {
                        0
                    } else {
                        inclusive_range_length(self.end().abs_diff(*self.start()))
                    }
                }
            }
        )+
    };
}

impl_has_length_for_unsigned_ranges!(usize, u8, u16, u32, u64);
impl_has_length_for_signed_ranges!(i8, i16, i32, i64);

#[cfg(test)]
mod tests {
    #[allow(clippy::reversed_empty_ranges)]
    mod has_length {

        mod on_usize_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_usize..9_usize).has_length(8);
                assert_that!(1_usize..=9_usize).has_length(9);
                assert_that!(5_usize..=5_usize).has_length(1);

                // inverted range
                assert_that!(9_usize..1_usize).has_length(0);
                assert_that!(9_usize..=1_usize).has_length(0);
            }

            #[test]
            fn exhausted_inclusive_range_has_length_zero() {
                let mut range = 5_usize..=5_usize;
                assert_eq!(range.next(), Some(5));
                assert!(range.is_empty());

                assert_that!(range).has_length(0);
            }
        }

        mod on_u8_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_u8..9_u8).has_length(8);
                assert_that!(1_u8..=9_u8).has_length(9);
                assert_that!(u8::MIN..=u8::MAX).has_length(256);

                // inverted range
                assert_that!(9_u8..1_u8).has_length(0);
                assert_that!(9_u8..=1_u8).has_length(0);
            }
        }

        mod on_u16_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_u16..9_u16).has_length(8);
                assert_that!(1_u16..=9_u16).has_length(9);

                // inverted range
                assert_that!(9_u16..1_u16).has_length(0);
                assert_that!(9_u16..=1_u16).has_length(0);
            }
        }

        mod on_u32_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_u32..9_u32).has_length(8);
                assert_that!(1_u32..=9_u32).has_length(9);

                // inverted range
                assert_that!(9_u32..1_u32).has_length(0);
                assert_that!(9_u32..=1_u32).has_length(0);
            }
        }

        mod on_u64_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_u64..9_u64).has_length(8);
                assert_that!(1_u64..=9_u64).has_length(9);

                // inverted range
                assert_that!(9_u64..1_u64).has_length(0);
                assert_that!(9_u64..=1_u64).has_length(0);
            }
        }

        mod on_i8_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_i8..9_i8).has_length(8);
                assert_that!(1_i8..=9_i8).has_length(9);

                // inverted range
                assert_that!(9_i8..1_i8).has_length(0);
                assert_that!(9_i8..=1_i8).has_length(0);

                // negative range
                assert_that!(-9_i8..-1_i8).has_length(8);
                assert_that!(-9_i8..=-1_i8).has_length(9);

                // across zero
                assert_that!(-4_i8..4_i8).has_length(8);
                assert_that!(-4_i8..=4_i8).has_length(9);

                // full domain without intermediate signed overflow
                assert_that!(i8::MIN..i8::MAX).has_length(255);
                assert_that!(i8::MIN..=i8::MAX).has_length(256);
            }

            #[test]
            fn exhausted_inclusive_range_has_length_zero() {
                let mut range = 5_i8..=5_i8;
                assert_eq!(range.next(), Some(5));
                assert!(range.is_empty());

                assert_that!(range).has_length(0);
            }
        }

        mod on_i16_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_i16..9_i16).has_length(8);
                assert_that!(1_i16..=9_i16).has_length(9);

                // inverted range
                assert_that!(9_i16..1_i16).has_length(0);
                assert_that!(9_i16..=1_i16).has_length(0);

                // negative range
                assert_that!(-9_i16..-1_i16).has_length(8);
                assert_that!(-9_i16..=-1_i16).has_length(9);

                // across zero
                assert_that!(-4_i16..4_i16).has_length(8);
                assert_that!(-4_i16..=4_i16).has_length(9);
            }
        }

        mod on_i32_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_i32..9_i32).has_length(8);
                assert_that!(1_i32..=9_i32).has_length(9);

                // inverted range
                assert_that!(9_i32..1_i32).has_length(0);
                assert_that!(9_i32..=1_i32).has_length(0);

                // negative range
                assert_that!(-9_i32..-1_i32).has_length(8);
                assert_that!(-9_i32..=-1_i32).has_length(9);

                // across zero
                assert_that!(-4_i32..4_i32).has_length(8);
                assert_that!(-4_i32..=4_i32).has_length(9);
            }
        }

        mod on_i64_ranges {
            use crate::prelude::*;

            #[test]
            fn works_on_range_and_inclusive_range() {
                assert_that!(1_i64..9_i64).has_length(8);
                assert_that!(1_i64..=9_i64).has_length(9);

                // inverted range
                assert_that!(9_i64..1_i64).has_length(0);
                assert_that!(9_i64..=1_i64).has_length(0);

                // negative range
                assert_that!(-9_i64..-1_i64).has_length(8);
                assert_that!(-9_i64..=-1_i64).has_length(9);

                // across zero
                assert_that!(-4_i64..4_i64).has_length(8);
                assert_that!(-4_i64..=4_i64).has_length(9);
            }
        }
    }
}
