use crate::{failure::GenericFailure, AssertThat};
use std::path::Path;

// TODO: PathBuf

impl<'t> AssertThat<'t, &Path> {
    #[track_caller]
    pub fn exists(self) -> Self {
        let actual = self.actual.borrowed();
        if !actual.exists() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {actual:#?}\n\nto exist, but it does not!"),
            });
        }
        self
    }

    #[track_caller]
    pub fn does_not_exist(self) -> Self {
        let actual = self.actual.borrowed();
        if actual.exists() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {actual:#?}\n\nto not exist, but it does!"),
            });
        }
        self
    }

    #[track_caller]
    pub fn is_a_file(self) -> Self {
        let actual = self.actual.borrowed();
        if !actual.is_file() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {actual:#?}\n\nto be a file, but it is not!"),
            });
        }
        self
    }

    #[track_caller]
    pub fn is_a_directory(self) -> Self {
        let actual = self.actual.borrowed();
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
    pub fn is_a_symlink(self) -> Self {
        let actual = self.actual.borrowed();
        if !actual.is_symlink() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {actual:#?}\n\nto be a symlink, but it is not!"),
            });
        }
        self
    }

    #[track_caller]
    pub fn has_a_root(self) -> Self {
        let actual = self.actual.borrowed();
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
    pub fn is_relative(self) -> Self {
        let actual = self.actual.borrowed();
        if !actual.is_relative() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Expected: {actual:#?}\n\nto be a relative path, but it is not!"
                ),
            });
        }
        self
    }

    // has file name
}

// TODO: Test panics

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::prelude::*;

    #[test]
    fn exists_succeeds_when_present() {
        let path = Path::new(file!());
        assert_that(path)
            .exists()
            .is_equal_to(Path::new("src/assertions/path.rs"))
            .map(|it| it.borrowed().to_str().unwrap_or_default().into())
            .is_equal_to("src/assertions/path.rs");
    }

    #[test]
    fn does_not_exist_succeeds_when_not_present() {
        let path = Path::new("src/foo/bar/baz.rs");
        assert_that(path).does_not_exist();
    }

    #[test]
    fn is_file_succeeds_when_file() {
        let path = Path::new(file!());
        assert_that(path).is_a_file();
    }

    #[test]
    fn is_directory_succeeds_when_directory() {
        let path = Path::new(file!()).parent().expect("present");
        assert_that(path).is_a_directory();
    }

    /*
    #[test]
    fn is_symlink_succeeds_when_directory() {
        let path = Path::new(file!());
        assert_that(path).is_symlink();
    }
    */

    #[test]
    fn has_a_root_succeeds_when_root() {
        let path = Path::new("/foo/bar/baz.rs");
        assert_that(path).has_a_root();
    }

    #[test]
    fn is_relative_succeeds_when_relative() {
        let path = Path::new("foo/bar/baz.rs");
        assert_that(path).is_relative();
    }
}
