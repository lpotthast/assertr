use crate::renderer::{IntoRendered, Rendered};
use crate::{AssertThat, Fact, Mode, ValueRenderer, failure::FailureKind};
use std::ops::Deref;
use std::{ffi::OsStr, path::Path};

/// Assertions for path values.
///
/// Blanket-implemented for path subjects that dereference to [`Path`], including [`Path`]
/// references and owned [`std::path::PathBuf`] values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
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
        Self::Renderer: ValueRenderer<Self::Subject> + ValueRenderer<std::io::Error>;

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
        Self::Renderer: ValueRenderer<Self::Subject> + ValueRenderer<OsStr>;

    /// Asserts that the final path component without its extension equals `expected`.
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject> + ValueRenderer<OsStr>;

    /// Asserts that the final path component's extension equals `expected`.
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject> + ValueRenderer<OsStr>;

    /// Asserts that the path starts with `expected` by whole path components.
    fn starts_with(self, expected: impl AsRef<Path>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject> + ValueRenderer<Path>;

    /// Asserts that the path ends with `expected` by whole path components.
    fn ends_with(self, expected: impl AsRef<Path>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject> + ValueRenderer<Path>;
}

impl<P: Deref<Target = Path>, M: Mode, R> PathAssertions for AssertThat<'_, P, M, R> {
    type Renderer = R;
    type Subject = P;

    #[track_caller]
    fn exists(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<std::io::Error>,
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
                    .fact(Fact::labelled("I/O error", self.render().value(&err)))
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
                .fact(Fact::note(entry_kind(actual)))
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
                .fact(Fact::note(entry_kind(actual)))
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
                .fact(Fact::note(entry_kind(actual)))
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
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected = expected.as_ref();
        if actual.file_name() != Some(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("does not have the file name")
                .expected(self.render().value(expected))
                .fact(Fact::labelled(
                    "Actual file name",
                    actual.file_name().map_or_else(
                        || Rendered::verbatim("<none>".into()),
                        |component| self.render().value(component).into_rendered(),
                    ),
                ))
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected = expected.as_ref();
        if actual.file_stem() != Some(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("does not have the file stem")
                .expected(self.render().value(expected))
                .fact(Fact::labelled(
                    "Actual file stem",
                    actual.file_stem().map_or_else(
                        || Rendered::verbatim("<none>".into()),
                        |component| self.render().value(component).into_rendered(),
                    ),
                ))
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected = expected.as_ref();
        if actual.extension() != Some(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("does not have the extension")
                .expected(self.render().value(expected))
                .fact(Fact::labelled(
                    "Actual extension",
                    actual.extension().map_or_else(
                        || Rendered::verbatim("<none>".into()),
                        |component| self.render().value(component).into_rendered(),
                    ),
                ))
                .raise();
        }
        self
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<Path>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected = expected.as_ref();
        if !actual.starts_with(expected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("does not start with")
                .expected(self.render().value(expected))
                .fact(Fact::note("Only whole path components are matched."))
                .raise();
        }
        self
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<Path>,
    {
        self.track_assertion();
        let actual = P::deref(self.actual());
        let expected = expected.as_ref();
        if !actual.ends_with(expected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("does not end with")
                .expected(self.render().value(expected))
                .fact(Fact::note("Only whole path components are matched."))
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
            fn renders_the_original_io_error() {
                use std::path::PathBuf;

                use indoc::formatdoc;

                use crate::test_support::{
                    CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
                };
                let subject = PathBuf::from("invalid\0private-path");
                let error = subject.try_exists().unwrap_err();
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(PathAssertions::exists);
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("invalid\0private-path")

                    does not exist

                    Details:
                      - I/O error: custom({error:?})
                    -------- assertr --------
                "#});

                assert_custom_value(&failures[0].facts[0].value, &error);
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(PathAssertions::exists);
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    does not exist

                    Details:
                      - I/O error: <redacted>
                    -------- assertr --------
                "});

                assert_redacted(&failures[0], &["private-path", "nul byte"]);
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
            #[cfg(unix)]
            fn renderer_receives_non_utf8_components() {
                use std::ffi::OsStr;
                use std::path::PathBuf;

                use indoc::formatdoc;

                use crate::test_support::{CustomValueRenderer, assert_custom_value};
                use std::os::unix::ffi::OsStrExt;
                let subject = PathBuf::from(OsStr::from_bytes(b"private-\xff.bin"));
                let operand = OsStr::from_bytes(b"other-\xfe.bin");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.has_file_name(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private-\xFF.bin")

                    does not have the file name

                    Expected: custom("other-\xFE.bin")

                    Details:
                      - Actual file name: custom("private-\xFF.bin")
                    -------- assertr --------
                "#});

                assert_custom_value(failures[0].expected.as_ref().unwrap(), operand);
                assert_custom_value(&failures[0].facts[0].value, subject.file_name().unwrap());
            }

            #[test]
            fn renders_typed_components_including_absence() {
                use std::ffi::OsStr;
                use std::path::PathBuf;

                use indoc::formatdoc;

                use crate::test_support::{
                    CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
                    rendered_text,
                };
                let subject = PathBuf::from("private-file.secret");
                let operand = OsStr::new("other-file");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.has_file_name(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private-file.secret")

                    does not have the file name

                    Expected: custom("other-file")

                    Details:
                      - Actual file name: custom("private-file.secret")
                    -------- assertr --------
                "#});

                assert_custom_value(failures[0].expected.as_ref().unwrap(), operand);
                assert_custom_value(&failures[0].facts[0].value, subject.file_name().unwrap());
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.has_file_name(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    does not have the file name

                    Expected: <redacted>

                    Details:
                      - Actual file name: <redacted>
                    -------- assertr --------
                "});

                assert_redacted(&failures[0], &["private-file", "secret", "other-file"]);
                let subject = PathBuf::from("/");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.has_file_name(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("/")

                    does not have the file name

                    Expected: custom("other-file")

                    Details:
                      - Actual file name: <none>
                    -------- assertr --------
                "#});

                assert_eq!(rendered_text(&failures[0].facts[0].value), "<none>");
                assert_eq!(failures[0].facts[0].value.type_name, None);
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
            fn renders_typed_components_including_absence() {
                use std::ffi::OsStr;
                use std::path::PathBuf;

                use indoc::formatdoc;

                use crate::test_support::{
                    CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
                    rendered_text,
                };
                let subject = PathBuf::from("private-file.secret");
                let operand = OsStr::new("other-file");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.has_file_stem(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private-file.secret")

                    does not have the file stem

                    Expected: custom("other-file")

                    Details:
                      - Actual file stem: custom("private-file")
                    -------- assertr --------
                "#});

                assert_custom_value(failures[0].expected.as_ref().unwrap(), operand);
                assert_custom_value(&failures[0].facts[0].value, subject.file_stem().unwrap());
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.has_file_stem(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    does not have the file stem

                    Expected: <redacted>

                    Details:
                      - Actual file stem: <redacted>
                    -------- assertr --------
                "});

                assert_redacted(&failures[0], &["private-file", "secret", "other-file"]);
                let subject = PathBuf::from("/");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.has_file_stem(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("/")

                    does not have the file stem

                    Expected: custom("other-file")

                    Details:
                      - Actual file stem: <none>
                    -------- assertr --------
                "#});

                assert_eq!(rendered_text(&failures[0].facts[0].value), "<none>");
                assert_eq!(failures[0].facts[0].value.type_name, None);
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
            fn renders_typed_components_including_absence() {
                use std::ffi::OsStr;
                use std::path::PathBuf;

                use indoc::formatdoc;

                use crate::test_support::{
                    CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
                    rendered_text,
                };
                let subject = PathBuf::from("private-file.secret");
                let operand = OsStr::new("other-file");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.has_extension(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private-file.secret")

                    does not have the extension

                    Expected: custom("other-file")

                    Details:
                      - Actual extension: custom("secret")
                    -------- assertr --------
                "#});

                assert_custom_value(failures[0].expected.as_ref().unwrap(), operand);
                assert_custom_value(&failures[0].facts[0].value, subject.extension().unwrap());
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.has_extension(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    does not have the extension

                    Expected: <redacted>

                    Details:
                      - Actual extension: <redacted>
                    -------- assertr --------
                "});

                assert_redacted(&failures[0], &["private-file", "secret", "other-file"]);
                let subject = PathBuf::from("/");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.has_extension(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("/")

                    does not have the extension

                    Expected: custom("other-file")

                    Details:
                      - Actual extension: <none>
                    -------- assertr --------
                "#});

                assert_eq!(rendered_text(&failures[0].facts[0].value), "<none>");
                assert_eq!(failures[0].facts[0].value.type_name, None);
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
            fn renders_original_path_operands() {
                use std::path::Path;
                use std::path::PathBuf;

                use indoc::formatdoc;

                use crate::test_support::{
                    CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
                };
                let subject = PathBuf::from("private/path");
                let operand = Path::new("other/path");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.starts_with(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private/path")

                    does not start with

                    Expected: custom("other/path")

                    Details:
                      - Only whole path components are matched.
                    -------- assertr --------
                "#});

                assert_custom_value(failures[0].expected.as_ref().unwrap(), operand);
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.starts_with(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    does not start with

                    Expected: <redacted>

                    Details:
                      - Only whole path components are matched.
                    -------- assertr --------
                "});

                assert_redacted(&failures[0], &["private", "other/path"]);
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
            fn renders_original_path_operands() {
                use std::path::Path;
                use std::path::PathBuf;

                use indoc::formatdoc;

                use crate::test_support::{
                    CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
                };
                let subject = PathBuf::from("private/path");
                let operand = Path::new("other/path");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.ends_with(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private/path")

                    does not end with

                    Expected: custom("other/path")

                    Details:
                      - Only whole path components are matched.
                    -------- assertr --------
                "#});

                assert_custom_value(failures[0].expected.as_ref().unwrap(), operand);
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.ends_with(operand));
                assert_that!(failures).has_length(1);
                assert_that!(failures[0]).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    does not end with

                    Expected: <redacted>

                    Details:
                      - Only whole path components are matched.
                    -------- assertr --------
                "});

                assert_redacted(&failures[0], &["private", "other/path"]);
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

            impl ValueRenderer<std::ffi::OsStr> for NonCloneRenderer {
                fn fmt(
                    &self,
                    value: &std::ffi::OsStr,
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
