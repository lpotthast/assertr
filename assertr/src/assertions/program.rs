use crate::AssertThat;
use crate::mode::{Mode, Panic};
use crate::tracking::AssertionTracking;
use alloc::borrow::Cow;
use indoc::writedoc;
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::PathBuf;

/// The name of a program. E.g. "ls", "sh", "chrome", ...
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program<'a>(Cow<'a, OsStr>);

impl<'a> Program<'a> {
    pub fn new(program: impl Into<Cow<'a, OsStr>>) -> Self {
        Program(program.into())
    }
}

impl<'a> From<&'a str> for Program<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(OsStr::new(value)))
    }
}

impl AsRef<OsStr> for Program<'_> {
    fn as_ref(&self) -> &OsStr {
        &self.0
    }
}

pub trait ProgramAssertions<'t, 'a, M: Mode> {
    /// Check that the program exists (equivalent to doing a `which {program}` check on unix).
    fn exists(self) -> AssertThat<'t, Program<'a>, M>;
}

pub trait ProgramAssertionsRequiringPanicMode<'t> {
    /// Check that the program exists (equivalent to doing a `which {program}` check on unix).
    ///
    /// Terminal operation, automatically mapping to the found `PathBuf` on success.
    ///
    /// This is only available in [`Panic`] mode, os we rely on the assertion panic for the mapping
    /// to only happen in the success case!
    fn exists_and(self) -> AssertThat<'t, PathBuf, Panic>;
}

impl<'a, 't, M: Mode> ProgramAssertions<'t, 'a, M> for AssertThat<'t, Program<'a>, M> {
    #[track_caller]
    fn exists(self) -> AssertThat<'t, Program<'a>, M> {
        self.track_assertion();
        let program = self.actual().as_ref();
        let found = which::which(program);

        if let Err(err) = &found {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected program: {program:?}

                    to exist, but it could not be found.

                    Reason: "{err}"
                "#}
            });
        }

        self
    }
}

impl<'t, 'a> ProgramAssertionsRequiringPanicMode<'t> for AssertThat<'t, Program<'a>, Panic> {
    #[track_caller]
    fn exists_and(self) -> AssertThat<'t, PathBuf, Panic> {
        self.track_assertion();
        let program = self.actual().as_ref();
        let found = which::which(program);

        if let Err(err) = &found {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected program: {program:?}

                    to exist, but it could not be found.

                    Reason: "{err}"
                "#}
            });
        }

        // Note: This will fail in capturing mode!
        self.map_owned(|_| found.expect("present"))
    }
}

#[cfg(test)]
mod tests {
    mod exists {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        #[test]
        fn succeeds_when_existent() {
            assert_that(Program::from("ls")).exists();
        }

        #[tokio::test]
        async fn panics_when_not_existent() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;

            assert_that_panic_by(|| {
                assert_that(Program::from("someNonexistentProgram"))
                    .with_location(false)
                    .exists()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected program: "someNonexistentProgram"

                    to exist, but it could not be found.

                    Reason: "cannot find binary path"
                    -------- assertr --------
                "#});

            drop(rw_lock_write_guard);
        }
    }

    mod exists_and {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

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
            assert_that(Program::from("ls"))
                .exists_and()
                .has_debug_value(expected_ls_location());
        }

        #[tokio::test]
        async fn panics_when_not_existent() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;

            assert_that_panic_by(|| {
                assert_that(Program::from("ls"))
                    .with_location(false)
                    .exists_and()
                    .has_debug_value("/some/unexpected/location/ls");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: "/some/unexpected/location/ls"

                      Actual: "/bin/ls"
                    -------- assertr --------
                "#});

            drop(rw_lock_write_guard);
        }
    }
}
