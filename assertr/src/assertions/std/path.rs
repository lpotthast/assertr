use std::{ffi::OsStr, fs::FileType, io, ops::Deref, path::Path};

use crate::{
    AssertThat, Mode,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{DebugRenderer, Rendered, ValueRenderer},
};

/// Whether an I/O error confirms that the inspected path is absent.
///
/// [`io::ErrorKind::NotADirectory`] means that an ancestor is not a directory.
fn confirms_absence(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
    )
}

/// Whether a [`Path::try_exists`] observation confirms the expected existence. Errors confirm only
/// absence, and only when they prove that the path does not exist.
fn existence_confirmed(observed: &io::Result<bool>, expected_existence: bool) -> bool {
    match observed {
        Ok(exists) => *exists == expected_existence,
        Err(error) => !expected_existence && confirms_absence(error),
    }
}

/// Explains a rejected [`Path::try_exists`] observation without inspecting the path again.
fn explain_existence<P, R>(
    actual: &P,
    observed: io::Result<bool>,
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder
where
    R: ValueRenderer<P> + ValueRenderer<io::Error>,
{
    let render = context.render();
    let failure = failure.actual(render.value(actual));
    match observed {
        Ok(true) => failure.relation("unexpectedly exists"),
        Err(error) if !confirms_absence(&error) => failure
            .relation("could not determine whether the path exists")
            .fact(Fact::labelled("I/O error", render.value(&error))),
        Ok(false) | Err(_) => failure.relation("does not exist"),
    }
}

/// Generates an existence expectation retaining the original [`Path::try_exists`] result.
macro_rules! existence_expectation {
    ($(#[$meta:meta])* $name:ident, exists: $exists:literal, relation: $relation:literal) => {
        $(#[$meta])*
        ///
        /// [`Path::try_exists`] follows symbolic links, so a dangling symlink counts as absent.
        /// Rejection retains the original result. Explanation classifies it without inspecting
        /// the path again.
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl<P: Deref<Target = Path>, R> Expectation<P, R> for $name
        where
            R: ValueRenderer<P> + ValueRenderer<io::Error>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                P: 'a;
            type Rejection<'a>
                = io::Result<bool>
            where
                Self: 'a,
                P: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a P,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), io::Result<bool>> {
                let observed = actual.deref().try_exists();
                if existence_confirmed(&observed, $exists) {
                    Ok(())
                } else {
                    Err(observed)
                }
            }

            const KIND: FailureKind = FailureKind::Other;
            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a P, io::Result<bool>)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                match rejected {
                    None => failure.relation($relation),
                    Some((actual, observed)) => explain_existence(actual, observed, failure, context),
                }
            }
        }
    };
}

existence_expectation!(
    /// Checks path existence, retaining an I/O error on rejection.
    ///
    /// A [`Path::try_exists`] result of `Ok(true)` passes. `Ok(false)` and an
    /// [`io::ErrorKind::NotFound`] or [`io::ErrorKind::NotADirectory`] error, which means that an
    /// ancestor is not a directory, confirm absence. Any other I/O error is retained as evidence
    /// that existence could not be determined.
    Exists,
    exists: true,
    relation: "exists"
);

existence_expectation!(
    /// Checks whether a path is absent, rejecting I/O errors that prevent determining existence.
    ///
    /// A [`Path::try_exists`] result of `Ok(false)` and an [`io::ErrorKind::NotFound`] or
    /// [`io::ErrorKind::NotADirectory`] error, which means that an ancestor is not a directory,
    /// confirm absence. Any other I/O error is retained as evidence that existence could not be
    /// determined.
    DoesNotExist,
    exists: false,
    relation: "does not exist"
);

property_expectation! {
    /// Checks whether a path has a root component.
    pub struct HasARoot for<P: Deref<Target = Path>> P;
    kind Other;
    check |actual| actual.has_root();
    relations "has a root", "does not have a root";
}

