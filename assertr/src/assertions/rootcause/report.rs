use crate::failure::FailureKind;
use crate::{AssertThat, Mode, ValueRenderer, mode::Panic};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use alloc::format;
use core::any::{TypeId, type_name};
use core::fmt::Display;
use rootcause::markers::Dynamic;

/// Compares the observed direct child count.
pub struct HasChildCount(usize);
impl<'r, C: ?Sized, O, T, R> Expectation<rootcause::ReportRef<'r, C, O, T>, R> for HasChildCount {
    type Success<'a>
        = ()
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    type Rejection<'a>
        = usize
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a rootcause::ReportRef<'r, C, O, T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let count = actual.children().len();
        if count == self.0 { Ok(()) } else { Err(count) }
    }
}
impl<'r, C: ?Sized, O, T, R> ExpectationDiagnostics<rootcause::ReportRef<'r, C, O, T>, R>
    for HasChildCount
where
    R: ValueRenderer<usize>,
{
    const KIND: FailureKind = FailureKind::Length;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a rootcause::ReportRef<'r, C, O, T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("has the expected child count")
                .expected(render.value(&self.0)),
            Some((_, count)) => failure
                .actual(render.value(&count))
                .relation("is not the expected child count")
                .expected(render.value(&self.0)),
        }
    }
}
impl HasChildCount {
    /// Expects this count.
    #[must_use]
    pub const fn new(expected: usize) -> Self {
        Self(expected)
    }
}
/// Compares the observed direct attachment count.
pub struct HasAttachmentCount(usize);
impl<'r, C: ?Sized, O, T, R> Expectation<rootcause::ReportRef<'r, C, O, T>, R>
    for HasAttachmentCount
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    type Rejection<'a>
        = usize
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a rootcause::ReportRef<'r, C, O, T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let count = actual.attachments().len();
        if count == self.0 { Ok(()) } else { Err(count) }
    }
}
impl<'r, C: ?Sized, O, T, R> ExpectationDiagnostics<rootcause::ReportRef<'r, C, O, T>, R>
    for HasAttachmentCount
where
    R: ValueRenderer<usize>,
{
    const KIND: FailureKind = FailureKind::Length;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a rootcause::ReportRef<'r, C, O, T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("has the expected attachment count")
                .expected(render.value(&self.0)),
            Some((_, count)) => failure
                .actual(render.value(&count))
                .relation("is not the expected attachment count")
                .expected(render.value(&self.0)),
        }
    }
}
impl HasAttachmentCount {
    /// Expects this count.
    #[must_use]
    pub const fn new(expected: usize) -> Self {
        Self(expected)
    }
}
/// Compares the rootcause-formatted current context display value once.
pub struct HasCurrentContextDisplayValue<E>(E);
impl<'r, C: ?Sized, O, T, R, E> Expectation<rootcause::ReportRef<'r, C, O, T>, R>
    for HasCurrentContextDisplayValue<E>
where
    E: Display,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    type Rejection<'a>
        = (alloc::string::String, alloc::string::String)
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a rootcause::ReportRef<'r, C, O, T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let actual = format!("{}", actual.format_current_context());
        let expected = format!("{}", self.0);
        if actual == expected {
            Ok(())
        } else {
            Err((actual, expected))
        }
    }
}
impl<'r, C: ?Sized, O, T, R, E> ExpectationDiagnostics<rootcause::ReportRef<'r, C, O, T>, R>
    for HasCurrentContextDisplayValue<E>
where
    E: Display,
    R: ValueRenderer<str>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a rootcause::ReportRef<'r, C, O, T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("has the expected current context display value")
                .expected(render.value((format!("{}", self.0)).as_str())),
            Some((_, (actual, expected))) => failure
                .actual(render.value(actual.as_str()))
                .relation("is not the expected current context display value")
                .expected(render.value(expected.as_str())),
        }
    }
}
impl<E> HasCurrentContextDisplayValue<E> {
    /// Expects this formatted current context.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}
/// Compares the rootcause-formatted current context debug string once.
pub struct HasCurrentContextDebugString<E>(E);
impl<'r, C: ?Sized, O, T, R, E> Expectation<rootcause::ReportRef<'r, C, O, T>, R>
    for HasCurrentContextDebugString<E>
where
    E: AsRef<str>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    type Rejection<'a>
        = (alloc::string::String, &'a str)
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a rootcause::ReportRef<'r, C, O, T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let actual = format!("{:?}", actual.format_current_context());
        let expected = self.0.as_ref();
        if actual == expected {
            Ok(())
        } else {
            Err((actual, expected))
        }
    }
}
impl<'r, C: ?Sized, O, T, R, E> ExpectationDiagnostics<rootcause::ReportRef<'r, C, O, T>, R>
    for HasCurrentContextDebugString<E>
