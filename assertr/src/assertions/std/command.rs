use std::ffi::OsStr;
use std::process::Command;

use alloc::vec::Vec;

use crate::failure::FailureKind;
use crate::mode::Mode;
use crate::renderer::GroupStyle;
use crate::{AssertThat, ValueRenderer};

/// Assertions for process commands.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
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
        self.track_assertion();
        let actual: Vec<&OsStr> = self.actual().get_args().collect();
        let expected = expected.as_ref();
        if !actual.contains(&expected) {
            self.failure(FailureKind::Membership)
                .actual(
                    self.render()
                        .borrowed_values::<OsStr, _>(&actual, GroupStyle::List),
                )
                .relation("does not contain")
                .expected(self.render().value(expected))
                .raise();
        }
        self
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
