//! Entry points of the `assert_that!` and `assert_that_owned!` macros.
//!
//! **Do not name these items directly.** They are reachable only because the macros expand to them
//! through `$crate`.

use core::ops::Deref;

use crate::{AssertThat, actual::Actual, mode::Panic};

/// Fallback wrapper for the general borrowed path.
///
/// Reached via `Deref` from [`Wrap`] when no inherent method of `Wrap` applies: the asserted
/// expression is neither a reference to a `Sized` target nor a reference to one of the unsized
/// targets with a dedicated impl. Holds the borrow of the asserted expression.
pub struct Fallback<T>(pub T);

/// Primary wrapper used by the `assert_that!` macro. Holds a borrow of the asserted expression.
///
/// When the expression is itself a reference `&'a T` (or `&'a mut T`) to a `Sized` target, the
/// inherent [`into_assert_that`](Wrap::into_assert_that) methods fire first (before deref to
/// [`Fallback`]) and unwrap one reference level, so that `assert_that!(&value)` yields an
/// `AssertThat<Value>` borrowing with the reference's own lifetime.
pub struct Wrap<T> {
    pub inner: Fallback<T>,
}

// Inherent impl for shared-reference expressions, tried first by method resolution. The implicit
// `Sized` bound on `T` is intentional. References to unsized targets keep the reference itself as
// the subject, through the dedicated impls below for `str`, slices, `CStr`, `Path`, and `OsStr`,
// or through `Fallback` for other targets. For example, an `&str` subject satisfies the
// `S: AsRef<str>` bound of the blanket `StrAssertions` impl.
impl<'a, T> Wrap<&'_ &'a T> {
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'a, T, Panic> {
        AssertThat::new(Actual::Borrowed(*self.inner.0))
    }
}

// Inherent impl for mutable-reference expressions: reborrows immutably. The reborrow is limited to
// the wrapper's own lifetime, so the resulting assertion must be consumed within the statement.
impl<'x, T> Wrap<&'x &'_ mut T> {
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'x, T, Panic> {
        AssertThat::new(Actual::Borrowed(&**self.inner.0))
    }
}

// Shared references to other unsized targets, such as trait objects, cannot use the `T: Sized`
// inherent impl above or the concrete DST impls below. A temporary reference expression therefore
// remains limited to the enclosing statement through `Fallback`:
//
// ```rust,ignore
// let value = 42;
// let assertion = assert_that!(&value as &dyn core::fmt::Debug);
// assertion.has_debug_string("42"); // E0716: the temporary reference was dropped.
// ```
//
// Bind the reference first, or use `assert_that_owned!`, to retain the assertion beyond the
// statement.

// Inherent impls for references to unsized targets. The reference stays the subject (see the
// `Sized` note above), but it is copied out of the macro's temporary and owned by the assertion.
// The assertion therefore lives as long as the referenced data, not only until the end of the
// statement. Without these, `let a = assert_that!(text.as_str());` fails with E0716 because the
// `Fallback` path borrows the temporary `&str` produced by the call.
impl<'a> Wrap<&'_ &'a str> {
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'a, &'a str, Panic> {
        AssertThat::new(Actual::Owned(*self.inner.0))
    }
}

impl<'a, T> Wrap<&'_ &'a [T]> {
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'a, &'a [T], Panic> {
        AssertThat::new(Actual::Owned(*self.inner.0))
    }
}

impl<'a> Wrap<&'_ &'a core::ffi::CStr> {
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'a, &'a core::ffi::CStr, Panic> {
        AssertThat::new(Actual::Owned(*self.inner.0))
    }
}

#[cfg(feature = "std")]
impl<'a> Wrap<&'_ &'a std::path::Path> {
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'a, &'a std::path::Path, Panic> {
        AssertThat::new(Actual::Owned(*self.inner.0))
    }
}

#[cfg(feature = "std")]
impl<'a> Wrap<&'_ &'a std::ffi::OsStr> {
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'a, &'a std::ffi::OsStr, Panic> {
        AssertThat::new(Actual::Owned(*self.inner.0))
    }
}

// Deref to Fallback so that when no inherent method above matches, method resolution finds
// `Fallback::into_assert_that` via deref.
impl<T> Deref for Wrap<T> {
    type Target = Fallback<T>;

    fn deref(&self) -> &Fallback<T> {
        &self.inner
    }
}

// Fallback impl: the borrow taken by the macro references the asserted place directly (keeping a
// named value usable afterwards) or a temporary that lives until the end of the enclosing statement
// (keeping assertions on literals and temporaries ergonomic).
impl<'t, T> Fallback<&'t T> {
    #[must_use]
    pub fn into_assert_that(&self) -> AssertThat<'t, T, Panic> {
        AssertThat::new(Actual::Borrowed(self.0))
    }
}

/// Entry point of the `assert_that_owned!` macro.
#[must_use]
pub fn owned<'t, T: 't>(value: T) -> AssertThat<'t, T, Panic> {
    AssertThat::new(Actual::Owned(value))
}

#[cfg(test)]
mod tests {
    mod borrowing_by_default {
        use crate::prelude::*;

        #[test]
        fn a_named_collection_remains_usable() {
            let value = vec![1, 2, 3];
            assert_that!(value).has_length(3);
            assert_that!(value.len()).is_equal_to(3);
        }