where
    E: AsRef<str>,
    R: ValueRenderer<str>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a rootcause::ReportRef<'r, C, O, T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("has the expected current context debug string")
                .expected(render.value(self.0.as_ref())),
            Some((_, (actual, expected))) => failure
                .actual(render.value(actual.as_str()))
                .relation("is not the expected current context debug string")
                .expected(render.value(expected)),
        }
    }
}
impl<E> HasCurrentContextDebugString<E> {
    /// Expects this formatted current context.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}
/// Checks the concrete type of a report reference's current context.
pub struct HasCurrentContextType<E>(core::marker::PhantomData<fn() -> E>);
impl<'r, C: ?Sized, O, T, R, E> Expectation<rootcause::ReportRef<'r, C, O, T>, R>
    for HasCurrentContextType<E>
where
    E: 'static,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    type Rejection<'a>
        = &'static str
    where
        Self: 'a,
        rootcause::ReportRef<'r, C, O, T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a rootcause::ReportRef<'r, C, O, T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.current_context_type_id() == TypeId::of::<E>() {
            Ok(())
        } else {
            Err(actual.current_context_type_name())
        }
    }
}
impl<'r, C: ?Sized, O, T, R, E> ExpectationDiagnostics<rootcause::ReportRef<'r, C, O, T>, R>
    for HasCurrentContextType<E>
where
    E: 'static,
{
    const KIND: FailureKind = FailureKind::Variant;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a rootcause::ReportRef<'r, C, O, T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        explain_context_type::<E, _>(rejected.map(|(_, name)| name), failure)
    }
}
impl<E> HasCurrentContextType<E> {
    /// Expects the current context to have type `E`.
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}
impl<E> Default for HasCurrentContextType<E> {
    fn default() -> Self {
        Self::new()
    }
}
/// Downcasts the current context once and returns its borrowed value.
pub struct HasCurrentContext<E>(core::marker::PhantomData<fn() -> E>);
impl<'r, O, T, E, R> Expectation<rootcause::ReportRef<'r, Dynamic, O, T>, R>
    for HasCurrentContext<E>
where
    E: 'static,
{
    type Success<'a>
        = &'a E
    where
        Self: 'a,
        rootcause::ReportRef<'r, Dynamic, O, T>: 'a;
    type Rejection<'a>
        = &'static str
    where
        Self: 'a,
        rootcause::ReportRef<'r, Dynamic, O, T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a rootcause::ReportRef<'r, Dynamic, O, T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual
            .downcast_current_context::<E>()
            .ok_or_else(|| actual.current_context_type_name())
    }
}
impl<'r, O, T, E, R> ExpectationDiagnostics<rootcause::ReportRef<'r, Dynamic, O, T>, R>
    for HasCurrentContext<E>
where
    E: 'static,
{
    const KIND: FailureKind = FailureKind::Variant;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(
            &'a rootcause::ReportRef<'r, Dynamic, O, T>,
            Self::Rejection<'a>,
        )>,
        failure: FailureBuilder<Target>,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        explain_context_type::<E, _>(rejected.map(|(_, name)| name), failure)
    }
}
impl<O, T, E, R> Expectation<rootcause::Report<Dynamic, O, T>, R> for HasCurrentContext<E>
where
    E: 'static,
{
    type Success<'a>
        = &'a E
    where
        Self: 'a,
        rootcause::Report<Dynamic, O, T>: 'a;
    type Rejection<'a>
        = &'static str
    where
        Self: 'a,
        rootcause::Report<Dynamic, O, T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a rootcause::Report<Dynamic, O, T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual
            .downcast_current_context::<E>()
            .ok_or_else(|| actual.current_context_type_name())
    }
}
impl<O, T, E, R> ExpectationDiagnostics<rootcause::Report<Dynamic, O, T>, R>
    for HasCurrentContext<E>
