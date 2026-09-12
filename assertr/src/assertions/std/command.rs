use crate::failure::FailureKind;
use crate::mode::Mode;
use crate::renderer::GroupStyle;
use crate::{AssertThat, ValueRenderer};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use alloc::vec::Vec;
use std::ffi::OsStr;
use std::process::Command;

/// Checks command arguments and retains their observed views on rejection.
pub struct HasArg<E>(E);
impl<E, R> Expectation<Command, R> for HasArg<E>
where
    E: AsRef<OsStr>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Command: 'a;
    type Rejection<'a>
        = (Vec<&'a OsStr>, &'a OsStr)
    where
        Self: 'a,
        Command: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Command,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let args: Vec<&OsStr> = actual.get_args().collect();
        let expected = self.0.as_ref();
        if args.contains(&expected) {
            Ok(())
        } else {
            Err((args, expected))
        }
    }
}
impl<E, R> ExpectationDiagnostics<Command, R> for HasArg<E>
where
    E: AsRef<OsStr>,
    R: ValueRenderer<OsStr>,
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Command, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("contains")
                .expected(render.value(self.0.as_ref())),
            Some((_, (args, expected))) => failure
                .actual(render.borrowed_values::<OsStr, _>(&args, GroupStyle::List))
                .relation("does not contain")
                .expected(render.value(expected)),
        }
    }
}
impl<E> HasArg<E> {
    /// Expects this argument.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

/// Assertions for process commands.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait CommandAssertions<R = crate::DebugRenderer> {
    /// Asserts that the command contains `expected` in its argument list.
    fn has_arg(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>;
}

impl<M: Mode, R> CommandAssertions<R> for AssertThat<'_, Command, M, R> {
    #[track_caller]
    fn has_arg(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>,
    {
        self.apply_assertion(HasArg::new(expected))
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use std::process::Command;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Command, Panic, NoRenderer>
                    => CommandAssertions<NoRenderer>
            );
        }

        #[test]
        fn failures_render_arguments_with_the_active_renderer() {
            let mut command = Command::new("program");
            command.arg("--actual");

            let failures = assert_that!(command)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.has_arg("--expected"));

            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
        }
    }

    mod has_arg {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::process::Command;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar");
            cmd.must().have_arg("--bar");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar");
            assert_caller_location!(assert_that!(cmd), has_arg("help"));
        }

        #[test]
        fn succeeds_when_arg_present() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar").arg("--baz");

            assert_that!(cmd).has_arg("--bar").has_arg("--baz");
        }

        #[test]
        fn panics_when_arg_is_not_present() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar");

            assert_that_panic_by(|| {
                assert_that!(cmd).with_location(false).has_arg("help");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `cmd`

                Actual: [
                    "--bar",
                ]

                does not contain

                Expected: "help"
                -------- assertr --------
            "#});
        }
    }
}
