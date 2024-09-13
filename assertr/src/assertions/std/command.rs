use std::ffi::OsStr;
use std::process::Command;

use crate::mode::Mode;
use crate::prelude::IntoIteratorAssertions;
use crate::AssertThat;

pub trait CommandAssertions {
    fn has_arg(self, expected: impl AsRef<OsStr>) -> Self;
}

impl<'t, M: Mode> CommandAssertions for AssertThat<'t, Command, M> {
    fn has_arg(self, expected: impl AsRef<OsStr>) -> Self {
        self.derive(|it| it.get_args().into_iter().collect::<Vec<_>>())
            .contains(expected.as_ref());
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
        fn succeeds_when_arg_present() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar");

            assert_that(cmd).has_arg("--bar");
        }

        #[test]
        fn panic_when_arg_is_not_present() {
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