where
    E: 'static,
{
    const KIND: FailureKind = FailureKind::Variant;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a rootcause::Report<Dynamic, O, T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        explain_context_type::<E, _>(rejected.map(|(_, name)| name), failure)
    }
}
impl<E> HasCurrentContext<E> {
    /// Expects and borrows a current context of type `E`.
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}
impl<E> Default for HasCurrentContext<E> {
    fn default() -> Self {
        Self::new()
    }
}

fn explain_context_type<E, Target>(
    actual: Option<&'static str>,
    failure: FailureBuilder<Target>,
) -> FailureBuilder<Target> {
    let failure = match actual {
        None => failure.relation("has the expected current context type"),
        Some(name) => failure
            .actual(format_args!("{name}"))
            .relation("is not the expected current context type"),
    };
    failure.expected(format_args!("{}", type_name::<E>()))
}

/// Assertions for owned rootcause reports.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseReportAssertions<R = crate::DebugRenderer> {
    /// Asserts that the report has exactly `expected` direct children.
    fn has_child_count(self, expected: usize) -> Self
    where
        R: Clone + ValueRenderer<usize>;

    /// Asserts that the report has exactly `expected` attachments.
    fn has_attachment_count(self, expected: usize) -> Self
    where
        R: Clone + ValueRenderer<usize>;

    /// Asserts that the report's current context has type `E`.
    fn has_current_context_type<E: 'static>(self) -> Self
    where
        R: Clone;

    /// Asserts that the rootcause-formatted `Display` representation of the current context equals
    /// `expected`.
    ///
    /// This uses `Report::format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type.
    fn has_current_context_display_value(self, expected: impl Display) -> Self
    where
        R: Clone + ValueRenderer<str>;

    /// Asserts that the rootcause-formatted `Debug` representation of the current context equals
    /// `expected`.
    ///
    /// This uses `Report::format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type. Quotes and escape
    /// sequences are compared exactly.
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: Clone + ValueRenderer<str>;
}

impl<C: ?Sized, O, T, M: Mode, R> RootcauseReportAssertions<R>
    for AssertThat<'_, rootcause::Report<C, O, T>, M, R>
where
    O: rootcause::markers::ReportOwnershipMarker,
{
    #[track_caller]
    fn has_child_count(self, expected: usize) -> Self
    where
        R: Clone + ValueRenderer<usize>,
    {
        self.derive_owned(rootcause::Report::as_ref)
            .has_child_count(expected);
        self
    }

    #[track_caller]
    fn has_attachment_count(self, expected: usize) -> Self
    where
        R: Clone + ValueRenderer<usize>,
    {
        self.derive_owned(rootcause::Report::as_ref)
            .has_attachment_count(expected);
        self
    }

    #[track_caller]
    fn has_current_context_type<E: 'static>(self) -> Self
    where
        R: Clone,
    {
        self.derive_owned(rootcause::Report::as_ref)
            .has_current_context_type::<E>();
        self
    }

    #[track_caller]
    fn has_current_context_display_value(self, expected: impl Display) -> Self
    where
        R: Clone + ValueRenderer<str>,
    {
        self.derive_owned(rootcause::Report::as_ref)
            .has_current_context_display_value(expected);
        self
    }

    #[track_caller]
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: Clone + ValueRenderer<str>,
    {
        self.derive_owned(rootcause::Report::as_ref)
            .has_current_context_debug_string(expected);
        self
    }
}

/// Assertions for borrowed rootcause report references.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseReportRefAssertions<R = crate::DebugRenderer> {
    /// Asserts that the report has exactly `expected` direct children.
    fn has_child_count(self, expected: usize) -> Self
    where
        R: ValueRenderer<usize>;

    /// Asserts that the report has exactly `expected` attachments.
    fn has_attachment_count(self, expected: usize) -> Self
    where
        R: ValueRenderer<usize>;

    /// Asserts that the report's current context has type `E`.
    fn has_current_context_type<E: 'static>(self) -> Self;

    /// Asserts that the rootcause-formatted `Display` representation of the current context equals
    /// `expected`.
    ///
    /// This uses `ReportRef::format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type.
    fn has_current_context_display_value(self, expected: impl Display) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the rootcause-formatted `Debug` representation of the current context equals
    /// `expected`.
    ///
    /// This uses `ReportRef::format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type. Quotes and escape
    /// sequences are compared exactly.
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;
}