        #[test]
        // Consuming the literal-built value afterwards is the point of the test: it proves the
        // assertion only borrowed it.
        #[allow(clippy::unnecessary_literal_unwrap)]
        fn a_named_option_remains_usable() {
            let value = Some(String::from("v"));
            assert_that!(value).is_some();
            assert_that!(value.expect("present")).is_equal_to("v");
        }

        #[test]
        // Consuming the literal-built value afterwards is the point of the test: it proves the
        // assertion only borrowed it.
        #[allow(clippy::unnecessary_literal_unwrap)]
        fn a_named_result_remains_usable() {
            let value: Result<String, ()> = Ok(String::from("v"));
            assert_that!(value).is_ok();
            assert_that!(value.expect("ok")).is_equal_to("v");
        }
    }

    mod references_to_unsized_targets {
        use crate::prelude::*;

        #[test]
        fn a_str_from_a_method_call_outlives_the_statement() {
            let value = String::from("hello");
            let assertion = assert_that!(value.as_str());
            assertion.starts_with("hel");
        }

        #[test]
        fn a_slice_from_a_method_call_outlives_the_statement() {
            let value = vec![1, 2, 3];
            let assertion = assert_that!(value.as_slice());
            assertion.contains(2);
        }

        #[test]
        fn a_c_str_from_a_method_call_outlives_the_statement() {
            let value = alloc::ffi::CString::new("hello").expect("string contains no nul byte");
            let assertion = assert_that!(value.as_c_str());
            assertion.has_debug_string("\"hello\"");
        }

        #[test]
        #[cfg(feature = "std")]
        fn a_path_from_a_method_call_outlives_the_statement() {
            let value = std::path::PathBuf::from("/some/file.txt");
            let assertion = assert_that!(value.as_path());
            assertion.is_equal_to(std::path::Path::new("/some/file.txt"));
        }

        #[test]
        #[cfg(feature = "std")]
        fn an_os_str_from_a_method_call_outlives_the_statement() {
            let value = std::ffi::OsString::from("hello");
            let assertion = assert_that!(value.as_os_str());
            assertion.is_equal_to(std::ffi::OsStr::new("hello"));
        }

        #[test]
        #[cfg(feature = "std")]
        fn a_named_path_reference_keeps_path_assertions() {
            let path = std::path::Path::new("foo/bar.rs");
            assert_that!(path).has_file_name("bar.rs");
        }

        #[test]
        fn the_reference_stays_the_subject() {
            fn accepts_str(_: AssertThat<'_, &str, Panic>) {}
            fn accepts_slice(_: AssertThat<'_, &[i32], Panic>) {}
            fn accepts_c_str(_: AssertThat<'_, &core::ffi::CStr, Panic>) {}

            let text = String::from("hello");
            accepts_str(assert_that!(text.as_str()));
            accepts_str(assert_that!("literal"));

            let numbers = vec![1, 2, 3];
            accepts_slice(assert_that!(numbers.as_slice()));
            accepts_slice(assert_that!(&numbers[..]));

            let c_string = alloc::ffi::CString::new("hello").expect("string contains no nul byte");
            accepts_c_str(assert_that!(c_string.as_c_str()));
        }
    }

    mod owned_entry_point {
        use crate::prelude::*;

        #[cfg(feature = "std")]
        #[test]
        fn takes_ownership_for_consuming_assertions() {
            assert_that_owned!(|| panic!("boom"))
                .panics()
                .has_type::<&str>()
                .is_equal_to("boom");
        }

        #[test]
        fn takes_ownership_of_an_iterator() {
            assert_that_owned!([1, 2, 3].into_iter()).contains(2);
        }
    }

    mod literals_and_temporaries {
        use crate::prelude::*;

        #[test]
        fn extractions_continue_on_a_temporary() {
            assert_that!(Some(42)).some().is_equal_to(42);
            assert_that!(Result::<i32, ()>::Ok(42)).ok().is_equal_to(42);
        }
    }

    mod reference_expressions {
        use crate::prelude::*;

        #[test]
        fn borrowing_explicitly_is_equivalent_to_borrowing_by_default() {
            let value = String::from("hello");
            assert_that!(&value).is_equal_to("hello");
            assert_that!(value).is_equal_to("hello");
        }

        #[test]
        fn keeps_the_references_own_lifetime() {
            let value = 42;
            // The borrow is tied to `value`, not to the macro's internal temporaries, so the
            // assertion can outlive the statement that created it.
            let assertion = assert_that!(&value);
            assertion.is_equal_to(42);
        }

        #[test]
        fn works_with_a_mutable_reference() {
            let mut value = vec![1, 2, 3];
            let reference = &mut value;
            assert_that!(reference).has_length(3);
        }

        #[test]
        fn works_with_variable_holding_reference() {
            let value = 42;
            let r: &i32 = &value;
            // `r` is already a reference, so the `Wrap<&&T>` impl unwraps it.
            assert_that!(r).is_equal_to(42);
        }
    }

    mod evaluation {
        use crate::prelude::*;

        #[test]
        fn evaluates_the_actual_expression_once() {
            let mut evaluations = 0;

            assert_that!({
                evaluations += 1;
                Some(42)
            })
            .some()
            .is_equal_to(42);

            assert_that!(evaluations).is_equal_to(1);
        }
    }
}