property_expectation! {
    /// Checks whether a path is relative.
    pub struct IsRelative for<P: Deref<Target = Path>> P;
    kind Other;
    check |actual| actual.is_relative();
    relations "is relative", "is not relative";
}

/// Explains a rejected entry-kind observation without inspecting the path again.
fn explain_entry_kind<P, R>(
    actual: &P,
    observed: io::Result<FileType>,
    (mismatch, inspection_failed): (&'static str, &'static str),
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder
where
    R: ValueRenderer<P> + ValueRenderer<io::Error>,
{
    let render = context.render();
    let failure = failure.actual(render.value(actual));
    let note = match observed {
        Ok(file_type) if file_type.is_dir() => "The path is a directory.",
        Ok(file_type) if file_type.is_file() => "The path is a file.",
        Ok(_) => "The path exists.",
        Err(error) if confirms_absence(&error) => "The path does not exist.",
        Err(error) => {
            return failure
                .relation(inspection_failed)
                .fact(Fact::labelled("I/O error", render.value(&error)));
        }
    };
    failure.relation(mismatch).fact(Fact::note(note))
}

/// Generates a path entry-kind expectation from its metadata source, predicate, and relations.
///
/// Rejection retains the original metadata result as the observed file type or I/O error.
macro_rules! entry_kind_expectation {
    (
        $(#[$meta:meta])*
        $name:ident,
        metadata: $metadata:ident,
        matches: $matches:ident,
        relations: $met:literal, $unmet:literal, $inspection_failed:literal $(,)?
    ) => {
        $(#[$meta])*
        ///
        /// Rejection retains the observed file type, or the I/O error that confirmed absence or
        /// prevented inspecting the path.
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl<P: Deref<Target = Path>, R> Expectation<P, R> for $name
        where
            R: ValueRenderer<P> + ValueRenderer<io::Error>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                P: 'a;
            type Rejection<'a>
                = io::Result<FileType>
            where
                Self: 'a,
                P: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a P,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), io::Result<FileType>> {
                match actual.deref().$metadata() {
                    Ok(metadata) if metadata.file_type().$matches() => Ok(()),
                    observed => Err(observed.map(|metadata| metadata.file_type())),
                }
            }

            const KIND: FailureKind = FailureKind::Variant;
            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a P, io::Result<FileType>)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                match rejected {
                    None => failure.relation($met),
                    Some((actual, observed)) => explain_entry_kind(
                        actual,
                        observed,
                        ($unmet, $inspection_failed),
                        failure,
                        context,
                    ),
                }
            }
        }
    };
}

entry_kind_expectation!(
    /// Checks whether the path is a regular file, following symbolic links.
    IsAFile,
    metadata: metadata,
    matches: is_file,
    relations: "is a file", "is not a file", "could not determine whether the path is a file",
);

entry_kind_expectation!(
    /// Checks whether the path is a directory, following symbolic links.
    IsADirectory,
    metadata: metadata,
    matches: is_dir,
    relations: "is a directory", "is not a directory",
        "could not determine whether the path is a directory",
);

entry_kind_expectation!(
    /// Checks whether the path itself is a symbolic link, without following it.
    IsASymlink,
    metadata: symlink_metadata,
    matches: is_symlink,
    relations: "is a symlink", "is not a symlink",
        "could not determine whether the path is a symlink",
);

/// Rejected path component together with the expected component.
type ComponentRejection<'a> = (Option<&'a OsStr>, &'a OsStr);

