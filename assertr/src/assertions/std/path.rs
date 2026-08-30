use crate::{AssertThat, Mode, ValueRenderer};
use indoc::writedoc;
use std::fmt::Write;
use std::ops::Deref;
use std::{ffi::OsStr, path::Path};

/// Assertions for path values.
///
/// Blanket-implemented for path subjects that dereference to [`Path`], including [`Path`]
/// references and owned [`std::path::PathBuf`] values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PathAssertions {
    /// The path subject rendered in failure diagnostics.
    type Subject: Deref<Target = Path>;

    /// The renderer carried by the assertion chain.
    type Renderer;

    /// Asserts that the path exists.
    ///
    /// An I/O error while checking existence is reported as an assertion failure.
    fn exists(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the path does not exist.
    ///
    /// An I/O error while checking existence is treated as absence.
    fn does_not_exist(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the path exists and refers to a regular file.
    fn is_a_file(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the path exists and refers to a directory.
    fn is_a_directory(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the path itself is a symbolic link.
    fn is_a_symlink(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the path has a root component.
    fn has_a_root(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the path has no root component.
    fn is_relative(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the final path component equals `expected`.
    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the final path component without its extension equals `expected`.
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the final path component's extension equals `expected`.
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the path starts with `expected` by whole path components.
    fn starts_with(self, expected: impl AsRef<Path>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the path ends with `expected` by whole path components.
    fn ends_with(self, expected: impl AsRef<Path>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;
}

impl<P: Deref<Target = Path>, M: Mode, R> PathAssertions for AssertThat<'_, P, M, R> {
    type Renderer = R;
    type Subject = P;

    #[track_caller]
    fn exists(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        match actual.try_exists() {
            Ok(true) => {}
            Ok(false) => {
                let actual = self.render_value(self.actual());
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Expected: {actual:#?}

                        to exist, but it does not!
                    "}
                });
            }
            Err(err) => {
                let actual = self.render_value(self.actual());
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Expected: {actual:#?}

                        to exist, but it does not!

                        Encountered std::io::Error: {err:#?}
                    "}
                });
            }
        }
        self
    }

    #[track_caller]
    fn does_not_exist(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());

        match actual.try_exists() {
            Ok(true) => {
                let actual = self.render_value(self.actual());
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Expected: {actual:#?}

                        to not exist, but it does!
                    "}
                });
            }
            Ok(false) => {}
            Err(_err) => {}
        }
        self
    }

    #[track_caller]
    fn is_a_file(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        if !actual.is_file() {
            let exists = actual.exists();
            let is_dir = actual.is_dir();
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {actual:#?}

                    to be a file, but it is not!

                    The path exists: {exists}
                    The path is a directory: {is_dir}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_a_directory(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        if !actual.is_dir() {
            let exists = actual.exists();
            let is_file = actual.is_file();
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {actual:#?}

                    to be a directory, but it is not!

                    The path exists: {exists}
                    The path is a file: {is_file}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_a_symlink(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        if !actual.is_symlink() {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {actual:#?}

                    to be a symlink, but it is not!
                "}
            });
        }
        self
    }

    #[track_caller]
    fn has_a_root(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        if !actual.has_root() {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {actual:#?}

                    to be a root-path, but it is not!
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_relative(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        if !actual.is_relative() {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {actual:#?}

                    to be a relative path, but it is not!
                "}
            });
        }
        self
    }

    #[track_caller]
    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected_file_name = expected.as_ref();
        let Some(actual_file_name) = actual.file_name() else {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Path: {actual:?}

                    Expected filename: {expected_file_name:#?}
                      Actual filename: <none>
                "}
            });
            return self;
        };
        if actual_file_name != expected_file_name {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Path: {actual:?}

                    Expected filename: {expected_file_name:#?}
                      Actual filename: {actual_file_name:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected_file_stem = expected.as_ref();
        let Some(actual_file_stem) = actual.file_stem() else {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Path: {actual:?}

                    Expected filestem: {expected_file_stem:#?}
                      Actual filestem: <none>
                "}
            });
            return self;
        };
        if actual_file_stem != expected_file_stem {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Path: {actual:?}

                    Expected filestem: {expected_file_stem:#?}
                      Actual filestem: {actual_file_stem:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected_extension = expected.as_ref();
        let Some(actual_extension) = actual.extension() else {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Path: {actual:?}

                    Expected extension: {expected_extension:#?}
                      Actual extension: <none>
                "}
            });
            return self;
        };
        if actual_extension != expected_extension {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Path: {actual:?}

                    Expected extension: {expected_extension:#?}
                      Actual extension: {actual_extension:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected_prefix = expected.as_ref();
        if !actual.starts_with(expected_prefix) {
            let details = [String::from("Only whole path components are matched!")];
            let actual = self.render_value(self.actual());
            self.fail_with_details(details, |w: &mut String| {
                writedoc! {w, r"
                    Path: {actual:?}

                    Did not start with expected prefix: {expected_prefix:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected_postfix = expected.as_ref();
        if !actual.ends_with(expected_postfix) {
            let details = [String::from("Only whole path components are matched!")];
            let actual = self.render_value(self.actual());
            self.fail_with_details(details, |w: &mut String| {
                writedoc! {w, r"
                    Path: {actual:?}

                    Did not end with expected postfix: {expected_postfix:#?}
                "}
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
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                path.as_path().must().exist();
            }

            #[test]
            fn succeeds_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that!(path.as_path())
                    .exists()
                    .map(|it| it.borrowed().to_str().unwrap_or_default().into())
                    .ends_with("src/assertions/std/path.rs");
            }

            #[test]
            fn panics_when_absent() {
                let path = Path::new("src/assertions/std/some-non-existing-file.rs");
                assert_that_panic_by(|| assert_that!(path).with_location(false).exists())
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
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new("../../foo/bar/baz.rs").must().not_exist();
            }

            #[test]
            fn succeeds_when_absent() {
                let path = Path::new("../../foo/bar/baz.rs");
                assert_that!(path).does_not_exist();
            }

            #[test]
            fn panics_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    assert_that!(path.as_path())
                        .with_location(false)
                        .does_not_exist();
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Expected: \"")
                .contains("assertr/src/assertions/std/path.rs\"")
                .contains("to not exist, but it does!");
            }
        }

        mod is_a_file {
            use crate::prelude::*;
            use std::env;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                path.as_path().must().be_a_file();
            }

            #[test]
            fn succeeds_when_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that!(path.as_path()).is_a_file();
            }

            #[test]
            fn panics_when_not_a_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let dir = path.parent().unwrap();
                assert_that_panic_by(|| {
                    assert_that!(dir)
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

        mod is_a_directory {
            use std::env;
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = env::current_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(Path::new(file!()).parent().expect("present"));
                path.as_path().must().be_a_directory();
            }

            #[test]
            fn succeeds_when_directory() {
                let path = env::current_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(Path::new(file!()).parent().expect("present"));
                assert_that!(path.as_path()).is_a_directory();
            }

            #[test]
            fn panics_when_not_a_directory() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    assert_that!(path.as_path())
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_directory();
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

        #[cfg(unix)]
        mod is_a_symlink {
            use std::path::{Path, PathBuf};

            use crate::prelude::*;

            /// Creates a symlink to this source file in the temp dir and removes it on drop.
            struct TempSymlink(PathBuf);

            impl TempSymlink {
                fn new(name: &str) -> Self {
                    let link =
                        std::env::temp_dir().join(format!("assertr-{name}-{}", std::process::id()));
                    let _ = std::fs::remove_file(&link);
                    std::os::unix::fs::symlink(Path::new(file!()), &link).expect("symlink created");
                    Self(link)
                }
            }

            impl Drop for TempSymlink {
                fn drop(&mut self) {
                    let _ = std::fs::remove_file(&self.0);
                }
            }

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let link = TempSymlink::new("path-fluent");
                link.0.as_path().must().be_a_symlink();
            }

            #[test]
            fn succeeds_when_symlink() {
                let link = TempSymlink::new("path-succeeds");
                assert_that!(link.0.as_path()).is_a_symlink();
            }

            #[test]
            fn panics_when_not_a_symlink() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).is_a_symlink();
                })
                .has_type::<String>()
                .is_equal_to(indoc::formatdoc! {r#"
                    -------- assertr --------
                    Expected: "{}"

                    to be a symlink, but it is not!
                    -------- assertr --------
                "#, file!()});
            }
        }

        mod has_a_root {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new("/foo/bar/baz.rs").must().have_a_root();
            }

            #[test]
            fn succeeds_when_root() {
                let path = Path::new("/foo/bar/baz.rs");
                assert_that!(path).has_a_root();
            }

            #[test]
            fn panics_when_relative() {
                let path = Path::new("foo/bar/baz.rs");
                assert_that_panic_by(|| assert_that!(path).with_location(false).has_a_root())
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
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new("foo/bar/baz.rs").must().be_relative();
            }

            #[test]
            fn succeeds_when_relative() {
                let path = Path::new("foo/bar/baz.rs");
                assert_that!(path).is_relative();
            }

            #[test]
            fn panics_when_absolute() {
                let path = Path::new("/foo/bar/baz.rs");
                assert_that_panic_by(|| assert_that!(path).with_location(false).is_relative())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: "/foo/bar/baz.rs"

                        to be a relative path, but it is not!
                        -------- assertr --------
                    "#});
            }
        }

        mod has_file_name {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new(file!()).must().have_file_name("path.rs");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!());
                assert_that!(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that!(path)
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

            #[test]
            fn panics_when_path_has_no_file_name() {
                let path = Path::new("/");
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).has_file_name("foo")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Path: "/"

                    Expected filename: "foo"
                      Actual filename: <none>
                    -------- assertr --------
                "#});
            }
        }

        mod has_file_stem {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new(file!()).must().have_file_stem("path");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!());
                assert_that!(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that!(path)
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

            #[test]
            fn panics_when_path_has_no_file_stem() {
                let path = Path::new("/");
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).has_file_stem("foo")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Path: "/"

                    Expected filestem: "foo"
                      Actual filestem: <none>
                    -------- assertr --------
                "#});
            }
        }

        mod has_extension {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new(file!()).must().have_extension("rs");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!());
                assert_that!(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that!(path)
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

            #[test]
            fn panics_when_path_has_no_extension() {
                let path = Path::new("/");
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).has_extension("rs")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Path: "/"

                    Expected extension: "rs"
                      Actual extension: <none>
                    -------- assertr --------
                "#});
            }
        }

        mod starts_with {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new(file!()).must().start_with("assertr/src");
            }

            #[test]
            fn succeeds_when_prefix() {
                let path = Path::new(file!());
                assert_that!(path).starts_with("assertr/src");
            }

            #[test]
            fn panics_when_not_a_prefix() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .starts_with("foobar")
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
                    assert_that!(path)
                        .with_location(false)
                        .starts_with("assert")
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
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new(file!()).must().end_with("std/path.rs");
            }

            #[test]
            fn succeeds_when_postfix() {
                let path = Path::new(file!());
                assert_that!(path).ends_with("std/path.rs");
            }

            #[test]
            fn panics_when_not_a_postfix() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).ends_with("foobar")
                })
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
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).ends_with("ath.rs")
                })
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
        use crate::prelude::*;
        use std::path::PathBuf;

        #[test]
        fn failure_retains_the_subject_name_without_requiring_a_clone_renderer() {
            struct NonCloneRenderer;

            impl ValueRenderer<PathBuf> for NonCloneRenderer {
                fn fmt(
                    &self,
                    value: &PathBuf,
                    f: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result {
                    core::fmt::Debug::fmt(value, f)
                }
            }

            let failures = assert_that!(PathBuf::from("settings.json"))
                .with_subject_name("configuration path")
                .with_renderer(NonCloneRenderer)
                .capture(|it| it.has_extension("toml"));

            assert_that!(&failures).has_length(1);
            assert_that!(failures[0].subject_name.as_deref())
                .is_equal_to(Some("configuration path"));
        }

        mod exists {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::env;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                path.must().exist();
            }

            #[test]
            fn succeeds_when_present() {
                let path = env::current_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(file!())
                    .clone();
                assert_that_owned!(path)
                    .exists()
                    .map(|it| it.unwrap_owned().display().to_string().into())
                    .ends_with("src/assertions/std/path.rs");
            }

            #[test]
            fn panics_when_absent() {
                let path = Path::new("src/assertions/std/some-non-existing-file.rs").to_owned();
                assert_that_panic_by(|| assert_that!(path).with_location(false).exists())
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
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = Path::new("../../foo/bar/baz.rs").to_owned();
                path.must().not_exist();
            }

            #[test]
            fn succeeds_when_absent() {
                let path = Path::new("../../foo/bar/baz.rs").to_owned();
                assert_that!(path).does_not_exist();
            }

            #[test]
            fn panics_when_present() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| assert_that!(path).with_location(false).does_not_exist())
                    .has_type::<String>()
                    .contains("-------- assertr --------")
                    .contains("Expected: \"")
                    .contains("assertr/src/assertions/std/path.rs\"")
                    .contains("to not exist, but it does!");
            }
        }

        mod is_a_file {
            use crate::prelude::*;
            use std::env;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                path.must().be_a_file();
            }

            #[test]
            fn succeeds_when_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that!(path).is_a_file();
            }

            #[test]
            fn panics_when_not_a_file() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                let dir = path.parent().unwrap().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(dir)
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

        mod is_a_directory {
            use std::env;
            use std::path::Path;

            use crate::prelude::*;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = env::current_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(Path::new(file!()).parent().expect("present"));
                path.must().be_a_directory();
            }

            #[test]
            fn succeeds_when_directory() {
                let path = env::current_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(Path::new(file!()).parent().expect("present"));
                assert_that!(path).is_a_directory();
            }

            #[test]
            fn panics_when_not_a_directory() {
                let path = env::current_dir().unwrap().parent().unwrap().join(file!());
                assert_that_panic_by(|| {
                    assert_that!(path)
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

        #[cfg(unix)]
        mod is_a_symlink {
            use std::path::{Path, PathBuf};

            use crate::prelude::*;

            /// Creates a symlink to this source file in the temp dir and removes it on drop.
            struct TempSymlink(PathBuf);

            impl TempSymlink {
                fn new(name: &str) -> Self {
                    let link =
                        std::env::temp_dir().join(format!("assertr-{name}-{}", std::process::id()));
                    let _ = std::fs::remove_file(&link);
                    std::os::unix::fs::symlink(Path::new(file!()), &link).expect("symlink created");
                    Self(link)
                }
            }

            impl Drop for TempSymlink {
                fn drop(&mut self) {
                    let _ = std::fs::remove_file(&self.0);
                }
            }

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let link = TempSymlink::new("path-buf-fluent");
                link.0.clone().must().be_a_symlink();
            }

            #[test]
            fn succeeds_when_symlink() {
                let link = TempSymlink::new("path-buf-succeeds");
                assert_that!(link.0.clone()).is_a_symlink();
            }

            #[test]
            fn panics_when_not_a_symlink() {
                let path = Path::new(file!());
                assert_that_panic_by(|| {
                    assert_that!(path.to_path_buf())
                        .with_location(false)
                        .is_a_symlink();
                })
                .has_type::<String>()
                .is_equal_to(indoc::formatdoc! {r#"
                    -------- assertr --------
                    Expected: "{}"

                    to be a symlink, but it is not!
                    -------- assertr --------
                "#, file!()});
            }
        }

        mod has_a_root {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = Path::new("/foo/bar/baz.rs").to_owned();
                path.must().have_a_root();
            }

            #[test]
            fn succeeds_when_root() {
                let path = Path::new("/foo/bar/baz.rs").to_owned();
                assert_that!(path).has_a_root();
            }

            #[test]
            fn panics_when_relative() {
                let path = Path::new("foo/bar/baz.rs").to_owned();
                assert_that_panic_by(|| assert_that!(path).with_location(false).has_a_root())
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
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = Path::new("foo/bar/baz.rs").to_owned();
                path.must().be_relative();
            }

            #[test]
            fn succeeds_when_relative() {
                let path = Path::new("foo/bar/baz.rs").to_owned();
                assert_that!(path).is_relative();
            }

            #[test]
            fn panics_when_absolute() {
                let path = Path::new("/foo/bar/baz.rs").to_owned();
                assert_that_panic_by(|| assert_that!(path).with_location(false).is_relative())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: "/foo/bar/baz.rs"

                        to be a relative path, but it is not!
                        -------- assertr --------
                    "#});
            }
        }

        mod has_file_name {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = Path::new(file!()).to_owned();
                path.must().have_file_name("path.rs");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!()).to_owned();
                assert_that!(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
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

            #[test]
            fn panics_when_path_has_no_file_name() {
                let path = Path::new("/").to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).has_file_name("foo")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Path: "/"

                    Expected filename: "foo"
                      Actual filename: <none>
                    -------- assertr --------
                "#});
            }
        }

        mod has_file_stem {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = Path::new(file!()).to_owned();
                path.must().have_file_stem("path");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!()).to_owned();
                assert_that!(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
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

            #[test]
            fn panics_when_path_has_no_file_stem() {
                let path = Path::new("/").to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).has_file_stem("foo")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Path: "/"

                    Expected filestem: "foo"
                      Actual filestem: <none>
                    -------- assertr --------
                "#});
            }
        }

        mod has_extension {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = Path::new(file!()).to_owned();
                path.must().have_extension("rs");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = Path::new(file!()).to_owned();
                assert_that!(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
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

            #[test]
            fn panics_when_path_has_no_extension() {
                let path = Path::new("/").to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).has_extension("rs")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Path: "/"

                    Expected extension: "rs"
                      Actual extension: <none>
                    -------- assertr --------
                "#});
            }
        }

        mod starts_with {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = Path::new(file!()).to_owned();
                path.must().start_with("assertr/src");
            }

            #[test]
            fn succeeds_when_prefix() {
                let path = Path::new(file!()).to_owned();
                assert_that!(path).starts_with("assertr/src");
            }

            #[test]
            fn panics_when_not_a_prefix() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .starts_with("foobar")
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
                    assert_that!(path)
                        .with_location(false)
                        .starts_with("assert")
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
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = Path::new(file!()).to_owned();
                path.must().end_with("std/path.rs");
            }

            #[test]
            fn succeeds_when_postfix() {
                let path = Path::new(file!()).to_owned();
                assert_that!(path).ends_with("std/path.rs");
            }

            #[test]
            fn panics_when_not_a_postfix() {
                let path = Path::new(file!()).to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).ends_with("foobar")
                })
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
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).ends_with("ath.rs")
                })
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
