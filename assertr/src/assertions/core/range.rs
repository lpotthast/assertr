use core::fmt::Debug;
use core::ops::{RangeFrom, RangeTo, RangeToInclusive};

use crate::{tracking::AssertionTracking, AssertThat, Mode};

impl<'t, T, M: Mode> AssertThat<'t, RangeFrom<T>, M> {
    #[track_caller]
    pub fn contains_element(&self, expected: T)
    where
        T: PartialOrd + Debug,
    {
        self.track_assertion();
        if !self.actual().contains(&expected) {
            self.fail(format_args!(
                "Actual range: {actual:#?}\n\nDoes not contain expected: {expected:#?}",
                actual = self.actual()
            ))
        }
    }
}

impl<'t, T, M: Mode> AssertThat<'t, RangeTo<T>, M> {
    #[track_caller]
    pub fn contains_element(&self, expected: T)
    where
        T: PartialOrd + Debug,
    {
        self.track_assertion();
        if !self.actual().contains(&expected) {
            self.fail(format_args!(
                "Actual range: {actual:#?}\n\nDoes not contain expected: {expected:#?}",
                actual = self.actual()
            ))
        }
    }

    #[track_caller]
    pub fn not_contains_element(&self, expected: T)
    where
        T: PartialOrd + Debug,
    {
        self.track_assertion();
        if self.actual().contains(&expected) {
            self.fail(format_args!(
                    "Actual RangeTo: {actual:#?}\n\nContains element expected not to be contained: {expected:#?}",
                    actual = self.actual()
                ),
            )
        }
    }
}

impl<'t, T, M: Mode> AssertThat<'t, RangeToInclusive<T>, M> {
    #[track_caller]
    pub fn contains_element(&self, expected: T)
    where
        T: PartialOrd + Debug,
    {
        self.track_assertion();
        if !self.actual().contains(&expected) {
            self.fail(format_args!(
                "Actual range: {actual:#?}\n\nDoes not contain expected: {expected:#?}",
                actual = self.actual()
            ))
        }
    }
}