impl<C: ?Sized, O, T, M: Mode, R> RootcauseReportRefAssertions<R>
    for AssertThat<'_, rootcause::ReportRef<'_, C, O, T>, M, R>
{
    #[track_caller]
    fn has_child_count(self, expected: usize) -> Self
    where
        R: ValueRenderer<usize>,
    {
        self.apply_assertion(HasChildCount::new(expected))
    }

    #[track_caller]
    fn has_attachment_count(self, expected: usize) -> Self
    where
        R: ValueRenderer<usize>,
    {
        self.apply_assertion(HasAttachmentCount::new(expected))
    }

    #[track_caller]
    fn has_current_context_type<E: 'static>(self) -> Self {
        self.apply_assertion(HasCurrentContextType::<E>::new())
    }

    #[track_caller]
    fn has_current_context_display_value(self, expected: impl Display) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.apply_assertion(HasCurrentContextDisplayValue::new(expected))
    }

    #[track_caller]
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.apply_assertion(HasCurrentContextDebugString::new(expected))
    }
}

/// Assertions over the dynamically typed current context of an owned report.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseDynamicReportAssertions<'t, M: Mode, R = crate::DebugRenderer> {
    /// Asserts that this dynamic report's current context has type `E`, then runs additional
    /// assertions on it.
    ///
    /// The closure receives an `AssertThat<E>` borrowing the current context.
    fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
    where
        E: 'static,
        A: for<'a> FnOnce(AssertThat<'a, E, M, R>),
        R: Clone;
}

impl<'t, O, T, M: Mode, R> RootcauseDynamicReportAssertions<'t, M, R>
    for AssertThat<'t, rootcause::Report<Dynamic, O, T>, M, R>
where
    O: rootcause::markers::ReportOwnershipMarker,
{
    #[track_caller]
    fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
    where
        E: 'static,
        A: for<'a> FnOnce(AssertThat<'a, E, M, R>),
        R: Clone,
    {
        if let Some(value) = self.test_assertion(&const { HasCurrentContext::<E>::new() }) {
            assertions(self.derive(|_| value));
        }
        self
    }
}

/// Assertions over the dynamically typed current context of a report reference.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseDynamicReportRefAssertions<'r, M: Mode, R = crate::DebugRenderer> {
    /// Asserts that this dynamic report reference's current context has type `E`, then runs
    /// additional assertions on it.
    ///
    /// The closure receives an `AssertThat<E>` borrowing the current context.
    fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
    where
        E: 'static,
        A: for<'a> FnOnce(AssertThat<'a, E, M, R>),
        R: Clone;
}

impl<'t, 'r, O, T, M: Mode, R> RootcauseDynamicReportRefAssertions<'r, M, R>
    for AssertThat<'t, rootcause::ReportRef<'r, Dynamic, O, T>, M, R>
where
    'r: 't,
{
    #[track_caller]
    fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
    where
        E: 'static,
        A: for<'a> FnOnce(AssertThat<'a, E, M, R>),
        R: Clone,
    {
        if let Some(value) = self.test_assertion(&const { HasCurrentContext::<E>::new() }) {
            assertions(self.derive(|_| value));
        }
        self
    }
}

/// Panic-mode extraction from a dynamic report reference.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseDynamicReportRefExtractAssertions<'t, R = crate::DebugRenderer> {
    /// Asserts that this dynamic report reference's current context has type `E`, then returns an
    /// `AssertThat<E>` borrowing it.
    ///
    /// A type mismatch becomes a formatted assertion failure.
    fn has_current_context<E: 'static>(&'t self) -> AssertThat<'t, E, Panic, R>
    where
        R: Clone;
}

impl<'t, 'r, O, T, R> RootcauseDynamicReportRefExtractAssertions<'t, R>
    for AssertThat<'t, rootcause::ReportRef<'r, Dynamic, O, T>, Panic, R>
where
    'r: 't,
{
    #[track_caller]
    fn has_current_context<E: 'static>(&'t self) -> AssertThat<'t, E, Panic, R>
    where
        R: Clone,
    {
        let value = self
            .test_assertion(&const { HasCurrentContext::<E>::new() })
            .expect("Panic mode raises a context mismatch");
        self.derive(|_| value)
    }
}

/// Panic-mode extraction from an owned dynamic report.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseDynamicReportExtractAssertions<'t, R = crate::DebugRenderer> {
    /// Asserts that this dynamic report's current context has type `E`, then returns an
    /// `AssertThat<E>` borrowing it.
    ///
    /// A type mismatch becomes a formatted assertion failure.
    fn has_current_context<E: 'static>(&'t self) -> AssertThat<'t, E, Panic, R>
    where
        R: Clone;
}

