use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo, RangeToInclusive};

use crate::failure::GenericFailure;
use crate::{AssertThat, Mode};

/// Assertions for generic arrays.
impl<'t, T, M: Mode> AssertThat<'t, RangeFrom<T>, M> {
    #[track_caller]
    pub fn contains_element(&self, expected: T)
    where
        T: PartialOrd + Debug,
    {
        if !self.actual().borrowed().contains(&expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual range: {actual:#?}\n\nDoes not contain expected: {expected:#?}",
                    actual = self.actual().borrowed()
                ),
            })
        }
    }
}

/// Assertions for generic arrays.
impl<'t, T, M: Mode> AssertThat<'t, RangeTo<T>, M> {
    #[track_caller]
    pub fn contains_element(&self, expected: T)
    where
        T: PartialOrd + Debug,
    {
        if !self.actual().borrowed().contains(&expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual range: {actual:#?}\n\nDoes not contain expected: {expected:#?}",
                    actual = self.actual().borrowed()
                ),
            })
        }
    }

    #[track_caller]
    pub fn not_contains_element(&self, expected: T)
    where
        T: PartialOrd + Debug,
    {
        if self.actual().borrowed().contains(&expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual RangeTo: {actual:#?}\n\nContains element expected not to be contained: {expected:#?}",
                    actual = self.actual().borrowed()
                ),
            })
        }
    }
}

// Assertions for generic arrays.
impl<'t, T, M: Mode> AssertThat<'t, RangeToInclusive<T>, M> {
    #[track_caller]
    pub fn contains_element(&self, expected: T)
    where
        T: PartialOrd + Debug,
    {
        if !self.actual().borrowed().contains(&expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual range: {actual:#?}\n\nDoes not contain expected: {expected:#?}",
                    actual = self.actual().borrowed()
                ),
            })
        }
    }
}
