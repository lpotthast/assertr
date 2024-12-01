use std::ffi::OsStr;
use std::process::Command;

use crate::mode::Mode;
use crate::prelude::IteratorAssertions;
use crate::AssertThat;

pub trait CommandAssertions {
    fn has_arg(self, expected: impl AsRef<OsStr>) -> Self;
}

impl<'t, M: Mode> CommandAssertions for AssertThat<'t, Command, M> {
    fn has_arg(self, expected: impl AsRef<OsStr>) -> Self {
        self.derive(|it| it.get_args()).contains(expected.as_ref());
        self
    }
}

#[cfg(test)]
mod tests {
    mod has_arg {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::ffi::{OsStr, OsString};
        use std::process::Command;

        #[test]
        fn succeeds_when_arg_present() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar").arg("--baz");

            assert_that(cmd).has_arg("--bar").has_arg("--baz");
        }

        #[test]
        fn panics_when_arg_is_not_present() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar");

            assert_that_panic_by(|| {
                assert_that(cmd).with_location(false).has_arg("help");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: [
                    "--bar",
                ]
                
                does not contain expected: "help"
                -------- assertr --------
            "#});
        }
    }
}
