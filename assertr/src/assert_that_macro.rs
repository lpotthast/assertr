//! Internal implementation details for the `assert_that!` macro.
//!
//! **Do not use these types directly.** They are public only because the
//! `assert_that!` macro needs to reference them.

use core::cell::Cell;
use core::ops::Deref;

use crate::AssertThat;
use crate::actual::Actual;
use crate::mode::Panic;

/// Fallback wrapper for the owned-value path.
///
/// Reached via `Deref` from [`Wrap`] when the value is not a reference to a `Sized` type.
#[doc(hidden)]
pub struct Fallback<T>(pub Cell<Option<T>>);

/// Primary wrapper used by the `assert_that!` macro.
///
/// For reference types `&'a T` where `T: Sized`, the inherent [`into_assert_that`](Wrap::into_assert_that)
/// method fires first (before deref to [`Fallback`]), producing an [`AssertThat`] with [`Actual::Borrowed`].
#[doc(hidden)]
pub struct Wrap<T> {
    pub inner: Fallback<T>,
}

// Inherent impl for reference types — tried FIRST by method resolution.
// The implicit `Sized` bound on `T` is intentional: unsized targets like `str`
// and `Path` fall through to the `Fallback` (owned) path, matching existing
// assertion trait impls like `StrSliceAssertions for AssertThat<'_, &str, M>`.
impl<'a, T> Wrap<&'a T> {
    #[track_caller]
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'a, T, Panic> {
        AssertThat::new(Actual::Borrowed(
            self.inner
                .0
                .take()
                .expect("assertr: value already consumed"),
        ))
    }
}

// Deref to Fallback so that when the inherent method above doesn't match,
// method resolution finds `Fallback::into_assert_that` via deref.
impl<T> Deref for Wrap<T> {
    type Target = Fallback<T>;

    fn deref(&self) -> &Fallback<T> {
        &self.inner
    }
}

// Fallback impl for owned values.
// Uses a free lifetime parameter `'t` so the caller's inference picks a
// lifetime short enough for `T: 't` (important when `T` borrows locals,
// e.g. iterators). The `Actual::Owned` variant doesn't actually use `'t`,
// so this is sound for any `'t` satisfying the well-formedness bound.
impl<T> Fallback<T> {
    #[track_caller]
    #[must_use]
    pub fn into_assert_that<'t>(&self) -> AssertThat<'t, T, Panic>
    where
        T: 't,
    {
        AssertThat::new(Actual::Owned(
            self.0.take().expect("assertr: value already consumed"),
        ))
    }
}

#[cfg(test)]
mod tests {
    mod owned_values {
        use crate::prelude::*;

        #[test]
        fn works_with_integer() {
            assert_that!(42).is_equal_to(42);
        }

        #[test]
        fn works_with_string() {
            assert_that!(String::from("hello")).is_equal_to("hello".to_string());
        }

        #[test]
        fn works_with_vec() {
            assert_that!(vec![1, 2, 3]).has_length(3);
        }

        #[test]
        fn works_with_bool() {
            assert_that!(true).is_true();
        }

        #[test]
        fn works_with_option() {
            assert_that!(Some(42)).is_some().is_equal_to(42);
        }

        #[test]
        fn works_with_result() {
            assert_that!(Result::<i32, ()>::Ok(42))
                .is_ok()
                .is_equal_to(42);
        }
    }

    mod unsized_reference_targets {
        use crate::prelude::*;

        #[test]
        fn works_with_str_slice() {
            // &str has unsized target `str`, so goes through Fallback (owned) path
            // producing AssertThat<'_, &str, Panic> — same as assert_that("hello")
            assert_that!("hello").starts_with("hel");
        }

        #[cfg(feature = "std")]
        #[test]
        fn works_with_path() {
            use std::path::Path;
            let path = Path::new("foo/bar.rs");
            // &Path has unsized target `Path`, goes through Fallback path
            assert_that!(path).has_file_name("bar.rs");
        }
    }

    mod borrowed_values {
        use crate::prelude::*;

        #[test]
        fn works_with_borrowed_integer() {
            let value = 42;
            assert_that!(&value).is_equal_to(42);
            // value is still usable
            let _ = value;
        }

        #[test]
        fn works_with_borrowed_string() {
            let value = String::from("hello");
            assert_that!(&value).is_equal_to("hello".to_string());
            // value is still usable (not moved)
            let _ = value;
        }

        #[test]
        fn works_with_borrowed_vec() {
            let value = vec![1, 2, 3];
            assert_that!(&value).has_length(3);
            let _ = value;
        }

        #[test]
        fn works_with_borrowed_option() {
            let value = Some(42);
            assert_that!(&value).is_some().is_equal_to(42);
            let _ = value;
        }

        #[cfg(feature = "std")]
        #[test]
        fn works_with_borrowed_mutex() {
            use std::sync::Mutex;
            let mutex = Mutex::new(42);
            let guard = mutex.lock().expect("lock");
            assert_that!(&mutex).is_locked();
            drop(guard);
        }

        #[test]
        fn works_with_variable_holding_reference() {
            let value = 42;
            let r: &i32 = &value;
            // r is already a reference — autoref specialization detects this
            assert_that!(r).is_equal_to(42);
        }
    }

    mod chaining {
        use crate::prelude::*;

        #[test]
        fn allows_chaining_multiple_assertions() {
            assert_that!(42).is_equal_to(42).is_not_equal_to(43);
        }
    }

    mod capture_mode {
        use crate::prelude::*;

        #[test]
        fn works_with_capture() {
            let failures = assert_that!(42)
                .with_capture()
                .is_equal_to(43)
                .capture_failures();
            assert_that!(failures).has_length(1);
        }
    }
}
