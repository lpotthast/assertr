//! Assertions for resolving executable programs.

use crate::mode::{Mode, Panic};
use crate::{Actual, AssertThat, ValueRenderer, failure::FailureKind};
use alloc::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// A program name or path to resolve with [`which::which`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program<'a>(Cow<'a, OsStr>);

impl<'a> Program<'a> {
    /// Creates a program name from owned or borrowed platform string data.
    pub fn new(program: impl Into<Cow<'a, OsStr>>) -> Self {
        Program(program.into())
    }
}

impl<'a> From<&'a str> for Program<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(OsStr::new(value)))
    }
}

impl From<String> for Program<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(OsString::from(value)))
    }
}

impl<'a> From<&'a OsStr> for Program<'a> {
    fn from(value: &'a OsStr) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<OsString> for Program<'_> {
    fn from(value: OsString) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for Program<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        match value {
            Cow::Borrowed(v) => Self::from(v),
            Cow::Owned(v) => Self::from(v),
        }
    }
}

impl AsRef<OsStr> for Program<'_> {
    fn as_ref(&self) -> &OsStr {
        &self.0
    }
}

/// Non-extracting assertions for [`Program`] subjects.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ProgramAssertions<'t, 'a, M: Mode, R = crate::DebugRenderer> {
    /// Asserts that [`which::which`] resolves the program.
    fn exists(self) -> AssertThat<'t, Program<'a>, M, R>
    where
        R: ValueRenderer<Program<'a>>;
}

/// Panic-mode assertions that project a [`Program`] to its resolved path.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ProgramAssertionsRequiringPanicMode<'t, R = crate::DebugRenderer> {
    /// The program subject rendered in failure diagnostics.
    type Subject;

    /// Asserts that [`which::which`] resolves the program, then returns an assertion over the
    /// resulting [`PathBuf`].
    ///
    /// This projection is available only in [`Panic`] mode because failure cannot produce a path.
    fn exists_and(self) -> AssertThat<'t, PathBuf, Panic, R>
    where
        R: ValueRenderer<Self::Subject>;
}

impl<'a, 't, M: Mode, R> ProgramAssertions<'t, 'a, M, R> for AssertThat<'t, Program<'a>, M, R> {
    #[track_caller]
    fn exists(self) -> AssertThat<'t, Program<'a>, M, R>
    where
        R: ValueRenderer<Program<'a>>,
    {
        self.track_assertion();
        let program = self.actual().as_ref();
        let found = which::which(program);

        if let Err(err) = &found {
            self.failure(FailureKind::Other)
                .actual(self.render().value(self.actual()))
                .relation("was not found")
                .fact("Reason", format_args!("{err}"))
                .raise();
        }

        self
    }
}

impl<'a, 't, R> ProgramAssertionsRequiringPanicMode<'t, R>
    for AssertThat<'t, Program<'a>, Panic, R>
{
    type Subject = Program<'a>;

    #[track_caller]
    fn exists_and(self) -> AssertThat<'t, PathBuf, Panic, R>
    where
        R: ValueRenderer<Program<'a>>,
    {
        self.track_assertion();
        let program = self.actual().as_ref();
        let found = which::which(program);

        if let Err(err) = &found {
            self.failure(FailureKind::Other)
                .actual(self.render().value(self.actual()))
                .relation("was not found")
                .fact("Reason", format_args!("{err}"))
                .raise();
        }

        self.map(|_| Actual::Owned(found.expect("present")))
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::assertions::program::Program;
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Program<'static>, Panic, NoRenderer>
                    => ProgramAssertions<'static, 'static, Panic, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Program<'static>, Panic, NoRenderer>
                    => ProgramAssertionsRequiringPanicMode<'static, NoRenderer>
            );
        }

        #[test]
        fn checking_and_extracting_failures_use_the_active_renderer() {
            const MISSING: &str = "assertr-renderer-test-program-that-does-not-exist";

            let failures = assert_that!(Program::from(MISSING))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(ProgramAssertions::exists);
            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);

            assert_that_panic_by(|| {
                assert_that!(Program::from(MISSING))
                    .with_renderer(SentinelRenderer)
                    .with_location(false)
                    .exists_and();
            })
            .has_type::<String>()
            .contains(SENTINEL);
        }
    }

    mod program_construction {
        use crate::prelude::*;
        use alloc::borrow::Cow;
        use std::ffi::OsStr;
        use std::ffi::OsString;

        #[test]
        fn new_os_str() {
            let _ = Program::new(OsStr::new("ls"));
        }

        #[test]
        fn new_os_string() {
            let _ = Program::new(OsString::from("ls"));
        }

        #[test]
        fn from_str() {
            let _ = Program::from("ls");
        }

        #[test]
        fn from_string() {
            let _ = Program::from(String::from("ls"));
        }

        #[test]
        fn from_os_str() {
            let _ = Program::from(OsStr::new("ls"));
        }

        #[test]
        fn from_os_string() {
            let _ = Program::from(OsString::from("ls"));
        }

        #[test]
        fn from_cow_str() {
            let _ = Program::from(Cow::Borrowed("ls"));
        }

        #[test]
        fn from_cow_string() {
            let _ = Program::from(Cow::Owned("ls".to_owned()));
        }
    }

    mod exists {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Program::from("ls").must().exist();
        }

        #[test]
        fn succeeds_when_existent() {
            assert_that!(Program::from("ls")).exists();
        }

        #[tokio::test]
        async fn panics_when_not_existent() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;

            assert_that_panic_by(|| {
                assert_that_owned!(Program::from("someNonexistentProgram"))
                    .with_location(false)
                    .exists()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `Program::from("someNonexistentProgram")`

                    Actual: Program(
                        "someNonexistentProgram",
                    )

                    was not found

                    Details:
                      - Reason: cannot find binary path
                    -------- assertr --------
                "#});

            drop(rw_lock_write_guard);
        }
    }

    mod exists_and {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Program::from("ls").must_owned().exist_and();
        }

        #[cfg(target_os = "linux")]
        fn expected_ls_location() -> &'static str {
            "/usr/bin/ls"
        }

        #[cfg(target_os = "macos")]
        fn expected_ls_location() -> &'static str {
            "/bin/ls"
        }

        #[test]
        fn succeeds_when_existent() {
            assert_that!(Program::from("ls"))
                .exists_and()
                .has_debug_value(expected_ls_location());
        }

        #[tokio::test]
        async fn panics_when_not_existent() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;

            assert_that_panic_by(|| {
                assert_that!(Program::from("ls"))
                    .with_location(false)
                    .exists_and()
                    .has_debug_value("/some/unexpected/location/ls");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `Program::from("ls")`

                    Expected: "/some/unexpected/location/ls"

                      Actual: "{}"
                    -------- assertr --------
                "#, expected_ls_location()});

            drop(rw_lock_write_guard);
        }
    }
}
