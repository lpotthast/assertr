use crate::{AssertThat, Mode, ValueRenderer, failure::FailureKind};
use core::fmt::{self, Debug, Display, Formatter};
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
                self.failure(FailureKind::Other)
                    .actual(self.render().value(self.actual()))
                    .relation("does not exist")
                    .raise();
            }
            Err(err) => {
                self.failure(FailureKind::Other)
                    .actual(self.render().value(self.actual()))
                    .relation("does not exist")
                    .fact("I/O error", format_args!("{err}"))
                    .raise();
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
        if matches!(actual.try_exists(), Ok(true)) {
            self.failure(FailureKind::Other)
                .actual(self.render().value(self.actual()))
                .relation("unexpectedly exists")
                .raise();
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
            self.failure(FailureKind::Variant)
                .actual(self.render().value(self.actual()))
                .relation("is not a file")
                .note(entry_kind(actual))
                .raise();
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
            self.failure(FailureKind::Variant)
                .actual(self.render().value(self.actual()))
                .relation("is not a directory")
                .note(entry_kind(actual))
                .raise();
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
            self.failure(FailureKind::Variant)
                .actual(self.render().value(self.actual()))
                .relation("is not a symlink")
                .note(entry_kind(actual))
                .raise();
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
            self.failure(FailureKind::Other)
                .actual(self.render().value(self.actual()))
                .relation("does not have a root")
                .raise();
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
            self.failure(FailureKind::Other)
                .actual(self.render().value(self.actual()))
                .relation("is not relative")
                .raise();
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
        let expected = expected.as_ref();
        if actual.file_name() != Some(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("does not have the file name")
                .expected(format_args!("{}", DebugValue(expected)))
                .fact(
                    "Actual file name",
                    format_args!("{}", Component(actual.file_name())),
                )
                .raise();
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
        let expected = expected.as_ref();
        if actual.file_stem() != Some(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("does not have the file stem")
                .expected(format_args!("{}", DebugValue(expected)))
                .fact(
                    "Actual file stem",
                    format_args!("{}", Component(actual.file_stem())),
                )
                .raise();
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
        let expected = expected.as_ref();
        if actual.extension() != Some(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("does not have the extension")
                .expected(format_args!("{}", DebugValue(expected)))
                .fact(
                    "Actual extension",
                    format_args!("{}", Component(actual.extension())),
                )
                .raise();
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
        let expected = expected.as_ref();
        if !actual.starts_with(expected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("does not start with")
                .expected(format_args!("{}", DebugValue(expected)))
                .note("Only whole path components are matched.")
                .raise();
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
        let expected = expected.as_ref();
        if !actual.ends_with(expected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("does not end with")
                .expected(format_args!("{}", DebugValue(expected)))
                .note("Only whole path components are matched.")
                .raise();
        }
        self
    }
}

/// What the file system holds at `path`, as the note of a failed kind check.
fn entry_kind(path: &Path) -> &'static str {
    if path.is_dir() {
        "The path is a directory."
    } else if path.is_file() {
        "The path is a file."
    } else if path.exists() {
        "The path exists."
    } else {
        "The path does not exist."
    }
}

/// Adapts a path-related value's quoted `Debug` form to the builder's verbatim text input.
struct DebugValue<'a, T: ?Sized>(&'a T);

impl<T: Debug + ?Sized> Display for DebugValue<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.0, f)
    }
}

/// An optional path component as a fact value: quoted and escaped like the expected component it
/// is compared with, or `<none>` when the path has no such component.
struct Component<'a>(Option<&'a OsStr>);

