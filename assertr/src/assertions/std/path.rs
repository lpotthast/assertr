use crate::{tracking::AssertionTracking, AssertThat, Mode};
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
}

impl<'t, M: Mode> PathAssertions for AssertThat<'t, PathBuf, M> {
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

impl<'t, M: Mode> PathAssertions for AssertThat<'t, &Path, M> {
    #[track_caller]
    fn exists(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.exists() {
            self.fail(format_args!(
                "Expected: {actual:#?}\n\nto exist, but it does not!\n"
            ));
        }
        self
    }

    #[track_caller]
    fn does_not_exist(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if actual.exists() {
            self.fail(format_args!(
                "Expected: {actual:#?}\n\nto not exist, but it does!\n"
            ));
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

// TODO: Test panics

#[cfg(test)]
mod tests {
    mod path {
        mod exists {
            use std::env;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path.as_path())
                    .exists()
                    .map(|it| it.borrowed().to_str().unwrap_or_default().into())
                    .ends_with("src/assertions/std/path.rs");
            }
        }

        mod does_not_exist {
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_not_present() {
                let path = Path::new("../../foo/bar/baz.rs");
                assert_that(path).does_not_exist();
            }
        }

        mod is_file {
            use std::env;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path.as_path()).is_a_file();
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
        }

        mod is_symlink {
            /*
            #[test]
            fn is_symlink_succeeds_when_directory() {
                let path = Path::new(file!());
                assert_that(path).is_symlink();
            }
            */
        }

        mod has_a_root {
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_root() {
                let path = Path::new("/foo/bar/baz.rs");
                assert_that(path).has_a_root();
            }
        }

        mod is_relative {
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_relative() {
                let path = Path::new("foo/bar/baz.rs");
                assert_that(path).is_relative();
            }
        }

        mod has_filename {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;

            #[test]
            fn succeeds_when_equal() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path.as_path()).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let relative_path = path.strip_prefix(env::current_dir().unwrap()).unwrap();
                assert_that_panic_by(|| {
                    assert_that(relative_path)
                        .with_location(false)
                        .has_file_name("some.json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "src/assertions/std/path.rs"
    
                        Expected filename: "some.json"
                          Actual filename: "path.rs"
                        -------- assertr --------
                    "#});
            }
        }

        mod has_file_stem {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;

            #[test]
            fn succeeds_when_equal() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path.as_path()).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let relative_path = path.strip_prefix(env::current_dir().unwrap()).unwrap();
                assert_that_panic_by(|| {
                    assert_that(relative_path)
                        .with_location(false)
                        .has_file_stem("some")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "src/assertions/std/path.rs"
    
                        Expected filestem: "some"
                          Actual filestem: "path"
                        -------- assertr --------
                    "#});
            }
        }

        mod has_extension {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;

            #[test]
            fn succeeds_when_equal() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path.as_path()).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let relative_path = path.strip_prefix(env::current_dir().unwrap()).unwrap();
                assert_that_panic_by(|| {
                    assert_that(relative_path)
                        .with_location(false)
                        .has_extension("json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "src/assertions/std/path.rs"
    
                        Expected extension: "json"
                          Actual extension: "rs"
                        -------- assertr --------
                    "#});
            }
        }
    }

    mod path_buf {
        mod exists {
            use std::env;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path)
                    .exists()
                    .map(|it| it.unwrap_owned().display().to_string().into())
                    .ends_with("src/assertions/std/path.rs");
            }
        }

        mod does_not_exist {
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_not_present() {
                let path = Path::new("../../foo/bar/baz.rs").to_owned();
                assert_that(path).does_not_exist();
            }
        }

        mod is_file {
            use std::env;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path).is_a_file();
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
        }

        mod is_symlink {
            /*
            #[test]
            fn is_symlink_succeeds_when_directory() {
                let path = Path::new(file!());
                assert_that(path).is_symlink();
            }
            */
        }

        mod has_a_root {
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_root() {
                let path = Path::new("/foo/bar/baz.rs").to_owned();
                assert_that(path).has_a_root();
            }
        }

        mod is_relative {
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_relative() {
                let path = Path::new("foo/bar/baz.rs").to_owned();
                assert_that(path).is_relative();
            }
        }

        mod has_filename {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;

            #[test]
            fn succeeds_when_equal() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let relative_path = path
                    .strip_prefix(env::current_dir().unwrap())
                    .unwrap()
                    .to_owned();
                assert_that_panic_by(|| {
                    assert_that(relative_path)
                        .with_location(false)
                        .has_file_name("some.json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "src/assertions/std/path.rs"
    
                        Expected filename: "some.json"
                          Actual filename: "path.rs"
                        -------- assertr --------
                    "#});
            }
        }

        mod has_file_stem {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;

            #[test]
            fn succeeds_when_equal() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let relative_path = path
                    .strip_prefix(env::current_dir().unwrap())
                    .unwrap()
                    .to_owned();
                assert_that_panic_by(|| {
                    assert_that(relative_path)
                        .with_location(false)
                        .has_file_stem("some")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "src/assertions/std/path.rs"
    
                        Expected filestem: "some"
                          Actual filestem: "path"
                        -------- assertr --------
                    "#});
            }
        }

        mod has_extension {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;

            #[test]
            fn succeeds_when_equal() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let relative_path = path
                    .strip_prefix(env::current_dir().unwrap())
                    .unwrap()
                    .to_owned();
                assert_that_panic_by(|| {
                    assert_that(relative_path)
                        .with_location(false)
                        .has_extension("json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Path: "src/assertions/std/path.rs"
    
                        Expected extension: "json"
                          Actual extension: "rs"
                        -------- assertr --------
                    "#});
            }
        }
    }
}