impl<'t, O, T, R> RootcauseDynamicReportExtractAssertions<'t, R>
    for AssertThat<'t, rootcause::Report<Dynamic, O, T>, Panic, R>
{
    #[track_caller]
    fn has_current_context<E: 'static>(&'t self) -> AssertThat<'t, E, Panic, R>
    where
        R: Clone,
    {
        let value = self
            .test_assertion(&const { HasCurrentContext::<E>::new() })
            .expect("Panic mode raises a context mismatch");
        self.derive(|_| value)
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::NoRenderer;
        use rootcause::markers::Dynamic;
        use rootcause::prelude::*;

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            fn assert_dynamic_report<'t, O, T, R>(
                _: &AssertThat<'t, rootcause::Report<Dynamic, O, T>, Panic, R>,
            ) where
                O: rootcause::markers::ReportOwnershipMarker,
                AssertThat<'t, rootcause::Report<Dynamic, O, T>, Panic, R>:
                    RootcauseReportAssertions<R>
                        + RootcauseDynamicReportAssertions<'t, Panic, R>
                        + RootcauseDynamicReportExtractAssertions<'t, R>,
            {
            }

            fn assert_dynamic_report_ref<'t, 'r: 't, O, T, R>(
                _: &AssertThat<'t, rootcause::ReportRef<'r, Dynamic, O, T>, Panic, R>,
            ) where
                AssertThat<'t, rootcause::ReportRef<'r, Dynamic, O, T>, Panic, R>:
                    RootcauseReportRefAssertions<R>
                        + RootcauseDynamicReportRefAssertions<'r, Panic, R>
                        + RootcauseDynamicReportRefExtractAssertions<'t, R>,
            {
            }

            let report = report!("root");
            let assertion = assert_that!(report).with_renderer(NoRenderer);
            assert_dynamic_report(&assertion);

            let report_ref = report.as_ref();
            let assertion = assert_that!(report_ref).with_renderer(NoRenderer);
            assert_dynamic_report_ref(&assertion);
        }
    }

    #[derive(Debug)]
    struct TestError(&'static str);

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(self.0)
        }
    }

    impl core::error::Error for TestError {}

    mod has_length {
        use crate::assertions::HasLength;
        use crate::prelude::*;
        use indoc::formatdoc;
        use rootcause::markers::{Dynamic, SendSync};
        use rootcause::prelude::*;
        use rootcause::report_attachment::ReportAttachment;
        use rootcause::report_attachments::ReportAttachments;
        use rootcause::report_collection::ReportCollection;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let collection: ReportCollection<Dynamic, SendSync> = ReportCollection::new();
            collection.must().have_length(0);
        }

        #[test]
        fn succeeds_when_report_collection_length_matches() {
            let mut collection: ReportCollection<Dynamic, SendSync> = ReportCollection::new();
            collection.push(report!("child").into_cloneable());

            assert_that!(collection).has_length(1).is_not_empty();
        }

        #[test]
        fn succeeds_when_report_collection_length_matches_on_borrowed_collection() {
            let collection: ReportCollection<Dynamic, SendSync> = ReportCollection::new();

            assert_that!(&collection).has_length(0).is_empty();
        }

        #[test]
        fn panics_when_expected_length_does_not_match() {
            let collection: ReportCollection<Dynamic, SendSync> = ReportCollection::new();

            assert_that_panic_by(|| {
                assert_that!(collection).with_location(false).has_length(1);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `collection`

                Actual: ReportCollection{trailing_space}

                does not have the expected length

                Expected: 1

                Details:
                  - Actual length: 0
                -------- assertr --------
            ", trailing_space = " "});
        }

        #[test]
        fn implements_has_length_for_attachments() {
            let mut attachments = ReportAttachments::new_sendsync();
            attachments.push(ReportAttachment::new("metadata").into_dynamic());

            assert_that!(HasLength::length(&attachments)).is_equal_to(1);
            assert_that!(HasLength::is_empty(&attachments)).is_false();
        }
    }

    mod has_child_count {
        use super::TestError;
        use crate::prelude::*;
        use indoc::formatdoc;
        use rootcause::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let report = report!(TestError("root"));
            report.must().have_child_count(0);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(assert_that!(report), has_child_count(1));
            assert_caller_location!(assert_that!(report.as_ref()), has_child_count(1));
        }

        #[test]
        fn counts_are_rendered_as_typed_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = report!(TestError("private-context"));
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.has_child_count(9));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Actual: custom(0)

                is not the expected child count

                Expected: custom(9)
                -------- assertr --------
            "});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &0_usize);
                    assert_custom_value(element.actual().expected.as_ref().unwrap(), &9_usize);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.has_child_count(9));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Actual: <redacted>

                is not the expected child count

                Expected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &["private-context", "9"]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_expected_count_matches() {
            let mut report = report!(TestError("root"));
            report
                .children_mut()
                .push(report!(TestError("child")).into_dynamic().into_cloneable());

            assert_that!(report).has_child_count(1);
        }

        #[test]
        fn panics_when_expected_count_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .has_child_count(1);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Actual: 0

                is not the expected child count

                Expected: 1
                -------- assertr --------
            "});
        }
    }

    mod has_attachment_count {
        use super::TestError;
        use crate::prelude::*;
        use indoc::formatdoc;
        use rootcause::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let report = report!(TestError("root")).attach("metadata");
            report.must().have_attachment_count(2);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(assert_that!(report), has_attachment_count(0));
            assert_caller_location!(assert_that!(report.as_ref()), has_attachment_count(0));
        }

        #[test]
        fn counts_are_rendered_as_typed_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = report!(TestError("private-context"));
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.has_attachment_count(9));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Actual: custom(1)

                is not the expected attachment count

                Expected: custom(9)
                -------- assertr --------
            "});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &1_usize);
                    assert_custom_value(element.actual().expected.as_ref().unwrap(), &9_usize);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.has_attachment_count(9));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Actual: <redacted>

                is not the expected attachment count

                Expected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &["private-context", "9"]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_expected_count_matches() {
            let report = report!(TestError("root")).attach("metadata");

            assert_that!(report).has_attachment_count(2);
        }

        #[test]
        fn panics_when_expected_count_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(report!(TestError("root")).attach("metadata"))
                    .with_location(false)
                    .has_attachment_count(1);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Actual: 2

                is not the expected attachment count

                Expected: 1
                -------- assertr --------
            "});
        }
    }

    mod has_current_context_type {
        use super::TestError;
        use crate::prelude::*;
        use indoc::formatdoc;
        use rootcause::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let report = report!(TestError("root"));
            report.must().have_current_context_type::<TestError>();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(assert_that!(report), has_current_context_type::<String>());
            assert_caller_location!(
                assert_that!(report.as_ref()),
                has_current_context_type::<String>()
            );
        }

        #[test]
        fn succeeds_when_type_matches() {
            assert_that!(report!(TestError("root"))).has_current_context_type::<TestError>();
        }

        #[test]
        fn panics_when_type_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .has_current_context_type::<String>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Actual: {actual_type}

                is not the expected current context type

                Expected: alloc::string::String
                -------- assertr --------
            ", actual_type = core::any::type_name::<TestError>()});
        }
    }

    mod has_current_context_display_value {
        use super::TestError;
        use crate::prelude::*;
        use indoc::formatdoc;
        use rootcause::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let report = report!(TestError("root"));
            report.must().have_current_context_display_value("root");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(
                assert_that!(report),
                has_current_context_display_value("other")
            );
            assert_caller_location!(
                assert_that!(report.as_ref()),
                has_current_context_display_value("other")
            );
        }

        #[test]
        fn succeeds_when_display_value_matches() {
            assert_that!(report!(TestError("root"))).has_current_context_display_value("root");
        }

        #[test]
        fn panics_when_display_value_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .has_current_context_display_value("other");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "root"

                is not the expected current context display value

                Expected: "other"
                -------- assertr --------
            "#});
        }
    }

    mod has_current_context_debug_string {
        use super::TestError;
        use crate::prelude::*;
        use indoc::formatdoc;
        use rootcause::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let report = report!(TestError("root"));
            report
                .must()
                .have_current_context_debug_string(r#"TestError("root")"#);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(
                assert_that!(report),
                has_current_context_debug_string("other")
            );
            assert_caller_location!(
                assert_that!(report.as_ref()),
                has_current_context_debug_string("other")
            );
        }

        #[test]
        fn succeeds_when_debug_string_matches() {
            assert_that!(report!(TestError("root")))
                .has_current_context_debug_string(r#"TestError("root")"#);
        }

        #[test]
        fn compares_complete_formatter_output_without_stripping_quotes() {
            let report = report!("\n");
            let formatted = format!("{:?}", report.format_current_context());
            assert_that!(report).has_current_context_debug_string(&formatted);
            assert_that!(report.as_ref()).has_current_context_debug_string(&formatted);
            let failures = assert_that!(report)
                .capture(|it| it.has_current_context_debug_string(format!("\"{formatted}\"")));
            assert_that!(failures).has_length(1);
            assert_that!(report).has_current_context_display_value("\n");
        }

        #[test]
        fn panics_when_debug_string_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .has_current_context_debug_string("other");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "TestError(\"root\")"

                is not the expected current context debug string

                Expected: "other"
                -------- assertr --------
            "#});
        }
    }

    mod report_ref_assertions {
        use super::TestError;
        use crate::prelude::*;
        use rootcause::prelude::*;

        #[test]
        fn supports_report_ref() {
            let report = report!(TestError("root")).attach("metadata");
            let report_ref = report.as_ref();

            assert_that!(report_ref)
                .has_child_count(0)
                .has_attachment_count(2)
                .has_current_context_type::<TestError>()
                .has_current_context_display_value("root")
                .has_current_context_debug_string(r#"TestError("root")"#);
        }
    }

    mod dynamic_context_assertions {
        use crate::prelude::*;
        use indoc::formatdoc;
        use rootcause::prelude::*;

        mod has_current_context_satisfying {
            use super::*;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let report = report!("root");
                report
                    .must()
                    .have_current_context_satisfying::<&'static str, _>(|context| {
                        context.is_equal_to("root");
                    });
            }

            #[test]
            fn caller_location_is_as_expected() {
                let report = report!("root");
                assert_caller_location!(
                    assert_that!(report),
                    has_current_context_satisfying::<u32, _>(|_| {})
                );
                assert_caller_location!(
                    assert_that!(report.as_ref()),
                    has_current_context_satisfying::<u32, _>(|_| {})
                );
            }

            #[test]
            fn succeeds_when_callback_assertions_pass_in_panic_mode() {
                assert_that!(report!("root")).has_current_context_satisfying::<&'static str, _>(
                    |context| {
                        context.is_equal_to("root");
                    },
                );
            }

            #[test]
            fn captures_failures_from_callback_assertions_in_capture_mode() {
                let failures = assert_that!(report!("root"))
                    .with_location(false)
                    .capture(|it| {
                        it.has_current_context_satisfying::<&'static str, _>(|context| {
                            context.is_equal_to("other");
                        })
                    });

                assert_that!(failures).contains_exactly_satisfying([
                    |it: AssertThat<AssertionFailure, Capture>| {
                        it.has_text_report(formatdoc! {r#"
                            -------- assertr --------
                            Expected: "other"

                              Actual: "root"
                            -------- assertr --------
                        "#});
                    },
                ]);
            }

            #[test]
            fn succeeds_on_report_ref() {
                let report = report!("root");
                let report_ref = report.as_ref();

                assert_that!(report_ref).has_current_context_satisfying::<&'static str, _>(
                    |context| {
                        context.is_equal_to("root");
                    },
                );
            }
        }

        mod has_current_context {
            use super::*;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                let report = report!("root");
                report.must().have_current_context::<&'static str>();
            }

            #[test]
            fn caller_location_is_as_expected() {
                let report = report!("root");
                assert_caller_location!(assert_that!(report), has_current_context::<String>());
                assert_caller_location!(
                    assert_that!(report.as_ref()),
                    has_current_context::<String>()
                );
            }

            #[test]
            fn succeeds_when_type_matches() {
                assert_that!(report!("root"))
                    .has_current_context::<&'static str>()
                    .is_equal_to("root");
            }

            #[test]
            fn succeeds_on_report_ref() {
                let report = report!("root");
                let report_ref = report.as_ref();

                assert_that!(report_ref)
                    .has_current_context::<&'static str>()
                    .is_equal_to("root");
            }

            #[test]
            fn panics_when_type_does_not_match() {
                assert_that_panic_by(|| {
                    assert_that!(report!("root"))
                        .with_location(false)
                        .has_current_context::<String>();
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `report!("root")`

                    Actual: &str

                    is not the expected current context type

                    Expected: alloc::string::String
                    -------- assertr --------
                "#});
            }
        }
    }
}
