use alloc::string::String;
use core::any::TypeId;

pub trait Mode: Default + Clone + 'static {
    fn is_panic(&self) -> bool {
        TypeId::of::<Self>() == TypeId::of::<Panic>()
    }

    fn is_capture(&self) -> bool {
        TypeId::of::<Self>() == TypeId::of::<Capture>()
    }

    fn set_derived(&mut self);
}

/// Panic mode. When an assertion fails, a panic message is raised and the program terminates immediately.
/// Subsequent assertions after a failure are therefore not executed.
/// This is the default mode and allows an AssertThat to be mapped to a different type with a condition,
/// failing when that condition cannot be met.
/// A good example for that is `assert_that(Err("foo")).is_err().is_equal_to("foo")`, where the `is_err`
/// implementation can map the contained actual value to the results error value and allow for simpler chaining of assertions.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Panic {
    pub(crate) derived: bool,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Capture {
    pub(crate) derived: bool,
    pub(crate) captured: bool,
}

impl Mode for Panic {
    fn set_derived(&mut self) {
        self.derived = true;
    }
}
impl Mode for Capture {
    fn set_derived(&mut self) {
        self.derived = true;
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if !self.captured && !self.derived {
            // Note: We cannot print the actual value, as we cannot add bounds to T,
            // as this would render this Drop implementation not being called for all AssertThat's!
            panic!("{}", String::from("You dropped an `assert_that(..)` value, on which `.with_capture()` was called, without actually capturing the assertion failures using `.capture_failures()`!"));
        }
    }
}