/// Generates a path-component equality expectation from its component accessor and labels.
macro_rules! component_expectation {
    ($(#[$meta:meta])* $name:ident, $accessor:ident, $component:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $name<E>(E);
        impl<P: Deref<Target = Path>, E, R> Expectation<P, R> for $name<E>
        where
            E: AsRef<OsStr>,
            R: ValueRenderer<P> + ValueRenderer<OsStr>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                P: 'a;
            type Rejection<'a>
                = ComponentRejection<'a>
            where
                Self: 'a,
                P: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a P,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), ComponentRejection<'a>> {
                let component = actual.deref().$accessor();
                let expected = self.0.as_ref();
                if component == Some(expected) {
                    Ok(())
                } else {
                    Err((component, expected))
                }
            }

            const KIND: FailureKind = FailureKind::Equality;
            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a P, ComponentRejection<'a>)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                let render = context.render();
                let Some((actual, (component, expected))) = rejected else {
                    return failure
                        .relation(concat!("has the ", $component))
                        .expected(render.value(self.0.as_ref()));
                };
                failure
                    .actual(render.value(actual))
                    .relation(concat!("does not have the ", $component))
                    .expected(render.value(expected))
                    .fact(Fact::labelled(
                        concat!("Actual ", $component),
                        component.map_or_else(|| Rendered::from("<none>"), |value| render.value(value)),
                    ))
            }
        }

        impl<E> $name<E> {
            /// Expects this path component.
            #[must_use]
            pub const fn new(expected: E) -> Self {
                Self(expected)
            }
        }
    };
}

component_expectation!(
    /// Compares the observed path file name with an expected component.
    HasFileName,
    file_name,
    "file name"
);

component_expectation!(
    /// Compares the observed path file stem with an expected component.
    HasFileStem,
    file_stem,
    "file stem"
);

component_expectation!(
    /// Compares the observed path extension with an expected component.
    HasExtension,
    extension,
    "extension"
);

/// Generates a whole-component path prefix or suffix expectation.
macro_rules! affix_expectation {
    ($(#[$meta:meta])* $name:ident, $check:ident, $met:literal, $unmet:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $name<E>(E);
        impl<P: Deref<Target = Path>, E, R> Expectation<P, R> for $name<E>
        where
            E: AsRef<Path>,
            R: ValueRenderer<P> + ValueRenderer<Path>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                P: 'a;
            type Rejection<'a>
                = &'a Path
            where
                Self: 'a,
                P: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a P,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), &'a Path> {
                let expected = self.0.as_ref();
                if actual.deref().$check(expected) {
                    Ok(())
                } else {
                    Err(expected)
                }
            }

            const KIND: FailureKind = FailureKind::Membership;
            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a P, &'a Path)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                let render = context.render();
                let expected = rejected.map_or_else(|| self.0.as_ref(), |(_, expected)| expected);
                failure
                    .relations(rejected.map(|(actual, _)| render.value(actual)), $met, $unmet)
                    .expected(render.value(expected))
                    .fact(Fact::note("Only whole path components are matched."))
            }
        }

        impl<E> $name<E> {
            /// Expects these whole path components.
            #[must_use]
            pub const fn new(expected: E) -> Self {
                Self(expected)
            }
        }
    };
}

affix_expectation!(
    /// Checks whether a path starts with the expected whole components.
    StartsWith,
    starts_with,
    "starts with",
    "does not start with"
);

affix_expectation!(
    /// Checks whether a path ends with the expected whole components.
    EndsWith,
    ends_with,
    "ends with",
    "does not end with"
);

