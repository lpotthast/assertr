use crate::{failure::GenericFailure, tracking::AssertionTracking, AssertThat, Mode};
use std::{ffi::OsStr, path::Path};

// TODO: PathBuf

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

impl<'t, M: Mode> PathAssertions for AssertThat<'t, &Path, M> {
    #[track_caller]
    fn exists(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.exists() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {actual:#?}\n\nto exist, but it does not!"),
            });
        }
        self
    }

    #[track_caller]
    fn does_not_exist(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if actual.exists() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {actual:#?}\n\nto not exist, but it does!"),
            });
        }
        self
    }

    #[track_caller]
    fn is_a_file(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_file() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {actual:#?}\n\nto be a file, but it is not!"),
            });
        }
        self
    }

    #[track_caller]
    fn is_a_directory(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_dir() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Expected: {actual:#?}\n\nto be a directory, but it is not!"
                ),
            });
        }
        self
    }

    #[track_caller]
    fn is_a_symlink(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_symlink() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {actual:#?}\n\nto be a symlink, but it is not!"),
            });
        }
        self
    }

    #[track_caller]
    fn has_a_root(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.has_root() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Expected: {actual:#?}\n\nto be a root-path, but it is not!"
                ),
            });
        }
        self
    }

    #[track_caller]
    fn is_relative(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_relative() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Expected: {actual:#?}\n\nto be a relative path, but it is not!"
                ),
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
        if actual_file_name == Some(OsStr::new(expected_file_name)) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Expected: {actual:#?}\n\nwith actual file_name: {actual_file_name:#?}\n\nto be equal to\n\nExpected: {expected_file_name:#?}"
                ),
            });
        }
        self
    }

    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let actual_file_stem = actual.file_stem();
        let expected_file_stem = expected.as_ref();
        if actual_file_stem == Some(OsStr::new(expected_file_stem)) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Expected: {actual:#?}\n\nwith actual file_stem: {actual_file_stem:#?}\n\nto be equal to\n\nExpected: {expected_file_stem:#?}"
                ),
            });
        }
        self
    }

    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let actual_extension = actual.extension();
        let expected_extension = expected.as_ref();
        if actual_extension == Some(OsStr::new(expected_extension)) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Expected: {actual:#?}\n\nwith actual extension: {actual_extension:#?}\n\nto be equal to\n\nExpected: {expected_extension:#?}"
                ),
            });
        }
        self
    }
}

// TODO: Test panics

#[cfg(test)]
mod tests {

    mod exists {
        use crate::prelude::*;
        use std::path::Path;

        #[test]
        fn succeeds_when_present() {
            let path = Path::new(file!());
            assert_that(path)
                .exists()
                .is_equal_to(Path::new("src/assertions/std/path.rs"))
                .map(|it| it.borrowed().to_str().unwrap_or_default().into())
                .is_equal_to("src/assertions/std/path.rs");
        }
    }

    mod does_not_exist {
        use crate::prelude::*;
        use std::path::Path;

        #[test]
        fn succeeds_when_not_present() {
            let path = Path::new("src/foo/bar/baz.rs");
            assert_that(path).does_not_exist();
        }
    }

    mod is_file {
        use crate::prelude::*;
        use std::path::Path;

        #[test]
        fn succeeds_when_file() {
            let path = Path::new(file!());
            assert_that(path).is_a_file();
        }
    }

    mod is_directory {
        use crate::prelude::*;
        use std::path::Path;

        #[test]
        fn succeeds_when_directory() {
            let path = Path::new(file!()).parent().expect("present");
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
        use crate::prelude::*;
        use std::path::Path;

        #[test]
        fn succeeds_when_root() {
            let path = Path::new("/foo/bar/baz.rs");
            assert_that(path).has_a_root();
        }
    }

    mod is_relative {
        use crate::prelude::*;
        use std::path::Path;

        #[test]
        fn succeeds_when_relative() {
            let path = Path::new("foo/bar/baz.rs");
            assert_that(path).is_relative();
        }
    }
}
