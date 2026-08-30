use std::ffi::OsStr;
use std::process::Command;

use alloc::{format, string::String, vec::Vec};
use core::fmt::Write;
use indoc::writedoc;

use crate::assertions::collection::CollectionStyle;
use crate::mode::Mode;
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
            let details = [format!("Consumed {} element(s).", actual.len())];
            let actual = self.render_values(&actual, CollectionStyle::List);
            let expected = self.render_value(expected);
            self.fail_with_details(details, |w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    does not contain expected: {expected:#?}
                "}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
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
                Actual: [
                    "--bar",
                ]
                
                does not contain expected: "help"

                Details: [
                    Consumed 1 element(s).,
                ]
                -------- assertr --------
            "#});
        }
    }
}
