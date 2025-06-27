use crate::{AssertThat, Mode, tracking::AssertionTracking};
use std::path::PathBuf;
use std::{ffi::OsStr, path::Path};

pub trait PathAssertions {
    fn exists(self) -> Self;
    fn exist(self) -> Self
    where
        Self: Sized,
    {
        self.exists()
    }

    fn does_not_exist(self) -> Self;
    fn not_exist(self) -> Self
    where
        Self: Sized,
    {
        self.does_not_exist()
    }

    fn is_a_file(self) -> Self;
    fn be_a_file(self) -> Self
    where
        Self: Sized,
    {
        self.is_a_file()
    }

    fn is_a_directory(self) -> Self;
    fn be_a_directory(self) -> Self
    where
        Self: Sized,
    {
        self.is_a_directory()
    }

    fn is_a_symlink(self) -> Self;
    fn be_a_symlink(self) -> Self
    where
        Self: Sized,
    {
        self.is_a_symlink()
    }

    fn has_a_root(self) -> Self;
    fn have_a_root(self) -> Self
    where
        Self: Sized,
    {
        self.has_a_root()
    }

    fn is_relative(self) -> Self;
    fn be_relative(self) -> Self
    where
        Self: Sized,
    {
        self.is_relative()
    }

    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self;
    fn have_file_name(self, expected: impl AsRef<OsStr>) -> Self
    where
        Self: Sized,
    {
        self.has_file_name(expected)
    }

    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self;
    fn have_file_stem(self, expected: impl AsRef<OsStr>) -> Self
    where
        Self: Sized,
    {
        self.has_file_stem(expected)
    }

    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self;
    fn have_extension(self, expected: impl AsRef<OsStr>) -> Self
    where
        Self: Sized,
    {
        self.has_extension(expected)
    }
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

    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self {
        self.derive(|it| it.as_path()).has_file_stem(expected);
        self
    }

    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self {
        self.derive(|it| it.as_path()).has_extension(expected);
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
                self.fail(format_args!(
                    "Expected: {actual:#?}\n\nto exist, but it does not!\n"
                ));
            }
            Err(err) => {
                self.fail(format_args!(
                    "Expected: {actual:#?}\n\nto exist, but it does not!\nstd::io::Error: {err:#?}\n"
                ));
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
                self.fail(format_args!(
                    "Expected: {actual:#?}\n\nto not exist, but it does!\n"
                ));
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
            self.fail(format_args!(
                "Expected: {actual:#?}\n\nto be a file, but it is not!\n"
            ));
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
            self.fail(format_args!(
                "Expected: {actual:#?}\n\nto be a directory, but it is not!\nThe path exists: {exists}\nThe path is a file: {is_file}\n"
            ));
        }
        self
    }

    #[track_caller]
    fn is_a_symlink(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_symlink() {
            self.fail(format_args!(
                "Expected: {actual:#?}\n\nto be a symlink, but it is not!\n"
            ));
        }
        self
    }

    #[track_caller]
    fn has_a_root(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.has_root() {
            self.fail(format_args!(
                "Expected: {actual:#?}\n\nto be a root-path, but it is not!\n"
            ));
        }
        self
    }

    #[track_caller]
    fn is_relative(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_relative() {
            self.fail(format_args!(
                "Expected: {actual:#?}\n\nto be a relative path, but it is not!\n"
            ));
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
                self.fail(format_args!(
                    "Path: {actual:?}\n\nExpected filename: {expected_file_name:#?}\n  Actual filename: {actual_file_name:#?}\n"
                ));
            }
        }
        self
    }

    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let actual_file_stem = actual.file_stem();
        let expected_file_stem = expected.as_ref();
        if let Some(actual_file_stem) = actual_file_stem {
            if actual_file_stem != expected_file_stem {
                self.fail(format_args!(
                    "Path: {actual:?}\n\nExpected filestem: {expected_file_stem:#?}\n  Actual filestem: {actual_file_stem:#?}\n"
                ));
            }
        }
        self
    }

    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let actual_extension = actual.extension();
        let expected_extension = expected.as_ref();
        if let Some(actual_extension) = actual_extension {
            if actual_extension != expected_extension {
                self.fail(format_args!(
                    "Path: {actual:?}\n\nExpected extension: {expected_extension:#?}\n  Actual extension: {actual_extension:#?}\n"
                ));
            }
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
                path.as_path()
                    .must()
                    .exist()
                    .map(|it| it.borrowed().to_str().unwrap_or_default().into())
                    .and()
                    .ends_with("src/assertions/std/path.rs");
            }

            #[test]
            fn panics_when_absent() {
                let path = Path::new("src/assertions/std/some-non-existing-file.rs");
                assert_that_panic_by(|| assert_that_owned(path).with_location(false).exists())
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
                assert_that_owned(path).does_not_exist();
            }

            #[test]
            fn panics_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    path.as_path().must().with_location(false).not_exist();
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
                assert_that_owned(path.as_path()).is_a_file();
            }

            #[test]
            fn panics_when_not_a_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let dir = path.parent().unwrap();
                assert_that_panic_by(|| {
                    assert_that_owned(dir)
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
                assert_that_owned(path.as_path()).is_a_directory();
            }

            #[test]
            fn panics_when_not_a_directory() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    path.as_path()
                        .must()
                        .with_location(false)
                        .exist() // Sanity-check. Non-existing paths would also not be files!
                        .be_a_directory();
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
                assert_that_owned(path).is_symlink();
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
                assert_that_owned(path).has_a_root();
            }

            #[test]
            fn panics_when_relative() {
                let path = Path::new("foo/bar/baz.rs");
                assert_that_panic_by(|| assert_that_owned(path).with_location(false).has_a_root())
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
                assert_that_owned(path).is_relative();
            }

            #[test]
            fn panics_when_absolute() {
                let path = Path::new("/foo/bar/baz.rs");
                assert_that_panic_by(|| assert_that_owned(path).with_location(false).is_relative())
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
                assert_that_owned(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that_owned(path)
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
                assert_that_owned(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that_owned(path)
                        .with_location(false)
                        .has_file_stem("some")
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
                assert_that_owned(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that_owned(path)
                        .with_location(false)
                        .has_extension("json")
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
                assert_that_owned(path)
                    .exists()
                    .map(|it| it.unwrap_owned().display().to_string().into())
                    .ends_with("src/assertions/std/path.rs");
            }

            #[test]
            fn panics_when_absent() {
                let path = Path::new("src/assertions/std/some-non-existing-file.rs").to_owned();
                assert_that_panic_by(|| assert_that_owned(path).with_location(false).exists())
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
                assert_that_owned(path).does_not_exist();
            }

            #[test]
            fn panics_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    assert_that_owned(path)
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
                assert_that_owned(path).is_a_file();
            }

            #[test]
            fn panics_when_not_a_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let dir = path.parent().unwrap().to_owned();
                assert_that_panic_by(|| {
                    assert_that_owned(dir)
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_file();
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
                assert_that_owned(path).is_a_directory();
            }

            #[test]
            fn panics_when_not_a_directory() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    assert_that_owned(path)
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
                assert_that_owned(path).has_a_root();
            }

            #[test]
            fn panics_when_relative() {
                let path = Path::new("foo/bar/baz.rs").to_owned();
                assert_that_panic_by(|| assert_that_owned(path).with_location(false).has_a_root())
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
                assert_that_owned(path).is_relative();
            }

            #[test]
            fn panics_when_absolute() {
                let path = Path::new("/foo/bar/baz.rs").to_owned();
                assert_that_panic_by(|| assert_that_owned(path).with_location(false).is_relative())
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
                assert_that_owned(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that_owned(path)
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
                assert_that_owned(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that_owned(path)
                        .with_location(false)
                        .has_file_stem("some")
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
                assert_that_owned(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that_owned(path)
                        .with_location(false)
                        .has_extension("json")
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
    }
}
