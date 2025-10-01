use crate::{AssertThat, Mode, tracking::AssertionTracking};
use indoc::writedoc;
use std::fmt::Write;
use std::path::PathBuf;
use std::{ffi::OsStr, path::Path};

pub trait PathAssertions {
    fn exists(self) -> Self;
    fn does_not_exist(self) -> Self;
    fn is_a_file(self) -> Self;
    fn is_a_directory(self) -> Self;
    fn is_a_symlink(self) -> Self;
    fn has_a_root(self) -> Self;
    fn is_relative(self) -> Self;
    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self;
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self;
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self;
    fn starts_with(self, expected: impl AsRef<Path>) -> Self;
    fn ends_with(self, expected: impl AsRef<Path>) -> Self;
}

impl<M: Mode> PathAssertions for AssertThat<'_, PathBuf, M> {
    #[track_caller]
    fn exists(self) -> Self {
        self.derive(|it| it.as_path()).exists();
        self
    }

    #[track_caller]
    fn does_not_exist(self) -> Self {
        self.derive(|it| it.as_path()).does_not_exist();
        self
    }

    #[track_caller]
    fn is_a_file(self) -> Self {
        self.derive(|it| it.as_path()).is_a_file();
        self
    }

    #[track_caller]
    fn is_a_directory(self) -> Self {
        self.derive(|it| it.as_path()).is_a_directory();
        self
    }

    #[track_caller]
    fn is_a_symlink(self) -> Self {
        self.derive(|it| it.as_path()).is_a_symlink();
        self
    }

    #[track_caller]
    fn has_a_root(self) -> Self {
        self.derive(|it| it.as_path()).has_a_root();
        self
    }

    #[track_caller]
    fn is_relative(self) -> Self {
        self.derive(|it| it.as_path()).is_relative();
        self
    }

    #[track_caller]
    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self {
        self.derive(|it| it.as_path()).has_file_name(expected);
        self
    }

    #[track_caller]
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self {
        self.derive(|it| it.as_path()).has_file_stem(expected);
        self
    }

    #[track_caller]
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self {
        self.derive(|it| it.as_path()).has_extension(expected);
        self
    }
    #[track_caller]
    fn starts_with(self, expected: impl AsRef<Path>) -> Self {
        self.derive(|it| it.as_path()).starts_with(expected);
        self
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<Path>) -> Self {
        self.derive(|it| it.as_path()).ends_with(expected);
        self
    }
}

