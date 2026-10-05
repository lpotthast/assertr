use crate::renderer::{IntoRendered, Rendered};
use crate::{AssertThat, DebugRenderer, Fact, Mode, ValueRenderer, failure::FailureKind};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use std::ops::Deref;
use std::{ffi::OsStr, io, path::Path};

/// Checks path existence, retaining an I/O error on rejection.
///
/// A [`Path::try_exists`] result of `Ok(true)` passes. `Ok(false)` and an
/// [`io::ErrorKind::NotADirectory`] error, which means that an ancestor is not a directory,
/// confirm absence. Any other I/O error is retained as evidence that existence could not be
/// determined.
pub struct Exists;
impl<P: Deref<Target = Path>, R> Expectation<P, R> for Exists {
    type Success<'a>
        = ()
    where
        Self: 'a,
        P: 'a;
    type Rejection<'a>
        = ExistenceRejection
    where
        Self: 'a,
        P: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a P,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        observe_existence(true, actual.deref().try_exists())
    }
}

impl<P: Deref<Target = Path>, R> ExpectationDiagnostics<P, R> for Exists
where
    R: ValueRenderer<P> + ValueRenderer<io::Error>,
{
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a P, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("exists"),
            Some((actual, rejection)) => explain_existence(actual, rejection, failure, context),
        }
    }
}

/// Checks whether a path is absent, rejecting I/O errors that prevent determining existence.
///
/// A [`Path::try_exists`] result of `Ok(false)` and an [`io::ErrorKind::NotADirectory`] error,
/// which means that an ancestor is not a directory, confirm absence. Any other I/O error is
/// retained as evidence that existence could not be determined.
pub struct DoesNotExist;

/// Retained evidence that [`Exists`] or [`DoesNotExist`] could not establish its expected state.
///
/// The evidence distinguishes a confirmed existence or absence from an inspection error. Its
/// representation is private and is consumed by the diagnostic implementations without
/// inspecting again.
pub struct ExistenceRejection {
    reason: ExistenceRejectionReason,
}

enum ExistenceRejectionReason {
    Exists,
    Absent,
    InspectionFailed(io::Error),
}

/// Whether an I/O error confirms that the inspected path is absent.
fn confirms_absence(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
    )
}

fn observe_existence(expected: bool, result: io::Result<bool>) -> Result<(), ExistenceRejection> {
    let observed = match result {
        Err(error) if confirms_absence(&error) => Ok(false),
        other => other,
    };
    let reason = match observed {
        Ok(exists) if exists == expected => return Ok(()),
        Ok(true) => ExistenceRejectionReason::Exists,
        Ok(false) => ExistenceRejectionReason::Absent,
        Err(error) => ExistenceRejectionReason::InspectionFailed(error),
    };
    Err(ExistenceRejection { reason })
}

fn explain_existence<P, R, Target>(
    actual: &P,
    rejection: ExistenceRejection,
    failure: FailureBuilder<Target>,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder<Target>
where
    R: ValueRenderer<P> + ValueRenderer<io::Error>,
{
    let render = context.render();
    let failure = failure.actual(render.value(actual));
    match rejection.reason {
        ExistenceRejectionReason::Exists => failure.relation("unexpectedly exists"),
        ExistenceRejectionReason::Absent => failure.relation("does not exist"),
        ExistenceRejectionReason::InspectionFailed(error) => failure
            .relation("could not determine whether the path exists")
            .fact(Fact::labelled("I/O error", render.value(&error))),
    }
}

impl<P: Deref<Target = Path>, R> Expectation<P, R> for DoesNotExist {
    type Success<'a>
        = ()
    where
        Self: 'a,
        P: 'a;
    type Rejection<'a>
        = ExistenceRejection
    where
        Self: 'a,
        P: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a P,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        observe_existence(false, actual.deref().try_exists())
    }
}

impl<P: Deref<Target = Path>, R> ExpectationDiagnostics<P, R> for DoesNotExist
where
    R: ValueRenderer<P> + ValueRenderer<io::Error>,
{
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a P, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("does not exist"),
            Some((actual, rejection)) => explain_existence(actual, rejection, failure, context),
        }
    }
}