impl Display for Component<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(component) => Debug::fmt(component, f),
            None => f.write_str("<none>"),
        }
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use std::path::PathBuf;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, PathBuf, Panic, NoRenderer>
                    => PathAssertions<Subject = PathBuf, Renderer = NoRenderer>
            );
        }

        #[test]
        fn failures_use_the_active_renderer() {
            let failures = assert_that!(PathBuf::from(
                "assertr-renderer-test-path-that-does-not-exist",
            ))
            .with_renderer(SentinelRenderer)
            .with_location(false)
            .capture(PathAssertions::exists);

            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
        }
    }

    macro_rules! source_relative_path {
        () => {{
            let source = std::path::Path::new(file!());
            source
                .strip_prefix(env!("CARGO_PKG_NAME"))
                .unwrap_or(source)
        }};
    }

    macro_rules! source_path {
        () => {{
            let source = std::path::Path::new(file!());
            let package_relative = source
                .strip_prefix(env!("CARGO_PKG_NAME"))
                .unwrap_or(source);
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(package_relative)
        }};
    }

    mod path {
        mod exists {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = source_path!();
                path.as_path().must().exist();
            }

            #[test]
            fn succeeds_when_present() {
                let path = source_path!();
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
                        Expression: `path`

                        Actual: "src/assertions/std/some-non-existing-file.rs"

                        does not exist
                        -------- assertr --------
                    "#});
            }
        }

        mod does_not_exist {
            use crate::prelude::*;
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
                let path = source_path!();
                assert_that_panic_by(|| {
                    assert_that!(path.as_path())
                        .with_location(false)
                        .does_not_exist();
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Actual: \"")
                .contains("src/assertions/std/path.rs\"")
                .contains("unexpectedly exists");
            }
        }

        mod is_a_file {
            use crate::prelude::*;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = source_path!();
                path.as_path().must().be_a_file();
            }

            #[test]
            fn succeeds_when_file() {
                let path = source_path!();
                assert_that!(path.as_path()).is_a_file();
            }

            #[test]
            fn panics_when_not_a_file() {
                let path = source_path!();
                let dir = path.parent().unwrap();
                assert_that_panic_by(|| {
                    assert_that!(dir)
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_file()
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Actual: \"")
                .contains("src/assertions/std\"")
                .contains("is not a file")
                .contains("Details:\n  - The path is a directory.");
            }
        }

        mod is_a_directory {
            use crate::prelude::*;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = source_path!();
                let path = path.parent().expect("present");
                path.must().be_a_directory();
            }

            #[test]
            fn succeeds_when_directory() {
                let path = source_path!();
                let path = path.parent().expect("present");
                assert_that!(path).is_a_directory();
            }

            #[test]
            fn panics_when_not_a_directory() {
                let path = source_path!();
                assert_that_panic_by(|| {
                    assert_that!(path.as_path())
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_directory();
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Actual: \"")
                .contains("src/assertions/std/path.rs\"")
                .contains("is not a directory")
                .contains("Details:\n  - The path is a file.");
            }
        }

        #[cfg(unix)]
        mod is_a_symlink {
            use std::path::PathBuf;

            use crate::prelude::*;

            /// Creates a symlink to this source file in the temp dir and removes it on drop.
            struct TempSymlink(PathBuf);

            impl TempSymlink {
                fn new(name: &str) -> Self {
                    let link =
                        std::env::temp_dir().join(format!("assertr-{name}-{}", std::process::id()));
                    let _ = std::fs::remove_file(&link);
                    std::os::unix::fs::symlink(source_path!(), &link).expect("symlink created");
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
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).is_a_symlink();
                })
                .has_type::<String>()
                .is_equal_to(indoc::formatdoc! {r#"
                    -------- assertr --------
                    Expression: `path`

                    Actual: "{}"

                    is not a symlink

                    Details:
                      - The path is a file.
                    -------- assertr --------
                "#, source_relative_path!().display()});
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
                        Expression: `path`

                        Actual: "foo/bar/baz.rs"

                        does not have a root
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
                        Expression: `path`

                        Actual: "/foo/bar/baz.rs"

                        is not relative
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
                source_relative_path!().must().have_file_name("path.rs");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = source_relative_path!();
                assert_that!(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .has_file_name("some.json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not have the file name

                        Expected: "some.json"

                        Details:
                          - Actual file name: "path.rs"
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
                    Expression: `path`

                    Actual: "/"

                    does not have the file name

                    Expected: "foo"

                    Details:
                      - Actual file name: <none>
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
                source_relative_path!().must().have_file_stem("path");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = source_relative_path!();
                assert_that!(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .has_file_stem("some")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not have the file stem

                        Expected: "some"

                        Details:
                          - Actual file stem: "path"
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
                    Expression: `path`

                    Actual: "/"

                    does not have the file stem

                    Expected: "foo"

                    Details:
                      - Actual file stem: <none>
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
                source_relative_path!().must().have_extension("rs");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = source_relative_path!();
                assert_that!(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .has_extension("json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not have the extension

                        Expected: "json"

                        Details:
                          - Actual extension: "rs"
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
                    Expression: `path`

                    Actual: "/"

                    does not have the extension

                    Expected: "rs"

                    Details:
                      - Actual extension: <none>
                    -------- assertr --------
                "#});
            }
        }

        mod starts_with {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                source_relative_path!().must().start_with("src");
            }

            #[test]
            fn succeeds_when_prefix() {
                let path = source_relative_path!();
                assert_that!(path).starts_with("src");
            }

            #[test]
            fn panics_when_not_a_prefix() {
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .starts_with("foobar")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not start with

                        Expected: "foobar"

                        Details:
                          - Only whole path components are matched.
                        -------- assertr --------
                    "#});
            }

            #[test]
            fn panics_when_not_a_whole_segment_prefix() {
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .starts_with("assert")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not start with

                        Expected: "assert"

                        Details:
                          - Only whole path components are matched.
                        -------- assertr --------
                    "#});
            }
        }

        mod ends_with {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                source_relative_path!().must().end_with("std/path.rs");
            }

            #[test]
            fn succeeds_when_postfix() {
                let path = source_relative_path!();
                assert_that!(path).ends_with("std/path.rs");
            }

            #[test]
            fn panics_when_not_a_postfix() {
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).ends_with("foobar")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not end with

                        Expected: "foobar"

                        Details:
                          - Only whole path components are matched.
                        -------- assertr --------
                    "#});
            }

            #[test]
            fn panics_when_not_a_whole_segment_postfix() {
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).ends_with("ath.rs")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not end with

                        Expected: "ath.rs"

                        Details:
                          - Only whole path components are matched.
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
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = source_path!();
                path.must().exist();
            }

            #[test]
            fn succeeds_when_present() {
                let path = source_path!();
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
                        Expression: `path`

                        Actual: "src/assertions/std/some-non-existing-file.rs"

                        does not exist
                        -------- assertr --------
                    "#});
            }
        }

        mod does_not_exist {
            use crate::prelude::*;
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
                let path = source_path!();
                assert_that_panic_by(|| assert_that!(path).with_location(false).does_not_exist())
                    .has_type::<String>()
                    .contains("-------- assertr --------")
                    .contains("Actual: \"")
                    .contains("src/assertions/std/path.rs\"")
                    .contains("unexpectedly exists");
            }
        }

        mod is_a_file {
            use crate::prelude::*;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = source_path!();
                path.must().be_a_file();
            }

            #[test]
            fn succeeds_when_file() {
                let path = source_path!();
                assert_that!(path).is_a_file();
            }

            #[test]
            fn panics_when_not_a_file() {
                let path = source_path!();
                let dir = path.parent().unwrap().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(dir)
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_file();
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Actual: \"")
                .contains("src/assertions/std\"")
                .contains("is not a file")
                .contains("Details:\n  - The path is a directory.");
            }
        }

        mod is_a_directory {
            use crate::prelude::*;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = source_path!();
                let path = path.parent().expect("present");
                path.must().be_a_directory();
            }

            #[test]
            fn succeeds_when_directory() {
                let path = source_path!();
                let path = path.parent().expect("present");
                assert_that!(path).is_a_directory();
            }

            #[test]
            fn panics_when_not_a_directory() {
                let path = source_path!();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .exists() // Sanity-check. Non-existing paths would also not be files!
                        .is_a_directory()
                })
                .has_type::<String>()
                .contains("-------- assertr --------")
                .contains("Actual: \"")
                .contains("src/assertions/std/path.rs\"")
                .contains("is not a directory")
                .contains("Details:\n  - The path is a file.");
            }
        }

        #[cfg(unix)]
        mod is_a_symlink {
            use std::path::PathBuf;

            use crate::prelude::*;

            /// Creates a symlink to this source file in the temp dir and removes it on drop.
            struct TempSymlink(PathBuf);

            impl TempSymlink {
                fn new(name: &str) -> Self {
                    let link =
                        std::env::temp_dir().join(format!("assertr-{name}-{}", std::process::id()));
                    let _ = std::fs::remove_file(&link);
                    std::os::unix::fs::symlink(source_path!(), &link).expect("symlink created");
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
                let path = source_relative_path!();
                assert_that_panic_by(|| {
                    assert_that!(path.to_path_buf())
                        .with_location(false)
                        .is_a_symlink();
                })
                .has_type::<String>()
                .is_equal_to(indoc::formatdoc! {r#"
                    -------- assertr --------
                    Expression: `path.to_path_buf()`

                    Actual: "{}"

                    is not a symlink

                    Details:
                      - The path is a file.
                    -------- assertr --------
                "#, source_relative_path!().display()});
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
                        Expression: `path`

                        Actual: "foo/bar/baz.rs"

                        does not have a root
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
                        Expression: `path`

                        Actual: "/foo/bar/baz.rs"

                        is not relative
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
                let path = source_relative_path!().to_owned();
                path.must().have_file_name("path.rs");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = source_relative_path!().to_owned();
                assert_that!(path).has_file_name("path.rs");
            }

            #[test]
            fn panics_when_different() {
                let path = source_relative_path!().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .has_file_name("some.json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not have the file name

                        Expected: "some.json"

                        Details:
                          - Actual file name: "path.rs"
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
                    Expression: `path`

                    Actual: "/"

                    does not have the file name

                    Expected: "foo"

                    Details:
                      - Actual file name: <none>
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
                let path = source_relative_path!().to_owned();
                path.must().have_file_stem("path");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = source_relative_path!().to_owned();
                assert_that!(path).has_file_stem("path");
            }

            #[test]
            fn panics_when_different() {
                let path = source_relative_path!().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .has_file_stem("some")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not have the file stem

                        Expected: "some"

                        Details:
                          - Actual file stem: "path"
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
                    Expression: `path`

                    Actual: "/"

                    does not have the file stem

                    Expected: "foo"

                    Details:
                      - Actual file stem: <none>
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
                let path = source_relative_path!().to_owned();
                path.must().have_extension("rs");
            }

            #[test]
            fn succeeds_when_equal() {
                let path = source_relative_path!().to_owned();
                assert_that!(path).has_extension("rs");
            }

            #[test]
            fn panics_when_different() {
                let path = source_relative_path!().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .has_extension("json")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not have the extension

                        Expected: "json"

                        Details:
                          - Actual extension: "rs"
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
                    Expression: `path`

                    Actual: "/"

                    does not have the extension

                    Expected: "rs"

                    Details:
                      - Actual extension: <none>
                    -------- assertr --------
                "#});
            }
        }

        mod starts_with {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = source_relative_path!().to_owned();
                path.must().start_with("src");
            }

            #[test]
            fn succeeds_when_prefix() {
                let path = source_relative_path!().to_owned();
                assert_that!(path).starts_with("src");
            }

            #[test]
            fn panics_when_not_a_prefix() {
                let path = source_relative_path!().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .starts_with("foobar")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not start with

                        Expected: "foobar"

                        Details:
                          - Only whole path components are matched.
                        -------- assertr --------
                    "#});
            }

            #[test]
            fn panics_when_not_a_whole_segment_prefix() {
                let path = source_relative_path!().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path)
                        .with_location(false)
                        .starts_with("assert")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not start with

                        Expected: "assert"

                        Details:
                          - Only whole path components are matched.
                        -------- assertr --------
                    "#});
            }
        }

        mod ends_with {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let path = source_relative_path!().to_owned();
                path.must().end_with("std/path.rs");
            }

            #[test]
            fn succeeds_when_postfix() {
                let path = source_relative_path!().to_owned();
                assert_that!(path).ends_with("std/path.rs");
            }

            #[test]
            fn panics_when_not_a_postfix() {
                let path = source_relative_path!().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).ends_with("foobar")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not end with

                        Expected: "foobar"

                        Details:
                          - Only whole path components are matched.
                        -------- assertr --------
                    "#});
            }

            #[test]
            fn panics_when_not_a_whole_segment_postfix() {
                let path = source_relative_path!().to_owned();
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).ends_with("ath.rs")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "src/assertions/std/path.rs"

                        does not end with

                        Expected: "ath.rs"

                        Details:
                          - Only whole path components are matched.
                        -------- assertr --------
                    "#});
            }
        }
    }
}