impl<M: Mode> PathAssertions for AssertThat<'_, &Path, M> {
    #[track_caller]
    fn exists(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        match actual.try_exists() {
            Ok(true) => {}
            Ok(false) => {
                self.fail(|w: &mut String| {
                    writedoc! {w, r#"
                        Expected: {actual:#?}

                        to exist, but it does not!
                    "#}
                });
            }
            Err(err) => {
                self.fail(|w: &mut String| {
                    writedoc! {w, r#"
                        Expected: {actual:#?}

                        to exist, but it does not!

                        Encountered std::io::Error: {err:#?}
                    "#}
                });
            }
        }
        self
    }

    #[track_caller]
    fn does_not_exist(self) -> Self {
        self.track_assertion();
        let actual = self.actual();

        match actual.try_exists() {
            Ok(true) => {
                self.fail(|w: &mut String| {
                    writedoc! {w, r#"
                        Expected: {actual:#?}

                        to not exist, but it does!
                    "#}
                });
            }
            Ok(false) => {}
            Err(_err) => {}
        }
        self
    }

    #[track_caller]
    fn is_a_file(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_file() {
            let exists = actual.exists();
            let is_dir = actual.is_dir();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {actual:#?}

                    to be a file, but it is not!

                    The path exists: {exists}
                    The path is a directory: {is_dir}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_a_directory(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_dir() {
            let exists = actual.exists();
            let is_file = actual.is_file();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {actual:#?}

                    to be a directory, but it is not!

                    The path exists: {exists}
                    The path is a file: {is_file}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_a_symlink(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_symlink() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {actual:#?}

                    to be a symlink, but it is not!
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn has_a_root(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.has_root() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {actual:#?}

                    to be a root-path, but it is not!
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_relative(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_relative() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {actual:#?}

                    to be a relative path, but it is not!
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let actual_file_name = actual.file_name();
        let expected_file_name = expected.as_ref();
        if let Some(actual_file_name) = actual_file_name {
            if actual_file_name != expected_file_name {
                self.fail(|w: &mut String| {
                    writedoc! {w, r#"
                        Path: {actual:?}

                        Expected filename: {expected_file_name:#?}
                          Actual filename: {actual_file_name:#?}
                    "#}
                });
            }
        }
        self
    }

    #[track_caller]
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let actual_file_stem = actual.file_stem();
        let expected_file_stem = expected.as_ref();
        if let Some(actual_file_stem) = actual_file_stem {
            if actual_file_stem != expected_file_stem {
                self.fail(|w: &mut String| {
                    writedoc! {w, r#"
                        Path: {actual:?}

                        Expected filestem: {expected_file_stem:#?}
                          Actual filestem: {actual_file_stem:#?}
                    "#}
                });
            }
        }
        self
    }

    #[track_caller]
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let actual_extension = actual.extension();
        let expected_extension = expected.as_ref();
        if let Some(actual_extension) = actual_extension {
            if actual_extension != expected_extension {
                self.fail(|w: &mut String| {
                    writedoc! {w, r#"
                        Path: {actual:?}

                        Expected extension: {expected_extension:#?}
                          Actual extension: {actual_extension:#?}
                    "#}
                });
            }
        }
        self
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<Path>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let expected_prefix = expected.as_ref();
        if !actual.starts_with(expected_prefix) {
            self.add_detail_message("Only whole path components are matched!");
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Path: {actual:?}

                    Did not start with expected prefix: {expected_prefix:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<Path>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let expected_postfix = expected.as_ref();
        if !actual.ends_with(expected_postfix) {
            self.add_detail_message("Only whole path components are matched!");
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Path: {actual:?}

                    Did not end with expected postfix: {expected_postfix:#?}
                "#}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod path {
        mod exists {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;
            use std::path::Path;

            #[test]
            fn succeeds_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path.as_path())
                    .exists()
                    .map(|it| it.borrowed().to_str().unwrap_or_default().into())
                    .ends_with("src/assertions/std/path.rs");
            }

            #[test]
            fn panics_when_absent() {
                let path = Path::new("src/assertions/std/some-non-existing-file.rs");
                assert_that_panic_by(|| assert_that(path).with_location(false).exists())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: "src/assertions/std/some-non-existing-file.rs"

                        to exist, but it does not!
                        -------- assertr --------
                    "#});
            }
        }

        mod does_not_exist {
            use crate::prelude::*;
            use std::env;
            use std::path::Path;

            #[test]
            fn succeeds_when_absent() {
                let path = Path::new("../../foo/bar/baz.rs");
                assert_that(path).does_not_exist();
            }

            #[test]
            fn panics_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    assert_that(path.as_path())
                        .with_location(false)
                        .does_not_exist()
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Expected: \"")
                .contains("assertr/src/assertions/std/path.rs\"")
                .contains("to not exist, but it does!");
            }
        }

        mod is_file {
            use crate::prelude::*;
            use std::env;

            #[test]
            fn succeeds_when_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path.as_path()).is_a_file();
            }

            #[test]
            fn panics_when_not_a_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let dir = path.parent().unwrap();
                assert_that_panic_by(|| {
                    assert_that(dir)
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_file()
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Expected: \"")
                .contains("assertr/src/assertions/std\"")
                .contains("to be a file, but it is not!");
            }
        }

        mod is_directory {
            use std::env;
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_directory() {
                let path = env::current_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(Path::new(file!()).parent().expect("present"));
                assert_that(path.as_path()).is_a_directory();
            }

            #[test]
            fn panics_when_not_a_directory() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    assert_that(path.as_path())
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_directory()
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Expected: \"")
                .contains("assertr/src/assertions/std/path.rs\"")
                .contains("to be a directory, but it is not!")
                .contains("The path exists: true")
                .contains("The path is a file: true");
            }
        }

        mod is_symlink {
            // TODO: Add symlink tests.
            /*
            #[test]
            fn is_symlink_succeeds_when_directory() {
                let path = Path::new(file!());
                assert_that(path).is_symlink();
            }
            */
        }

        mod has_a_root {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_root() {
                let path = Path::new("/foo/bar/baz.rs");
                assert_that(path).has_a_root();
            }

            #[test]
            fn panics_when_relative() {
                let path = Path::new("foo/bar/baz.rs");
                assert_that_panic_by(|| assert_that(path).with_location(false).has_a_root())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: "foo/bar/baz.rs"

                        to be a root-path, but it is not!
                        -------- assertr --------
                    "#});
            }
        }

        mod is_relative {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_relative() {
                let path = Path::new("foo/bar/baz.rs");
                assert_that(path).is_relative();
            }

            #[test]
            fn panics_when_absolute() {
                let path = Path::new("/foo/bar/baz.rs");
                assert_that_panic_by(|| assert_that(path).with_location(false).is_relative())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: "/foo/bar/baz.rs"

                        to be a relative path, but it is not!
                        -------- assertr --------
                    "#});
            }
        }

        mod has_filename {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!());
                assert_that(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that(path)
                        .with_location(false)
                        .has_file_name("some.json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"
    
                        Expected filename: "some.json"
                          Actual filename: "path.rs"
                        -------- assertr --------
                    "#});
            }
        }

        mod has_file_stem {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!());
                assert_that(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that(path).with_location(false).has_file_stem("some")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"
    
                        Expected filestem: "some"
                          Actual filestem: "path"
                        -------- assertr --------
                    "#});
            }
        }

        mod has_extension {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!());
                assert_that(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that(path).with_location(false).has_extension("json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"
    
                        Expected extension: "json"
                          Actual extension: "rs"
                        -------- assertr --------
                    "#});
            }
        }

        mod starts_with {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_prefix() {
                let path = Path::new(file!());
                assert_that(path).starts_with("assertr/src");
            }

            #[test]
            fn panics_when_not_a_prefix() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that(path).with_location(false).starts_with("foobar")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"

                        Did not start with expected prefix: "foobar"

                        Details: [
                            Only whole path components are matched!,
                        ]
                        -------- assertr --------
                    "#});
            }

            #[test]
            fn panics_when_not_a_whole_segment_prefix() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that(path).with_location(false).starts_with("assert")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"

                        Did not start with expected prefix: "assert"

                        Details: [
                            Only whole path components are matched!,
                        ]
                        -------- assertr --------
                    "#});
            }
        }

        mod ends_with {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_postfix() {
                let path = Path::new(file!());
                assert_that(path).ends_with("std/path.rs");
            }

            #[test]
            fn panics_when_not_a_postfix() {
                let path = Path::new(file!());
                assert_that_panic_by(|| assert_that(path).with_location(false).ends_with("foobar"))
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"

                        Did not end with expected postfix: "foobar"

                        Details: [
                            Only whole path components are matched!,
                        ]
                        -------- assertr --------
                    "#});
            }

            #[test]
            fn panics_when_not_a_whole_segment_postfix() {
                let path = Path::new(file!());
                assert_that_panic_by(|| assert_that(path).with_location(false).ends_with("ath.rs"))
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"

                        Did not end with expected postfix: "ath.rs"

                        Details: [
                            Only whole path components are matched!,
                        ]
                        -------- assertr --------
                    "#});
            }
        }
    }

    mod path_buf {
        mod exists {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;
            use std::path::Path;

            #[test]
            fn succeeds_when_present() {
                let path = env::current_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(file!())
                    .to_owned();
                assert_that(path)
                    .exists()
                    .map(|it| it.unwrap_owned().display().to_string().into())
                    .ends_with("src/assertions/std/path.rs");
            }

            #[test]
            fn panics_when_absent() {
                let path = Path::new("src/assertions/std/some-non-existing-file.rs").to_owned();
                assert_that_panic_by(|| assert_that(path).with_location(false).exists())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: "src/assertions/std/some-non-existing-file.rs"

                        to exist, but it does not!
                        -------- assertr --------
                    "#});
            }
        }

        mod does_not_exist {
            use crate::prelude::*;
            use std::env;
            use std::path::Path;

            #[test]
            fn succeeds_when_absent() {
                let path = Path::new("../../foo/bar/baz.rs").to_owned();
                assert_that(path).does_not_exist();
            }

            #[test]
            fn panics_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| assert_that(path).with_location(false).does_not_exist())
                    .has_type::<String>()
                    .contains("-------- assertr --------")
                    .contains("Expected: \"")
                    .contains("assertr/src/assertions/std/path.rs\"")
                    .contains("to not exist, but it does!");
            }
        }

        mod is_file {
            use crate::prelude::*;
            use std::env;

            #[test]
            fn succeeds_when_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path).is_a_file();
            }

            #[test]
            fn panics_when_not_a_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let dir = path.parent().unwrap().to_owned();
                assert_that_panic_by(|| {
                    assert_that(dir)
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_file()
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Expected: \"")
                .contains("assertr/src/assertions/std\"")
                .contains("to be a file, but it is not!");
            }
        }

        mod is_directory {
            use std::env;
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_directory() {
                let path = env::current_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(Path::new(file!()).parent().expect("present"));
                assert_that(path).is_a_directory();
            }

            #[test]
            fn panics_when_not_a_directory() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    assert_that(path)
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_directory()
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Expected: \"")
                .contains("assertr/src/assertions/std/path.rs\"")
                .contains("to be a directory, but it is not!")
                .contains("The path exists: true")
                .contains("The path is a file: true");
            }
        }

        mod is_symlink {
            // TODO: Add symlink tests.
            /*
            #[test]
            fn is_symlink_succeeds_when_directory() {
                let path = Path::new(file!());
                assert_that(path).is_symlink();
            }
            */
        }

        mod has_a_root {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_root() {
                let path = Path::new("/foo/bar/baz.rs").to_owned();
                assert_that(path).has_a_root();
            }

            #[test]
            fn panics_when_relative() {
                let path = Path::new("foo/bar/baz.rs").to_owned();
                assert_that_panic_by(|| assert_that(path).with_location(false).has_a_root())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: "foo/bar/baz.rs"

                        to be a root-path, but it is not!
                        -------- assertr --------
                    "#});
            }
        }

        mod is_relative {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_relative() {
                let path = Path::new("foo/bar/baz.rs").to_owned();
                assert_that(path).is_relative();
            }

            #[test]
            fn panics_when_absolute() {
                let path = Path::new("/foo/bar/baz.rs").to_owned();
                assert_that_panic_by(|| assert_that(path).with_location(false).is_relative())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: "/foo/bar/baz.rs"

                        to be a relative path, but it is not!
                        -------- assertr --------
                    "#});
            }
        }

        mod has_filename {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!()).to_owned();
                assert_that(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that(path)
                        .with_location(false)
                        .has_file_name("some.json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"
    
                        Expected filename: "some.json"
                          Actual filename: "path.rs"
                        -------- assertr --------
                    "#});
            }
        }

        mod has_file_stem {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!()).to_owned();
                assert_that(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that(path).with_location(false).has_file_stem("some")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"
    
                        Expected filestem: "some"
                          Actual filestem: "path"
                        -------- assertr --------
                    "#});
            }
        }

        mod has_extension {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!()).to_owned();
                assert_that(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that(path).with_location(false).has_extension("json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"
    
                        Expected extension: "json"
                          Actual extension: "rs"
                        -------- assertr --------
                    "#});
            }
        }

        mod starts_with {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_prefix() {
                let path = Path::new(file!()).to_owned();
                assert_that(path).starts_with("assertr/src");
            }

            #[test]
            fn panics_when_not_a_prefix() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that(path).with_location(false).starts_with("foobar")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"

                        Did not start with expected prefix: "foobar"

                        Details: [
                            Only whole path components are matched!,
                        ]
                        -------- assertr --------
                    "#});
            }

            #[test]
            fn panics_when_not_a_whole_segment_prefix() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that(path).with_location(false).starts_with("assert")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"

                        Did not start with expected prefix: "assert"

                        Details: [
                            Only whole path components are matched!,
                        ]
                        -------- assertr --------
                    "#});
            }
        }

        mod ends_with {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            fn succeeds_when_postfix() {
                let path = Path::new(file!()).to_owned();
                assert_that(path).ends_with("std/path.rs");
            }

            #[test]
            fn panics_when_not_a_postfix() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| assert_that(path).with_location(false).ends_with("foobar"))
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"

                        Did not end with expected postfix: "foobar"

                        Details: [
                            Only whole path components are matched!,
                        ]
                        -------- assertr --------
                    "#});
            }

            #[test]
            fn panics_when_not_a_whole_segment_postfix() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| assert_that(path).with_location(false).ends_with("ath.rs"))
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "assertr/src/assertions/std/path.rs"

                        Did not end with expected postfix: "ath.rs"

                        Details: [
                            Only whole path components are matched!,
                        ]
                        -------- assertr --------
                    "#});
            }
        }
    }
}