/// Checks whether a path has a root component.
pub struct HasARoot;
impl<P: Deref<Target = Path>, R> Expectation<P, R> for HasARoot {
    type Success<'a>
        = ()
    where
        Self: 'a,
        P: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        P: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a P,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.deref().has_root() {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<P: Deref<Target = Path>, R> ExpectationDiagnostics<P, R> for HasARoot
where
    R: ValueRenderer<P>,
{
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a P, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("has a root"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("does not have a root"),
        }
    }
}

/// Checks whether a path is relative.
pub struct IsRelative;
impl<P: Deref<Target = Path>, R> Expectation<P, R> for IsRelative {
    type Success<'a>
        = ()
    where
        Self: 'a,
        P: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        P: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a P,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.deref().is_relative() {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<P: Deref<Target = Path>, R> ExpectationDiagnostics<P, R> for IsRelative
where
    R: ValueRenderer<P>,
{
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a P, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is relative"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not relative"),
        }
    }
}

/// Retained evidence that [`IsAFile`], [`IsADirectory`], or [`IsASymlink`] rejected a path.
///
/// The evidence describes the observed entry kind, a confirmed absence, or the I/O error that
/// prevented inspecting the path. Its representation is private and is consumed by the
/// diagnostic implementations without inspecting again.
pub struct EntryKindRejection {
    observation: EntryKindObservation,
}

enum EntryKindObservation {
    Directory,
    File,
    OtherEntry,
    Absent,
    InspectionFailed(io::Error),
}

fn observe_entry_kind(
    metadata: io::Result<std::fs::Metadata>,
    matches: impl FnOnce(std::fs::FileType) -> bool,
) -> Result<(), EntryKindRejection> {
    let observation = match metadata {
        Ok(metadata) if matches(metadata.file_type()) => return Ok(()),
        Ok(metadata) if metadata.is_dir() => EntryKindObservation::Directory,
        Ok(metadata) if metadata.is_file() => EntryKindObservation::File,
        Ok(_) => EntryKindObservation::OtherEntry,
        Err(error) if confirms_absence(&error) => EntryKindObservation::Absent,
        Err(error) => EntryKindObservation::InspectionFailed(error),
    };
    Err(EntryKindRejection { observation })
}

