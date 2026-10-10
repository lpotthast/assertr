//! Assertions for resolving executable programs.

use alloc::borrow::Cow;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use crate::{
    Actual, AssertThat,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::{Mode, Panic},
    renderer::{DebugRenderer, ValueRenderer},
};

/// Resolves an executable program once, returning its path or lookup error.
#[derive(Debug, Clone, Copy)]
pub struct Exists;
impl<'p, R> Expectation<Program<'p>, R> for Exists
where
    R: ValueRenderer<Program<'p>> + ValueRenderer<which::Error>,
{
    type Success<'a>
        = PathBuf
    where
        Self: 'a,
        Program<'p>: 'a;
    type Rejection<'a>
        = which::Error
    where
        Self: 'a,
        Program<'p>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Program<'p>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        which::which(actual.as_ref())
    }

    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Program<'p>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure.relation("can be resolved"),
            Some((actual, error)) => failure
                .actual(render.value(actual))
                .relation("cannot be resolved")
                .fact(Fact::labelled("Reason", render.value(&error))),
        }
    }
}

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
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ProgramAssertions<'a, R = DebugRenderer> {
    /// Asserts that [`which::which`] resolves the program.
    fn exists(self) -> Self
    where
        R: ValueRenderer<Program<'a>> + ValueRenderer<which::Error>;
}

/// Panic-mode assertions that project a [`Program`] to its resolved path.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ProgramExtractAssertions<'t, 'a, R = DebugRenderer> {
    /// Asserts that [`which::which`] resolves the program, then returns an assertion over the
    /// resulting [`PathBuf`].
    ///
    /// This projection is available only in [`Panic`] mode because failure cannot produce a path.
    fn resolved_path(self) -> AssertThat<'t, PathBuf, Panic, R>
    where
        R: ValueRenderer<Program<'a>> + ValueRenderer<which::Error>;
}

impl<'a, M: Mode, R> ProgramAssertions<'a, R> for AssertThat<'_, Program<'a>, M, R> {
    #[track_caller]
    fn exists(self) -> Self
    where
        R: ValueRenderer<Program<'a>> + ValueRenderer<which::Error>,
    {
        self.matches(Exists)
    }
}

impl<'t, 'a, R> ProgramExtractAssertions<'t, 'a, R> for AssertThat<'t, Program<'a>, Panic, R> {
    #[track_caller]
    fn resolved_path(self) -> AssertThat<'t, PathBuf, Panic, R>
    where
        R: ValueRenderer<Program<'a>> + ValueRenderer<which::Error>,
    {
        let path = self.require(&Exists);
        self.map(|_| Actual::Owned(path))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            Program::from("ls").must().exist();
        }
    }

    const MISSING: &str = "assertr-private-missing-executable-987";

    mod renderer_contract {
        use super::*;
        use crate::test_support::{
            CustomValueRenderer, NoRenderer, RedactingRenderer, assert_custom_value,
            assert_redacted, assert_trait_impl,
        };

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Program<'static>, Panic, NoRenderer>
                    => ProgramAssertions<'static, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Program<'static>, Panic, NoRenderer>
                    => ProgramExtractAssertions<'static, 'static, NoRenderer>
            );
        }

        #[test]
        fn lookup_errors_keep_the_original_type_and_can_be_redacted() {
            let failures = assert_that!(Program::from(MISSING))
                .with_renderer(CustomValueRenderer)
                .capture(ProgramAssertions::exists);
            assert_custom_value(
                &failures[0].facts[0].value,
                &which::Error::CannotFindBinaryPath,
            );
            let failures = assert_that!(Program::from(MISSING))
                .with_renderer(RedactingRenderer)
                .capture(ProgramAssertions::exists);
            assert_redacted(&failures[0], &[MISSING, "CannotFindBinaryPath"]);
        }

        #[test]
        fn checking_and_extracting_failures_use_the_active_renderer() {
            let program = format!("custom(Program({MISSING:?}))");
            assert_that!(|| {
                assert_that!(Program::from(MISSING))
                    .with_renderer(CustomValueRenderer)
                    .exists();
            })
            .panics()
            .has_message()
            .contains(program.as_str())
            .contains("Reason: custom(CannotFindBinaryPath)");
            assert_that!(|| {
                let _ = assert_that!(Program::from(MISSING))
                    .with_renderer(CustomValueRenderer)
                    .resolved_path();
            })
            .panics()
            .has_message()
            .contains(program.as_str())
            .contains("Reason: custom(CannotFindBinaryPath)");
            assert_that!(|| {
                let _ = assert_that!(Program::from(MISSING))
                    .with_renderer(RedactingRenderer)
                    .resolved_path();
            })
            .panics()
            .has_message()
            .contains("Actual: <redacted>")
            .contains("Reason: <redacted>");
        }
    }

    #[test]
    fn programs_convert_from_owned_and_borrowed_strings() {
        use alloc::borrow::Cow;
        use std::ffi::{OsStr, OsString};

        let ls = Program::from("ls");
        for program in [
            Program::new(OsStr::new("ls")),
            Program::new(OsString::from("ls")),
            Program::from(String::from("ls")),
            Program::from(OsStr::new("ls")),
            Program::from(OsString::from("ls")),
            Program::from(Cow::Borrowed("ls")),
            Program::from(Cow::<str>::Owned("ls".to_owned())),
        ] {
            assert_that!(program).is_equal_to(&ls);
        }
    }

    mod exists {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Program::from(MISSING)), exists());
        }

        #[test]
        fn succeeds_when_existent() {
            assert_that!(Program::from("ls")).exists();
        }

        #[test]
        fn panics_when_not_existent() {
            assert_that!(|| {
                assert_that_owned!(Program::from("someNonexistentProgram"))
                    .with_location(false)
                    .exists()
            })
            .panics()
            .has_message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `Program::from("someNonexistentProgram")`

                Actual: Program(
                    "someNonexistentProgram",
                )

                cannot be resolved

                Details:
                  - Reason: CannotFindBinaryPath
                -------- assertr --------
            "#});
        }
    }

    mod resolved_path {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Program::from(MISSING)), resolved_path());
        }

        #[test]
        fn continues_on_the_resolved_path() {
            assert_that!(Program::from("ls"))
                .resolved_path()
                .is_absolute()
                .is_a_file()
                .has_file_stem("ls");
        }
    }
}