/// Assertions for path values.
///
/// Blanket-implemented for path subjects that dereference to [`Path`], including [`Path`]
/// references and owned [`std::path::PathBuf`] values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PathAssertions<P: Deref<Target = Path>, R = DebugRenderer> {
    /// Asserts that the path exists.
    ///
    /// Passes only when [`Path::try_exists`] returns `Ok(true)`. A missing path, including one
    /// below an ancestor that is not a directory, fails as absent. Any other I/O error while
    /// checking existence is reported as an assertion failure, retaining the error as a fact.
    ///
    /// [`Path::try_exists`] follows symbolic links, so a dangling symlink counts as absent.
    fn exists(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>;

    /// Asserts that the path does not exist.
    ///
    /// Passes when [`Path::try_exists`] returns `Ok(false)` or an
    /// [`io::ErrorKind::NotADirectory`] error, which confirms that an ancestor is not a
    /// directory. An existing path or any other I/O error while checking existence is reported as
    /// an assertion failure, retaining the error as a fact.
    ///
    /// [`Path::try_exists`] follows symbolic links, so a dangling symlink counts as absent.
    fn does_not_exist(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>;

    /// Asserts that the path exists and refers to a regular file, following symbolic links.
    ///
    /// An I/O error other than a confirmed absence is reported as an assertion failure,
    /// retaining the error as a fact.
    fn is_a_file(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>;

    /// Asserts that the path exists and refers to a directory, following symbolic links.
    ///
    /// An I/O error other than a confirmed absence is reported as an assertion failure,
    /// retaining the error as a fact.
    fn is_a_directory(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>;

    /// Asserts that the path itself is a symbolic link.
    ///
    /// An I/O error other than a confirmed absence is reported as an assertion failure,
    /// retaining the error as a fact.
    fn is_a_symlink(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>;

    /// Asserts that the path has a root component.
    fn has_a_root(self) -> Self
    where
        R: ValueRenderer<P>;

    /// Asserts that the path has no root component.
    fn is_relative(self) -> Self
    where
        R: ValueRenderer<P>;

    /// Asserts that the final path component equals `expected`.
    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>;

    /// Asserts that the final path component without its extension equals `expected`.
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>;

    /// Asserts that the final path component's extension equals `expected`.
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>;

    /// Asserts that the path starts with `expected` by whole path components.
    fn starts_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<Path>;

    /// Asserts that the path ends with `expected` by whole path components.
    fn ends_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<Path>;
}

impl<P: Deref<Target = Path>, M: Mode, R> PathAssertions<P, R> for AssertThat<'_, P, M, R> {
    #[track_caller]
    fn exists(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.matches(Exists)
    }

    #[track_caller]
    fn does_not_exist(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.matches(DoesNotExist)
    }

    #[track_caller]
    fn is_a_file(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.matches(IsAFile)
    }

    #[track_caller]
    fn is_a_directory(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.matches(IsADirectory)
    }

    #[track_caller]
    fn is_a_symlink(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.matches(IsASymlink)
    }

    #[track_caller]
    fn has_a_root(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.matches(HasARoot)
    }

    #[track_caller]
    fn is_relative(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.matches(IsRelative)
    }

    #[track_caller]
    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.matches(HasFileName::new(expected))
    }

    #[track_caller]
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.matches(HasFileStem::new(expected))
    }

    #[track_caller]
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.matches(HasExtension::new(expected))
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<Path>,
    {
        self.matches(StartsWith::new(expected))
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<Path>,
    {
        self.matches(EndsWith::new(expected))
    }
}

#[cfg(test)]
mod tests {
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

    /// Creates a symlink to this source file in the temp dir and removes it on drop.
    #[cfg(unix)]
    struct TempSymlink(std::path::PathBuf);

    #[cfg(unix)]
    impl TempSymlink {
        fn new(name: &str) -> Self {
            let link = std::env::temp_dir().join(format!("assertr-{name}-{}", std::process::id()));
            let _ = std::fs::remove_file(&link);
            std::os::unix::fs::symlink(source_path!(), &link).expect("symlink created");
            Self(link)
        }
    }

    #[cfg(unix)]
    impl Drop for TempSymlink {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use std::path::Path;

        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            let path = source_path!();
            path.as_path()
                .must()
                .exist()
                .be_a_file()
                .have_file_name("path.rs")
                .have_file_stem("path")
                .have_extension("rs");
            path.parent().unwrap().must().be_a_directory();
            Path::new("../../foo/bar/baz.rs").must().not_exist();
            Path::new("/foo/bar/baz.rs").must().have_a_root();
            source_relative_path!()
                .must()
                .be_relative()
                .start_with("src")
                .end_with("std/path.rs");
            #[cfg(unix)]
            {
                let link = super::TempSymlink::new("path-fluent");
                link.0.as_path().must().be_a_symlink();
            }
        }
    }

    mod renderer_contract {
        use std::{
            ffi::OsStr,
            path::{Path, PathBuf},
        };

        use crate::{
            prelude::*,
            test_support::{
                CustomValueRenderer, NoRenderer, RedactingRenderer, SENTINEL, SentinelRenderer,
                assert_custom_value, assert_redacted, assert_trait_impl,
            },
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, PathBuf, Panic, NoRenderer>
                    => PathAssertions<PathBuf, NoRenderer>
            );
        }

        #[test]
        fn evidence_renders_and_redacts_through_the_active_renderer() {
            macro_rules! case {
                ($subject:literal, $check:expr, $relation:literal, $secrets:expr, |$failure:ident| $typed:expr) => {{
                    let subject = PathBuf::from($subject);
                    let failures = assert_that!(subject)
                        .with_renderer(CustomValueRenderer)
                        .capture($check);
                    let $failure = &failures[0];
                    assert_that!($failure.relation.as_deref()).is_equal_to(Some($relation));
                    assert_custom_value($failure.actual.as_ref().unwrap(), &subject);
                    $typed;

                    let failures = assert_that!(subject)
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .capture($check);
                    assert_redacted(&failures[0], $secrets);
                }};
            }
            let other = OsStr::new("other-file");
            let secrets = &["private-file", "secret", "other-file"];
            case!(
                "private-file.secret",
                |it| it.has_file_name(other),
                "does not have the file name",
                secrets,
                |failure| {
                    assert_custom_value(failure.expected.as_ref().unwrap(), other);
                    assert_custom_value(&failure.facts[0].value, OsStr::new("private-file.secret"));
                }
            );
            case!(
                "private-file.secret",
                |it| it.has_file_stem(other),
                "does not have the file stem",
                secrets,
                |failure| assert_custom_value(&failure.facts[0].value, OsStr::new("private-file"))
            );
            case!(
                "private-file.secret",
                |it| it.has_extension(other),
                "does not have the extension",
                secrets,
                |failure| assert_custom_value(&failure.facts[0].value, OsStr::new("secret"))
            );
            case!(
                "private/path",
                |it| it.starts_with(Path::new("other/path")),
                "does not start with",
                &["private", "other/path"],
                |failure| assert_custom_value(
                    failure.expected.as_ref().unwrap(),
                    Path::new("other/path")
                )
            );
            case!(
                "private/path",
                |it| it.ends_with(Path::new("other/path")),
                "does not end with",
                &["private", "other/path"],
                |failure| assert_custom_value(
                    failure.expected.as_ref().unwrap(),
                    Path::new("other/path")
                )
            );
            let error = Path::new("invalid\0private-path").try_exists().unwrap_err();
            case!(
                "invalid\0private-path",
                PathAssertions::exists,
                "could not determine whether the path exists",
                &["private-path", "nul byte"],
                |failure| assert_custom_value(&failure.facts[0].value, &error)
            );
        }

        #[test]
        fn absent_components_render_verbatim() {
            let failures = assert_that!(PathBuf::from("/"))
                .with_renderer(CustomValueRenderer)
                .capture(|it| it.has_file_name("other-file"));
            assert_that!(format!("{:#}", failures[0].facts[0].value)).is_equal_to("<none>");
            assert_that!(failures[0].facts[0].value.type_name).is_none();
        }

        #[test]
        fn inspection_errors_respect_the_budget_and_continue_in_capture_mode() {
            let failures = assert_that!(Path::new("invalid\0path"))
                .with_renderer(SentinelRenderer)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(3))
                .capture(|it| it.does_not_exist().has_extension("txt"));
            assert_that!(failures).has_length(2);
            assert_that!(format!("{:#}", failures[0].facts[0].value))
                .is_equal_to(format!("<re... {} more characters ...", SENTINEL.len() - 3));
        }
    }

    mod observations {
        use core::cell::Cell;
        use std::{ffi::OsStr, io, path::PathBuf};

        use crate::{
            assertions::std::path::DoesNotExist,
            failure::{FailureBuilder, FailureKind},
            prelude::*,
            test_support::{CustomValueRenderer, assert_custom_value},
        };

        #[test]
        fn evaluation_confirms_existence_only_from_conclusive_observations() {
            use super::super::existence_confirmed;
            let error = |kind| Err(io::Error::from(kind));
            for (observed, exists, absent) in [
                (Ok(true), true, false),
                (Ok(false), false, true),
                (error(io::ErrorKind::NotFound), false, true),
                (error(io::ErrorKind::NotADirectory), false, true),
                (error(io::ErrorKind::PermissionDenied), false, false),
            ] {
                assert_that!(existence_confirmed(&observed, true)).is_equal_to(exists);
                assert_that!(existence_confirmed(&observed, false)).is_equal_to(absent);
            }
        }

        struct Expected<F>(F);
        impl<F: Fn()> AsRef<OsStr> for Expected<F> {
            fn as_ref(&self) -> &OsStr {
                (self.0)();
                OsStr::new("expected.txt")
            }
        }

        #[test]
        fn expected_component_is_converted_once_after_tracking() {
            let calls = Cell::new(0);
            let failures = assert_that!(PathBuf::from("actual.txt")).capture(|root| {
                let expected = Expected(|| {
                    assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                    calls.set(calls.get() + 1);
                });
                root.derive(|path| path).has_file_name(expected);
                root
            });
            assert_that!(failures).has_length(1);
            assert_that!(calls.get()).is_equal_to(1);
        }

        #[test]
        fn explanation_classifies_the_retained_observation_without_inspecting_again() {
            // This path exists, but explanation must only use the supplied observation.
            let path = source_path!();
            let context = AssertionContext::new(&CustomValueRenderer, RenderingBudget::default());
            let explain = |rejected| {
                DoesNotExist
                    .explain(
                        rejected,
                        FailureBuilder::new::<PathBuf>(FailureKind::Other),
                        &context,
                    )
                    .build()
            };

            for (observed, relation) in [
                (Ok(true), "unexpectedly exists"),
                (Ok(false), "does not exist"),
                (
                    Err(io::Error::from(io::ErrorKind::NotADirectory)),
                    "does not exist",
                ),
            ] {
                let failure = explain(Some((&path, observed)));
                assert_that!(failure.relation.as_deref()).is_equal_to(Some(relation));
                assert_that!(failure.facts).is_empty();
            }

            let error = || io::Error::new(io::ErrorKind::PermissionDenied, "inspection denied");
            let failure = explain(Some((&path, Err(error()))));
            assert_that!(failure.relation.as_deref())
                .is_equal_to(Some("could not determine whether the path exists"));
            assert_custom_value(&failure.facts[0].value, &error());

            let failure = explain(None);
            assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not exist"));
            assert_that!(failure.actual).is_none();
            assert_that!(failure.facts).is_empty();
        }
    }

    mod exists {
        use std::path::Path;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let path = Path::new("src/assertions/std/some-non-existing-file.rs");
            assert_caller_location!(assert_that!(path), exists());
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
            assert_that!(|| assert_that!(path).with_location(false).exists())
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `path`

                    Actual: "src/assertions/std/some-non-existing-file.rs"

                    does not exist
                    -------- assertr --------
                "#});
        }

        #[test]
        #[cfg(unix)]
        fn reports_a_path_below_a_file_as_absent() {
            let path = source_path!().join("child");
            let failures = assert_that!(path).capture(PathAssertions::exists);
            assert_that!(failures[0].relation.as_deref()).is_equal_to(Some("does not exist"));
        }
    }

    mod does_not_exist {
        use std::path::Path;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let path = source_path!();
            assert_caller_location!(assert_that!(path.as_path()), does_not_exist());
        }

        #[test]
        fn succeeds_when_absent() {
            assert_that!(Path::new("../../foo/bar/baz.rs")).does_not_exist();
        }

        #[test]
        #[cfg(unix)]
        fn succeeds_when_an_ancestor_is_a_file() {
            let path = source_path!().join("child");
            assert_that!(path.try_exists().unwrap_err().kind())
                .is_equal_to(std::io::ErrorKind::NotADirectory);
            assert_that!(path.as_path()).does_not_exist();
        }

        #[test]
        fn panics_when_present() {
            let path = source_path!();
            let path = path.as_path();
            assert_that!(|| {
                assert_that!(path).with_location(false).does_not_exist();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `path`

                Actual: {path:?}

                unexpectedly exists
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_existence_cannot_be_determined() {
            // A NUL makes inspection fail on every supported platform, even as root.
            let path = Path::new("invalid\0path");
            let error = path.try_exists().unwrap_err();
            let error = format!("{error:#?}").replace('\n', "\n    ");
            assert_that!(|| {
                assert_that!(path).with_location(false).does_not_exist();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `path`

                Actual: "invalid\0path"

                could not determine whether the path exists

                Details:
                  - I/O error: {error}
                -------- assertr --------
            "#});
        }
    }

    mod entry_kinds {
        use std::path::Path;

        use crate::prelude::*;

        #[test]
        fn caller_locations_are_as_expected() {
            let path = source_path!();
            let dir = path.parent().unwrap();
            assert_caller_location!(assert_that!(dir), is_a_file());
            assert_caller_location!(assert_that!(path.as_path()), is_a_directory());
            assert_caller_location!(assert_that!(path.as_path()), is_a_symlink());
        }

        #[test]
        fn succeed_for_the_matching_kind() {
            let path = source_path!();
            assert_that!(path.as_path()).is_a_file();
            assert_that!(path.parent().unwrap()).is_a_directory();
            #[cfg(unix)]
            {
                let link = super::TempSymlink::new("path-succeeds");
                assert_that!(link.0.as_path()).is_a_symlink().is_a_file();
            }
        }

        #[test]
        fn rejections_describe_the_observed_entry() {
            let file = source_path!();
            let dir = file.parent().unwrap();
            let invalid = Path::new("invalid\0path");
            let failures = assert_that!(dir).capture(|it| it.is_a_file().is_a_symlink());
            let failures = failures
                .into_iter()
                .chain(assert_that!(file.as_path()).capture(PathAssertions::is_a_directory))
                .chain(
                    assert_that!(Path::new("assertr-missing-path"))
                        .capture(PathAssertions::is_a_file),
                )
                .chain(
                    assert_that!(invalid)
                        .capture(|it| it.is_a_file().is_a_directory().is_a_symlink()),
                )
                .map(|failure| {
                    let fact = &failure.facts[0];
                    let fact = match &fact.label {
                        None => format!("{:#}", fact.value),
                        Some(label) => label.to_string(),
                    };
                    (failure.relation.unwrap().into_owned(), fact)
                })
                .collect::<Vec<_>>();
            let expected = [
                ("is not a file", "The path is a directory."),
                ("is not a symlink", "The path is a directory."),
                ("is not a directory", "The path is a file."),
                ("is not a file", "The path does not exist."),
                (
                    "could not determine whether the path is a file",
                    "I/O error",
                ),
                (
                    "could not determine whether the path is a directory",
                    "I/O error",
                ),
                (
                    "could not determine whether the path is a symlink",
                    "I/O error",
                ),
            ]
            .map(|(relation, fact)| (relation.to_owned(), fact.to_owned()));
            assert_that!(failures).is_equal_to(expected);
        }

        #[test]
        fn panics_with_the_observed_entry_kind() {
            let path = source_path!();
            let dir = path.parent().unwrap();
            assert_that!(|| assert_that!(dir).with_location(false).is_a_file())
                .panics()
                .has_type::<String>()
                .is_equal_to(indoc::formatdoc! {r"
                    -------- assertr --------
                    Expression: `dir`

                    Actual: {dir:?}

                    is not a file

                    Details:
                      - The path is a directory.
                    -------- assertr --------
                "});
        }
    }

    mod has_a_root {
        use std::path::Path;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Path::new("foo/bar/baz.rs")), has_a_root());
        }

        #[test]
        fn succeeds_when_root() {
            assert_that!(Path::new("/foo/bar/baz.rs")).has_a_root();
        }

        #[test]
        fn panics_when_relative() {
            let path = Path::new("foo/bar/baz.rs");
            assert_that!(|| assert_that!(path).with_location(false).has_a_root())
                .panics()
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
        use std::path::Path;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Path::new("/foo/bar/baz.rs")), is_relative());
        }

        #[test]
        fn accepts_relative_paths_only() {
            assert_that!(Path::new("foo/bar/baz.rs")).is_relative();
            let failures =
                assert_that!(Path::new("/foo/bar/baz.rs")).capture(PathAssertions::is_relative);
            assert_that!(failures[0].relation.as_deref()).is_equal_to(Some("is not relative"));
        }
    }

    mod components {
        use std::path::Path;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_locations_are_as_expected() {
            let path = Path::new("/");
            assert_caller_location!(assert_that!(path), has_file_name("foo"));
            assert_caller_location!(assert_that!(path), has_file_stem("foo"));
            assert_caller_location!(assert_that!(path), has_extension("rs"));
        }

        #[test]
        fn succeed_when_equal() {
            assert_that!(source_relative_path!())
                .has_file_name("path.rs")
                .has_file_stem("path")
                .has_extension("rs");
        }

        #[test]
        fn panics_when_different() {
            let path = source_relative_path!();
            assert_that!(|| {
                assert_that!(path)
                    .with_location(false)
                    .has_file_name("some.json")
            })
            .panics()
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
        fn panics_when_path_has_no_component() {
            let path = Path::new("/");
            assert_that!(|| assert_that!(path).with_location(false).has_extension("rs"))
                .panics()
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

    mod affixes {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_locations_are_as_expected() {
            let path = source_relative_path!();
            assert_caller_location!(assert_that!(path), starts_with("assert"));
            assert_caller_location!(assert_that!(path), ends_with("ath.rs"));
        }

        #[test]
        fn match_whole_components_only() {
            let path = source_relative_path!();
            assert_that!(path)
                .starts_with("src")
                .ends_with("std/path.rs");
            let failures = assert_that!(path).capture(|it| it.ends_with("ath.rs"));
            assert_that!(failures[0].relation.as_deref()).is_equal_to(Some("does not end with"));
        }

        #[test]
        fn panics_when_not_a_whole_component_prefix() {
            let path = source_relative_path!();
            assert_that!(|| {
                assert_that!(path)
                    .with_location(false)
                    .starts_with("assert")
            })
            .panics()
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

    mod adapters {
        use std::path::PathBuf;

        use crate::prelude::*;

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

            assert_that!(failures[0].subject_name.as_deref())
                .is_equal_to(Some("configuration path"));
        }

        #[test]
        fn borrowed_and_owned_paths_support_assertions() {
            let path = source_path!();
            assert_that!(path).exists().is_a_file().has_extension("rs");
            assert_that_owned!(path)
                .exists()
                .is_a_file()
                .has_file_name("path.rs")
                .map(|it| it.unwrap_owned().display().to_string().into())
                .ends_with("src/assertions/std/path.rs");
        }
    }
}