fn explain_entry_kind<P, R, Target>(
    actual: &P,
    rejection: EntryKindRejection,
    relations: (&'static str, &'static str),
    failure: FailureBuilder<Target>,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder<Target>
where
    R: ValueRenderer<P> + ValueRenderer<io::Error>,
{
    let (mismatch, inspection_failed) = relations;
    let render = context.render();
    let failure = failure.actual(render.value(actual));
    let note = match rejection.observation {
        EntryKindObservation::Directory => "The path is a directory.",
        EntryKindObservation::File => "The path is a file.",
        EntryKindObservation::OtherEntry => "The path exists.",
        EntryKindObservation::Absent => "The path does not exist.",
        EntryKindObservation::InspectionFailed(error) => {
            return failure
                .relation(inspection_failed)
                .fact(Fact::labelled("I/O error", render.value(&error)));
        }
    };
    failure.relation(mismatch).fact(Fact::note(note))
}

/// Generates a path entry-kind expectation from its metadata source, predicate, and relations.
macro_rules! entry_kind_expectation {
    (
        $(#[$meta:meta])*
        $name:ident,
        metadata: $metadata:ident,
        matches: $matches:expr,
        relations: $met:literal, $unmet:literal, $inspection_failed:literal $(,)?
    ) => {
        $(#[$meta])*
        pub struct $name;
        impl<P: Deref<Target = Path>, R> Expectation<P, R> for $name {
            type Success<'a>
                = ()
            where
                Self: 'a,
                P: 'a;
            type Rejection<'a>
                = EntryKindRejection
            where
                Self: 'a,
                P: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a P,
                _context: &AssertionContext<'_, R>,
            ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
                observe_entry_kind(actual.deref().$metadata(), $matches)
            }
        }

        impl<P: Deref<Target = Path>, R> ExpectationDiagnostics<P, R> for $name
        where
            R: ValueRenderer<P> + ValueRenderer<io::Error>,
        {
            const KIND: FailureKind = FailureKind::Variant;
            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a P, Self::Rejection<'a>)>,
                failure: FailureBuilder<Target>,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder<Target> {
                match rejected {
                    None => failure.relation($met),
                    Some((actual, rejection)) => explain_entry_kind(
                        actual,
                        rejection,
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
    ///
    /// Rejection retains the observed entry kind, confirmed absence, or the I/O error that
    /// prevented inspecting the path.
    IsAFile,
    metadata: metadata,
    matches: |file_type| file_type.is_file(),
    relations: "is a file", "is not a file", "could not determine whether the path is a file",
);

entry_kind_expectation!(
    /// Checks whether the path is a directory, following symbolic links.
    ///
    /// Rejection retains the observed entry kind, confirmed absence, or the I/O error that
    /// prevented inspecting the path.
    IsADirectory,
    metadata: metadata,
    matches: |file_type| file_type.is_dir(),
    relations: "is a directory", "is not a directory",
        "could not determine whether the path is a directory",
);

entry_kind_expectation!(
    /// Checks whether the path itself is a symbolic link, without following it.
    ///
    /// Rejection retains the observed entry kind, confirmed absence, or the I/O error that
    /// prevented inspecting the path.
    IsASymlink,
    metadata: symlink_metadata,
    matches: |file_type| file_type.is_symlink(),
    relations: "is a symlink", "is not a symlink",
        "could not determine whether the path is a symlink",
);

/// Rejected path component together with the expected component.
type ComponentRejection<'a> = (Option<&'a OsStr>, &'a OsStr);

fn evaluate_component<'a>(
    component: Option<&'a OsStr>,
    expected: &'a OsStr,
) -> Result<(), ComponentRejection<'a>> {
    if component == Some(expected) {
        Ok(())
    } else {
        Err((component, expected))
    }
}

fn explain_component<P, R, Target>(
    expected: &impl AsRef<OsStr>,
    rejected: Option<(&P, ComponentRejection<'_>)>,
    labels: (&'static str, &'static str, &'static str),
    failure: FailureBuilder<Target>,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder<Target>
where
    R: ValueRenderer<P> + ValueRenderer<OsStr>,
{
    let (met, unmet, actual_label) = labels;
    let render = context.render();
    match rejected {
        None => failure
            .relation(met)
            .expected(render.value(expected.as_ref())),
        Some((actual, (component, expected))) => failure
            .actual(render.value(actual))
            .relation(unmet)
            .expected(render.value(expected))
            .fact(Fact::labelled(
                actual_label,
                component.map_or_else(
                    || Rendered::verbatim("<none>".into()),
                    |value| render.value(value).into_rendered(),
                ),
            )),
    }
}

/// Generates a path-component equality expectation from its component accessor and labels.
macro_rules! component_expectation {
    ($(#[$meta:meta])* $name:ident, $accessor:ident, $component:literal) => {
        $(#[$meta])*
        pub struct $name<E>(E);
        impl<P: Deref<Target = Path>, E, R> Expectation<P, R> for $name<E>
        where
            E: AsRef<OsStr>,
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
            ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
                evaluate_component(actual.deref().$accessor(), self.0.as_ref())
            }
        }

        impl<P: Deref<Target = Path>, E, R> ExpectationDiagnostics<P, R> for $name<E>
        where
            E: AsRef<OsStr>,
            R: ValueRenderer<P> + ValueRenderer<OsStr>,
        {
            const KIND: FailureKind = FailureKind::Equality;
            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a P, Self::Rejection<'a>)>,
                failure: FailureBuilder<Target>,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder<Target> {
                explain_component(
                    &self.0,
                    rejected,
                    (
                        concat!("has the ", $component),
                        concat!("does not have the ", $component),
                        concat!("Actual ", $component),
                    ),
                    failure,
                    context,
                )
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

/// Checks whether a path starts with the expected whole components.
pub struct StartsWith<E>(E);
impl<P: Deref<Target = Path>, E, R> Expectation<P, R> for StartsWith<E>
where
    E: AsRef<Path>,
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
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.0.as_ref();
        if actual.deref().starts_with(expected) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<P: Deref<Target = Path>, E, R> ExpectationDiagnostics<P, R> for StartsWith<E>
where
    E: AsRef<Path>,
    R: ValueRenderer<P> + ValueRenderer<Path>,
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a P, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("starts with")
                .expected(render.value(self.0.as_ref()))
                .fact(Fact::note("Only whole path components are matched.")),
            Some((actual, expected)) => failure
                .actual(render.value(actual))
                .relation("does not start with")
                .expected(render.value(expected))
                .fact(Fact::note("Only whole path components are matched.")),
        }
    }
}

impl<E> StartsWith<E> {
    /// Expects these whole path components.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

/// Checks whether a path ends with the expected whole components.
pub struct EndsWith<E>(E);
impl<P: Deref<Target = Path>, E, R> Expectation<P, R> for EndsWith<E>
where
    E: AsRef<Path>,
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
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.0.as_ref();
        if actual.deref().ends_with(expected) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<P: Deref<Target = Path>, E, R> ExpectationDiagnostics<P, R> for EndsWith<E>
where
    E: AsRef<Path>,
    R: ValueRenderer<P> + ValueRenderer<Path>,
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a P, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("ends with")
                .expected(render.value(self.0.as_ref()))
                .fact(Fact::note("Only whole path components are matched.")),
            Some((actual, expected)) => failure
                .actual(render.value(actual))
                .relation("does not end with")
                .expected(render.value(expected))
                .fact(Fact::note("Only whole path components are matched.")),
        }
    }
}

impl<E> EndsWith<E> {
    /// Expects these whole path components.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

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
    fn exists(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>;

    /// Asserts that the path does not exist.
    ///
    /// Passes when [`Path::try_exists`] returns `Ok(false)` or an
    /// [`io::ErrorKind::NotADirectory`] error, which confirms that an ancestor is not a
    /// directory. An existing path or any other I/O error while checking existence is reported as
    /// an assertion failure, retaining the error as a fact.
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
        self.apply_assertion(Exists)
    }

    #[track_caller]
    fn does_not_exist(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.apply_assertion(DoesNotExist)
    }

    #[track_caller]
    fn is_a_file(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.apply_assertion(IsAFile)
    }

    #[track_caller]
    fn is_a_directory(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.apply_assertion(IsADirectory)
    }

    #[track_caller]
    fn is_a_symlink(self) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
    {
        self.apply_assertion(IsASymlink)
    }

    #[track_caller]
    fn has_a_root(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.apply_assertion(HasARoot)
    }

    #[track_caller]
    fn is_relative(self) -> Self
    where
        R: ValueRenderer<P>,
    {
        self.apply_assertion(IsRelative)
    }

    #[track_caller]
    fn has_file_name(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.apply_assertion(HasFileName::new(expected))
    }

    #[track_caller]
    fn has_file_stem(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.apply_assertion(HasFileStem::new(expected))
    }

    #[track_caller]
    fn has_extension(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<OsStr>,
    {
        self.apply_assertion(HasExtension::new(expected))
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<Path>,
    {
        self.apply_assertion(StartsWith::new(expected))
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<Path>) -> Self
    where
        R: ValueRenderer<P> + ValueRenderer<Path>,
    {
        self.apply_assertion(EndsWith::new(expected))
    }
}

#[cfg(test)]
mod tests {
    mod observations {
        use crate::prelude::*;
        use core::cell::Cell;
        use std::{ffi::OsStr, path::PathBuf};

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
    }

    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use std::path::PathBuf;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, PathBuf, Panic, NoRenderer>
                    => PathAssertions<PathBuf, NoRenderer>
            );
            assert_trait_impl!(
                crate::assertions::std::path::DoesNotExist => Expectation<PathBuf, NoRenderer>
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

    mod assertions {
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
            fn caller_location_is_as_expected() {
                let path = Path::new("src/assertions/std/some-non-existing-file.rs");
                assert_caller_location!(assert_that!(path), exists());
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
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| item).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("invalid\0private-path")

                    could not determine whether the path exists

                    Details:
                      - I/O error: custom({error:?})
                    -------- assertr --------
                "#});

                        assert_custom_value(&element.actual().facts[0].value, &error);
                    },
                ]);
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(PathAssertions::exists);
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    could not determine whether the path exists

                    Details:
                      - I/O error: <redacted>
                    -------- assertr --------
                "});

                        assert_redacted(element.actual(), &["private-path", "nul byte"]);
                    },
                ]);
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

            #[test]
            #[cfg(unix)]
            fn reports_a_path_below_a_file_as_absent() {
                let path = source_path!().join("child");
                let path = path.as_path();
                assert_that_panic_by(|| assert_that!(path).with_location(false).exists())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r"
                        -------- assertr --------
                        Expression: `path`

                        Actual: {path:?}

                        does not exist
                        -------- assertr --------
                    "});
            }
        }

        mod does_not_exist {
            use crate::prelude::*;
            use indoc::formatdoc;
            use std::path::Path;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                Path::new("../../foo/bar/baz.rs").must().not_exist();
            }

            #[test]
            fn caller_location_is_as_expected() {
                let path = source_path!();
                assert_caller_location!(assert_that!(path.as_path()), does_not_exist());
            }

            #[test]
            fn succeeds_when_absent() {
                let path = Path::new("../../foo/bar/baz.rs");
                assert_that!(path).does_not_exist();
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
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).does_not_exist();
                })
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
                assert_that_panic_by(|| {
                    assert_that!(path).with_location(false).does_not_exist();
                })
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

            #[test]
            fn captures_inspection_errors_through_the_active_renderer() {
                use crate::test_support::{CustomValueRenderer, assert_custom_value};

                let path = Path::new("invalid\0private-path");
                let error = path.try_exists().unwrap_err();
                let failures = assert_that!(path)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.does_not_exist().has_extension("txt"));
                // Capture continues after the inspection error and records the next assertion too.
                assert_that!(failures).has_length(2);
                assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `path`

                    Actual: custom("invalid\0private-path")

                    could not determine whether the path exists

                    Details:
                      - I/O error: custom({error:?})
                    -------- assertr --------
                "#});
                assert_that!(failures[0].facts).has_length(1);
                assert_custom_value(&failures[0].facts[0].value, &error);
            }

            #[test]
            fn inspection_error_rendering_respects_the_budget() {
                use crate::test_support::{SENTINEL, SentinelRenderer, rendered_text};

                let path = Path::new("invalid\0path");
                let failures = assert_that!(path)
                    .with_renderer(SentinelRenderer)
                    .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(3))
                    .capture(PathAssertions::does_not_exist);
                assert_that!(failures).has_length(1);
                assert_that!(rendered_text(&failures[0].facts[0].value))
                    .is_equal_to(format!("<re... {} more characters ...", SENTINEL.len() - 3));
            }

            mod observations {
                use crate::assertions::std::path::{
                    DoesNotExist, ExistenceRejectionReason, observe_existence,
                };
                use crate::failure::{FailureBuilder, FailureKind};
                use crate::prelude::*;
                use crate::test_support::{CustomValueRenderer, NoRenderer, assert_custom_value};
                use std::{io, path::PathBuf};

                #[test]
                fn confirmed_absence_succeeds() {
                    assert_that!(observe_existence(false, Ok(false)).is_ok()).is_true();
                }

                #[test]
                fn not_a_directory_error_confirms_absence() {
                    let not_a_directory = || io::Error::from(io::ErrorKind::NotADirectory);
                    assert_that!(observe_existence(false, Err(not_a_directory())).is_ok())
                        .is_true();
                    let rejection = observe_existence(true, Err(not_a_directory()))
                        .err()
                        .unwrap();
                    assert_that!(matches!(rejection.reason, ExistenceRejectionReason::Absent))
                        .is_true();
                }

                #[test]
                fn confirmed_existence_retains_existing_path_evidence() {
                    let rejection = observe_existence(false, Ok(true)).err().unwrap();
                    assert_that!(matches!(rejection.reason, ExistenceRejectionReason::Exists))
                        .is_true();
                }

                #[test]
                fn inspection_failure_retains_the_original_error() {
                    let error =
                        io::Error::new(io::ErrorKind::PermissionDenied, "inspection denied");
                    let payload = core::ptr::from_ref(error.get_ref().unwrap());
                    let rejection = observe_existence(false, Err(error)).err().unwrap();
                    let ExistenceRejectionReason::InspectionFailed(error) = rejection.reason else {
                        panic!("inspection error must not establish existence or absence");
                    };
                    assert_that!(error.kind()).is_equal_to(io::ErrorKind::PermissionDenied);
                    assert_that!(core::ptr::eq(error.get_ref().unwrap(), payload)).is_true();
                }

                #[test]
                fn explanation_uses_the_retained_error_without_inspecting_again() {
                    // This path exists, but explanation must use the supplied failed observation.
                    let path = source_path!();
                    let error =
                        io::Error::new(io::ErrorKind::PermissionDenied, "inspection denied");
                    let rejection = observe_existence(false, Err(error)).err().unwrap();
                    let context =
                        AssertionContext::new(&CustomValueRenderer, RenderingBudget::default());
                    let failure = DoesNotExist
                        .explain(
                            Some((&path, rejection)),
                            FailureBuilder::detached::<PathBuf>(FailureKind::Other),
                            &context,
                        )
                        .build();
                    assert_that!(failure.relation.as_deref())
                        .is_equal_to(Some("could not determine whether the path exists"));
                    assert_that!(failure.facts).has_length(1);
                    assert_that!(failure.facts[0].label)
                        .is_equal_to(alloc::borrow::Cow::Borrowed("I/O error"));
                    assert_custom_value(
                        &failure.facts[0].value,
                        &io::Error::new(io::ErrorKind::PermissionDenied, "inspection denied"),
                    );
                }

                #[test]
                fn evaluation_needs_no_renderer() {
                    let path = PathBuf::from("invalid\0path");
                    let context = AssertionContext::new(&NoRenderer, RenderingBudget::default());
                    assert_that!(DoesNotExist.evaluate(&path, &context).is_err()).is_true();
                }

                #[test]
                fn missing_subject_describes_absence_without_an_inspection_error() {
                    let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
                    let failure = <DoesNotExist as ExpectationDiagnostics<PathBuf>>::explain(
                        &DoesNotExist,
                        None,
                        FailureBuilder::detached::<PathBuf>(FailureKind::Other),
                        &context,
                    )
                    .build();
                    assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not exist"));
                    assert_that!(failure.actual).is_none();
                    assert_that!(failure.facts).is_empty();
                }
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
            fn caller_location_is_as_expected() {
                let path = source_path!();
                let dir = path.parent().unwrap();
                assert_caller_location!(assert_that!(dir).exists(), is_a_file());
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

            #[test]
            #[cfg(unix)]
            fn reports_a_path_below_a_file_as_absent() {
                let path = source_path!().join("child");
                let path = path.as_path();
                assert_that_panic_by(|| assert_that!(path).with_location(false).is_a_file())
                    .has_type::<String>()
                    .is_equal_to(indoc::formatdoc! {r"
                        -------- assertr --------
                        Expression: `path`

                        Actual: {path:?}

                        is not a file

                        Details:
                          - The path does not exist.
                        -------- assertr --------
                    "});
            }

            #[test]
            fn retains_inspection_errors() {
                // A NUL makes inspection fail on every supported platform, even as root.
                let path = std::path::Path::new("invalid\0path");
                let error = path.metadata().unwrap_err();
                let error = format!("{error:#?}").replace('\n', "\n    ");
                assert_that_panic_by(|| assert_that!(path).with_location(false).is_a_file())
                    .has_type::<String>()
                    .is_equal_to(indoc::formatdoc! {r#"
                        -------- assertr --------
                        Expression: `path`

                        Actual: "invalid\0path"

                        could not determine whether the path is a file

                        Details:
                          - I/O error: {error}
                        -------- assertr --------
                    "#});
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
            fn caller_location_is_as_expected() {
                let path = source_path!();
                assert_caller_location!(assert_that!(path.as_path()).exists(), is_a_directory());
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

            #[test]
            fn retains_inspection_errors() {
                let path = std::path::Path::new("invalid\0path");
                assert_that_panic_by(|| assert_that!(path).with_location(false).is_a_directory())
                    .has_type::<String>()
                    .contains("could not determine whether the path is a directory")
                    .contains("Details:\n  - I/O error: ");
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
            fn caller_location_is_as_expected() {
                let path = source_relative_path!();
                assert_caller_location!(assert_that!(path), is_a_symlink());
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

            #[test]
            fn retains_inspection_errors() {
                let path = std::path::Path::new("invalid\0path");
                assert_that_panic_by(|| assert_that!(path).with_location(false).is_a_symlink())
                    .has_type::<String>()
                    .contains("could not determine whether the path is a symlink")
                    .contains("Details:\n  - I/O error: ");
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
            fn caller_location_is_as_expected() {
                let path = Path::new("foo/bar/baz.rs");
                assert_caller_location!(assert_that!(path), has_a_root());
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
            fn caller_location_is_as_expected() {
                let path = Path::new("/foo/bar/baz.rs");
                assert_caller_location!(assert_that!(path), is_relative());
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
            fn caller_location_is_as_expected() {
                let path = Path::new("/");
                assert_caller_location!(assert_that!(path), has_file_name("foo"));
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
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| item).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private-\xFF.bin")

                    does not have the file name

                    Expected: custom("other-\xFE.bin")

                    Details:
                      - Actual file name: custom("private-\xFF.bin")
                    -------- assertr --------
                "#});

                        assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                        assert_custom_value(
                            &element.actual().facts[0].value,
                            subject.file_name().unwrap(),
                        );
                    },
                ]);
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
            fn caller_location_is_as_expected() {
                let path = Path::new("/");
                assert_caller_location!(assert_that!(path), has_file_stem("foo"));
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
            fn caller_location_is_as_expected() {
                let path = Path::new("/");
                assert_caller_location!(assert_that!(path), has_extension("rs"));
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
            fn caller_location_is_as_expected() {
                let path = source_relative_path!();
                assert_caller_location!(assert_that!(path), starts_with("assert"));
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
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element
                            .derive(|value| value)
                            .has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private/path")

                    does not start with

                    Expected: custom("other/path")

                    Details:
                      - Only whole path components are matched.
                    -------- assertr --------
                "#});

                        assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                    },
                ]);
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.starts_with(operand));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    does not start with

                    Expected: <redacted>

                    Details:
                      - Only whole path components are matched.
                    -------- assertr --------
                "});

                        assert_redacted(element.actual(), &["private", "other/path"]);
                    },
                ]);
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
            fn caller_location_is_as_expected() {
                let path = source_relative_path!();
                assert_caller_location!(assert_that!(path), ends_with("ath.rs"));
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
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element
                            .derive(|value| value)
                            .has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom("private/path")

                    does not end with

                    Expected: custom("other/path")

                    Details:
                      - Only whole path components are matched.
                    -------- assertr --------
                "#});

                        assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                    },
                ]);
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.ends_with(operand));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    does not end with

                    Expected: <redacted>

                    Details:
                      - Only whole path components are matched.
                    -------- assertr --------
                "});

                        assert_redacted(element.actual(), &["private", "other/path"]);
                    },
                ]);
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

    mod component_rendering {
        use crate::ValueRenderer;
        use crate::prelude::*;
        use crate::test_support::{
            CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            rendered_text,
        };
        use indoc::formatdoc;
        use std::ffi::OsStr;
        use std::path::{Path, PathBuf};

        #[derive(Clone, Copy)]
        enum Component {
            FileName,
            FileStem,
            Extension,
        }

        impl Component {
            const ALL: [Self; 3] = [Self::FileName, Self::FileStem, Self::Extension];

            fn label(self) -> &'static str {
                match self {
                    Self::FileName => "file name",
                    Self::FileStem => "file stem",
                    Self::Extension => "extension",
                }
            }

            fn of(self, path: &Path) -> Option<&OsStr> {
                match self {
                    Self::FileName => path.file_name(),
                    Self::FileStem => path.file_stem(),
                    Self::Extension => path.extension(),
                }
            }

            fn check<'t, R>(
                self,
                it: AssertThat<'t, PathBuf, Capture, R>,
                operand: &OsStr,
            ) -> AssertThat<'t, PathBuf, Capture, R>
            where
                R: ValueRenderer<PathBuf> + ValueRenderer<OsStr>,
            {
                match self {
                    Self::FileName => it.has_file_name(operand),
                    Self::FileStem => it.has_file_stem(operand),
                    Self::Extension => it.has_extension(operand),
                }
            }
        }

        #[test]
        fn renders_typed_components_through_the_active_renderer() {
            let subject = PathBuf::from("private-file.secret");
            let operand = OsStr::new("other-file");
            for component in Component::ALL {
                let label = component.label();
                let actual_component = component.of(&subject).unwrap();
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| component.check(it, operand));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| item).has_text_report(formatdoc! {r#"
                            -------- assertr --------
                            Expression: `subject`

                            Actual: custom("private-file.secret")

                            does not have the {label}

                            Expected: custom("other-file")

                            Details:
                              - Actual {label}: custom({actual_component:?})
                            -------- assertr --------
                        "#});
                        assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                        assert_custom_value(&element.actual().facts[0].value, actual_component);
                    },
                ]);

                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| component.check(it, operand));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                            -------- assertr --------
                            Expression: `subject`

                            Actual: <redacted>

                            does not have the {label}

                            Expected: <redacted>

                            Details:
                              - Actual {label}: <redacted>
                            -------- assertr --------
                        "});
                        assert_redacted(
                            element.actual(),
                            &["private-file", "secret", "other-file"],
                        );
                    },
                ]);
            }
        }

        #[test]
        fn renders_absent_components_verbatim() {
            let subject = PathBuf::from("/");
            let operand = OsStr::new("other-file");
            for component in Component::ALL {
                let label = component.label();
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| component.check(it, operand));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| item).has_text_report(formatdoc! {r#"
                            -------- assertr --------
                            Expression: `subject`

                            Actual: custom("/")

                            does not have the {label}

                            Expected: custom("other-file")

                            Details:
                              - Actual {label}: <none>
                            -------- assertr --------
                        "#});
                        element
                            .derive_owned(|item| rendered_text(&item.facts[0].value))
                            .is_equal_to("<none>");
                        element
                            .derive(|item| &item.facts[0].value.type_name)
                            .is_equal_to(None);
                    },
                ]);
            }
        }
    }

    mod adapters {
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

            assert_that!(&failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| value.subject_name.as_deref())
                        .is_equal_to(Some("configuration path"));
                },
            ]);
        }

        #[test]
        fn borrowed_path_supports_assertions() {
            let path = source_relative_path!();
            assert_that!(path)
                .is_relative()
                .starts_with("src")
                .ends_with("std/path.rs")
                .has_file_name("path.rs");
        }

        #[test]
        fn borrowed_path_buf_supports_assertions() {
            let path = source_path!();
            assert_that!(path).exists().is_a_file().has_extension("rs");
        }

        #[test]
        fn owned_path_buf_preserves_owned_continuation() {
            let path = source_path!();
            assert_that_owned!(path)
                .exists()
                .is_a_file()
                .has_file_name("path.rs")
                .has_file_stem("path")
                .has_extension("rs")
                .map(|it| it.unwrap_owned().display().to_string().into())
                .ends_with("src/assertions/std/path.rs");
        }
    }
}
