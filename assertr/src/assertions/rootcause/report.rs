use alloc::{format, string::String};
use core::{
    any::{TypeId, type_name},
    fmt::Display,
};

use rootcause::markers::Dynamic;

use crate::{
    AssertThat, Mode,
    expectation::{AssertionContext, Evidence, Expectation, composite_items, field::field, passed},
    failure::{FailureBuilder, FailureKind},
    mode::Panic,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Compares the observed direct child count.
#[derive(Debug, Clone, Copy)]
pub struct HasChildCount(usize);

impl HasChildCount {
    /// Expects this count.
    #[must_use]
    pub const fn new(expected: usize) -> Self {
        Self(expected)
    }
}

/// Compares the observed direct attachment count.
#[derive(Debug, Clone, Copy)]
pub struct HasAttachmentCount(usize);

impl HasAttachmentCount {
    /// Expects this count.
    #[must_use]
    pub const fn new(expected: usize) -> Self {
        Self(expected)
    }
}

/// Compares the rootcause-formatted current context display value once.
#[derive(Debug, Clone)]
pub struct HasCurrentContextDisplayValue<E>(E);

impl<E> HasCurrentContextDisplayValue<E> {
    /// Expects this formatted current context.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

/// Compares the rootcause-formatted current context debug string once.
#[derive(Debug, Clone)]
pub struct HasCurrentContextDebugString<E>(E);

impl<E> HasCurrentContextDebugString<E> {
    /// Expects this formatted current context.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

/// Checks the concrete type of a report's current context.
pub struct HasCurrentContextType<E>(core::marker::PhantomData<fn() -> E>);

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

type_selection_traits!(HasCurrentContextType);

/// Checks that a report's current context does not have a specific concrete type.
///
/// This is the negation of [`HasCurrentContextType`], with diagnostics that name the unexpected
/// type.
pub struct DoesNotHaveCurrentContextType<E>(core::marker::PhantomData<fn() -> E>);

impl<E> DoesNotHaveCurrentContextType<E> {
    /// Expects the current context not to have type `E`.
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<E> Default for DoesNotHaveCurrentContextType<E> {
    fn default() -> Self {
        Self::new()
    }
}

type_selection_traits!(DoesNotHaveCurrentContextType);

/// Applies a matcher to the current context of a typed report or report reference, locating its
/// evidence at `.current_context()`.
///
/// Construct it with [`current_context`]. Use it where a matcher is expected, for example to check
/// every report of a collection:
///
/// ```
/// use assertr::{matchers::{each, eq, report::current_context}, prelude::*};
/// use rootcause::prelude::*;
///
/// #[derive(Debug, PartialEq)]
/// enum StoreError {
///     NotFound,
/// }
///
/// impl core::fmt::Display for StoreError {
///     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
///         f.write_str("not found")
///     }
/// }
///
/// let reports = [report!(StoreError::NotFound), report!(StoreError::NotFound)];
/// assert_that!(reports).matches(each(current_context(eq(StoreError::NotFound))));
/// ```
///
/// A dynamic report has no statically known context type. Check it with
/// [`RootcauseDynamicReportAssertions::has_current_context_satisfying`] instead.
#[derive(Debug, Clone)]
pub struct CurrentContext<M>(M);

impl<M> CurrentContext<M> {
    /// Applies `matcher` to the current context.
    #[must_use]
    pub const fn new(matcher: M) -> Self {
        Self(matcher)
    }
}

/// Matches the current context of a typed report against `matcher`. This is a convenience
/// constructor for [`CurrentContext::new`].
#[must_use]
pub const fn current_context<M>(matcher: M) -> CurrentContext<M> {
    CurrentContext::new(matcher)
}

/// Downcasts the current context once and returns its borrowed value.
pub struct HasCurrentContext<E>(core::marker::PhantomData<fn() -> E>);

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

type_selection_traits!(HasCurrentContext);

/// Assertions for rootcause reports and report references.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseReportAssertions<R = DebugRenderer> {
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

    /// Asserts that the report's current context does not have type `E`.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use rootcause::prelude::*;
    ///
    /// assert_that!(report!("timed out")).does_not_have_current_context_type::<u32>();
    /// ```
    fn does_not_have_current_context_type<E: 'static>(self) -> Self;

    /// Asserts that the rootcause-formatted `Display` representation of the current context equals
    /// `expected`.
    ///
    /// This uses rootcause's `format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type.
    fn has_current_context_display_value(self, expected: impl Display) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the rootcause-formatted `Debug` representation of the current context equals
    /// `expected`.
    ///
    /// This uses rootcause's `format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type. Quotes and escape
    /// sequences are compared exactly.
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;
}

/// Assertions over the dynamically typed current context of a report or report reference.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseDynamicReportAssertions<M: Mode, R = DebugRenderer> {
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

/// Panic-mode extraction from a dynamic report or report reference.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseDynamicReportExtractAssertions<'t, R = DebugRenderer> {
    /// Asserts that this dynamic report's current context has type `E`, then returns an
    /// `AssertThat<E>` borrowing it.
    ///
    /// A type mismatch becomes a formatted assertion failure.
    fn current_context<E: 'static>(&'t self) -> AssertThat<'t, E, Panic, R>
    where
        R: Clone;
}

/// Assertions over the statically typed current context of a typed report or report reference,
/// such as a `Report<MyError>`.
///
/// The context type is known at compile time, so these methods cannot fail on their own. They work
/// in every mode. Dynamic reports offer the same names with an explicit context type through
/// [`RootcauseDynamicReportAssertions`] and [`RootcauseDynamicReportExtractAssertions`].
///
/// ```
/// use assertr::prelude::*;
/// use rootcause::prelude::*;
///
/// #[derive(Debug, PartialEq)]
/// enum StoreError {
///     NotFound { id: u32 },
/// }
///
/// impl core::fmt::Display for StoreError {
///     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
///         f.write_str("not found")
///     }
/// }
///
/// fn load(id: u32) -> Result<String, Report<StoreError>> {
///     Err(report!(StoreError::NotFound { id }))
/// }
///
/// assert_that!(load(7))
///     .err()
///     .current_context()
///     .is_equal_to(StoreError::NotFound { id: 7 });
/// ```
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RootcauseTypedReportAssertions<'t, C, M: Mode, R = DebugRenderer> {
    /// Runs additional assertions on the report's current context.
    ///
    /// The closure receives an `AssertThat<C>` borrowing the current context. Like
    /// [`AssertThat::satisfies`], failures of the callback are located at this call.
    fn has_current_context_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, C, M, R>),
        R: Clone;

    /// Returns an assertion borrowing the report's current context.
    ///
    /// Failures of the returned chain propagate to this chain, like [`AssertThat::derive`].
    fn current_context(&'t self) -> AssertThat<'t, C, M, R>
    where
        R: Clone;
}

/// The path segment locating evidence of a [`CurrentContext`] matcher.
const CURRENT_CONTEXT: &str = "current_context()";

/// Implements the expectations and assertion traits for one report subject.
///
/// Owned reports and report references expose the same inspection methods, so both subjects
/// share these definitions without projecting the owned report.
macro_rules! report_impls {
    ($($lt:lifetime)? $report:ident) => {
        impl<$($lt,)? C: ?Sized, O, T, R> Expectation<rootcause::$report<$($lt,)? C, O, T>, R>
            for HasChildCount
        where
            R: ValueRenderer<usize>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;
            type Rejection<'a>
                = usize
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a rootcause::$report<$($lt,)? C, O, T>,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), usize> {
                let count = actual.children().len();
                if count == self.0 { Ok(()) } else { Err(count) }
            }

            const KIND: FailureKind = FailureKind::Length;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a rootcause::$report<$($lt,)? C, O, T>, usize)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                explain_count(
                    self.0,
                    rejected.map(|(_, count)| count),
                    ("has the expected child count", "is not the expected child count"),
                    failure,
                    context,
                )
            }
        }

        impl<$($lt,)? C: ?Sized, O, T, R> Expectation<rootcause::$report<$($lt,)? C, O, T>, R>
            for HasAttachmentCount
        where
            R: ValueRenderer<usize>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;
            type Rejection<'a>
                = usize
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a rootcause::$report<$($lt,)? C, O, T>,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), usize> {
                let count = actual.attachments().len();
                if count == self.0 { Ok(()) } else { Err(count) }
            }

            const KIND: FailureKind = FailureKind::Length;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a rootcause::$report<$($lt,)? C, O, T>, usize)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                explain_count(
                    self.0,
                    rejected.map(|(_, count)| count),
                    (
                        "has the expected attachment count",
                        "is not the expected attachment count",
                    ),
                    failure,
                    context,
                )
            }
        }

        impl<$($lt,)? C: ?Sized, O, T, R, E> Expectation<rootcause::$report<$($lt,)? C, O, T>, R>
            for HasCurrentContextDisplayValue<E>
        where
            E: Display,
            R: ValueRenderer<str>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;
            type Rejection<'a>
                = (String, String)
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a rootcause::$report<$($lt,)? C, O, T>,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), (String, String)> {
                let actual = format!("{}", actual.format_current_context());
                let expected = format!("{}", self.0);
                if actual == expected {
                    Ok(())
                } else {
                    Err((actual, expected))
                }
            }

            const KIND: FailureKind = FailureKind::Equality;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a rootcause::$report<$($lt,)? C, O, T>, (String, String))>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                let (actual, expected) = match rejected {
                    None => (None, format!("{}", self.0)),
                    Some((_, (actual, expected))) => (Some(actual), expected),
                };
                explain_formatted(
                    actual.as_deref(),
                    &expected,
                    (
                        "has the expected current context display value",
                        "is not the expected current context display value",
                    ),
                    failure,
                    context,
                )
            }
        }

        impl<$($lt,)? C: ?Sized, O, T, R, E> Expectation<rootcause::$report<$($lt,)? C, O, T>, R>
            for HasCurrentContextDebugString<E>
        where
            E: AsRef<str>,
            R: ValueRenderer<str>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;
            type Rejection<'a>
                = String
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a rootcause::$report<$($lt,)? C, O, T>,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), String> {
                let actual = format!("{:?}", actual.format_current_context());
                if actual == self.0.as_ref() {
                    Ok(())
                } else {
                    Err(actual)
                }
            }

            const KIND: FailureKind = FailureKind::Equality;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a rootcause::$report<$($lt,)? C, O, T>, String)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                explain_formatted(
                    rejected.as_ref().map(|(_, actual)| actual.as_str()),
                    self.0.as_ref(),
                    (
                        "has the expected current context debug string",
                        "is not the expected current context debug string",
                    ),
                    failure,
                    context,
                )
            }
        }

        impl<$($lt,)? C: ?Sized, O, T, R, E> Expectation<rootcause::$report<$($lt,)? C, O, T>, R>
            for HasCurrentContextType<E>
        where
            E: 'static,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;
            type Rejection<'a>
                = &'static str
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a rootcause::$report<$($lt,)? C, O, T>,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), &'static str> {
                if actual.current_context_type_id() == TypeId::of::<E>() {
                    Ok(())
                } else {
                    Err(actual.current_context_type_name())
                }
            }

            const KIND: FailureKind = FailureKind::Variant;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a rootcause::$report<$($lt,)? C, O, T>, &'static str)>,
                failure: FailureBuilder,
                _context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                explain_context_type::<E>(rejected.map(|(_, name)| name), failure)
            }
        }

        impl<$($lt,)? C: ?Sized, O, T, R, E> Expectation<rootcause::$report<$($lt,)? C, O, T>, R>
            for DoesNotHaveCurrentContextType<E>
        where
            E: 'static,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a,
                rootcause::$report<$($lt,)? C, O, T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a rootcause::$report<$($lt,)? C, O, T>,
                _context: &AssertionContext<'_, R>,
            ) -> Result<(), ()> {
                passed(actual.current_context_type_id() != TypeId::of::<E>())
            }

            const KIND: FailureKind = FailureKind::Variant;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a rootcause::$report<$($lt,)? C, O, T>, ())>,
                failure: FailureBuilder,
                _context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                let relation = match rejected {
                    None => "does not have the current context type",
                    Some(_) => "has the unexpected current context type",
                };
                failure.relation(relation).unexpected(type_name::<E>())
            }
        }

        impl<$($lt,)? C, O, T, R, Mt> Expectation<rootcause::$report<$($lt,)? C, O, T>, R>
            for CurrentContext<Mt>
        where
            Mt: Expectation<C, R>,
        {
            composite_items!(rootcause::$report<$($lt,)? C, O, T>);

            fn evaluate(
                &self,
                actual: &rootcause::$report<$($lt,)? C, O, T>,
                context: &AssertionContext<'_, R>,
            ) -> Result<(), Evidence> {
                field(CURRENT_CONTEXT, |report: &rootcause::$report<$($lt,)? C, O, T>| {
                    report.current_context()
                }, &self.0)
                .evaluate(actual, context)
            }

            const KIND: FailureKind = FailureKind::Matching;

            fn explain(
                &self,
                rejected: Option<(&rootcause::$report<$($lt,)? C, O, T>, Evidence)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                field(CURRENT_CONTEXT, |report: &rootcause::$report<$($lt,)? C, O, T>| {
                    report.current_context()
                }, &self.0)
                .explain(rejected, failure, context)
            }
        }

        impl<$($lt,)? O, T, R, E> Expectation<rootcause::$report<$($lt,)? Dynamic, O, T>, R>
            for HasCurrentContext<E>
        where
            E: 'static,
        {
            type Success<'a>
                = &'a E
            where
                Self: 'a,
                rootcause::$report<$($lt,)? Dynamic, O, T>: 'a;
            type Rejection<'a>
                = &'static str
            where
                Self: 'a,
                rootcause::$report<$($lt,)? Dynamic, O, T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a rootcause::$report<$($lt,)? Dynamic, O, T>,
                _context: &AssertionContext<'_, R>,
            ) -> Result<&'a E, &'static str> {
                actual
                    .downcast_current_context::<E>()
                    .ok_or_else(|| actual.current_context_type_name())
            }

            const KIND: FailureKind = FailureKind::Variant;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a rootcause::$report<$($lt,)? Dynamic, O, T>, &'static str)>,
                failure: FailureBuilder,
                _context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                explain_context_type::<E>(rejected.map(|(_, name)| name), failure)
            }
        }

        impl<$($lt,)? C: ?Sized, O, T, M: Mode, R> RootcauseReportAssertions<R>
            for AssertThat<'_, rootcause::$report<$($lt,)? C, O, T>, M, R>
        {
            #[track_caller]
            fn has_child_count(self, expected: usize) -> Self
            where
                R: ValueRenderer<usize>,
            {
                self.matches(HasChildCount::new(expected))
            }

            #[track_caller]
            fn has_attachment_count(self, expected: usize) -> Self
            where
                R: ValueRenderer<usize>,
            {
                self.matches(HasAttachmentCount::new(expected))
            }

            #[track_caller]
            fn has_current_context_type<E: 'static>(self) -> Self {
                self.matches(HasCurrentContextType::<E>::new())
            }

            #[track_caller]
            fn does_not_have_current_context_type<E: 'static>(self) -> Self {
                self.matches(DoesNotHaveCurrentContextType::<E>::new())
            }

            #[track_caller]
            fn has_current_context_display_value(self, expected: impl Display) -> Self
            where
                R: ValueRenderer<str>,
            {
                self.matches(HasCurrentContextDisplayValue::new(expected))
            }

            #[track_caller]
            fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self
            where
                R: ValueRenderer<str>,
            {
                self.matches(HasCurrentContextDebugString::new(expected))
            }
        }

        impl<$($lt,)? O, T, M: Mode, R> RootcauseDynamicReportAssertions<M, R>
            for AssertThat<'_, rootcause::$report<$($lt,)? Dynamic, O, T>, M, R>
        {
            #[track_caller]
            fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
            where
                E: 'static,
                A: for<'a> FnOnce(AssertThat<'a, E, M, R>),
                R: Clone,
            {
                if let Some(value) = self.test_assertion(&const { HasCurrentContext::<E>::new() }) {
                    assertions(self.derive_as("current_context()", |_| value));
                }
                self
            }
        }

        impl<'t, $($lt,)? C, O, T, M: Mode, R> RootcauseTypedReportAssertions<'t, C, M, R>
            for AssertThat<'t, rootcause::$report<$($lt,)? C, O, T>, M, R>
        {
            #[track_caller]
            fn has_current_context_satisfying<A>(self, assertions: A) -> Self
            where
                A: for<'a> FnOnce(AssertThat<'a, C, M, R>),
                R: Clone,
            {
                self.satisfies_as("current_context()", |report| report.current_context(), assertions)
            }

            fn current_context(&'t self) -> AssertThat<'t, C, M, R>
            where
                R: Clone,
            {
                self.derive_as("current_context()", |report| report.current_context())
            }
        }

        impl<'t, $($lt,)? O, T, R> RootcauseDynamicReportExtractAssertions<'t, R>
            for AssertThat<'t, rootcause::$report<$($lt,)? Dynamic, O, T>, Panic, R>
        {
            #[track_caller]
            fn current_context<E: 'static>(&'t self) -> AssertThat<'t, E, Panic, R>
            where
                R: Clone,
            {
                let value = self.require(&const { HasCurrentContext::<E>::new() });
                self.derive_as("current_context()", |_| value)
            }
        }
    };
}

report_impls!(Report);
report_impls!('r ReportRef);

fn explain_count<R: ValueRenderer<usize>>(
    expected: usize,
    rejected: Option<usize>,
    (relation, negated): (&'static str, &'static str),
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder {
    let render = context.render();
    failure
        .relations(
            rejected.map(|count| render.value(&count)),
            relation,
            negated,
        )
        .expected(render.value(&expected))
}

fn explain_formatted<R: ValueRenderer<str>>(
    actual: Option<&str>,
    expected: &str,
    (relation, negated): (&'static str, &'static str),
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder {
    let render = context.render();
    failure
        .relations(actual.map(|actual| render.value(actual)), relation, negated)
        .expected(render.value(expected))
}

fn explain_context_type<E>(
    actual: Option<&'static str>,
    failure: FailureBuilder,
) -> FailureBuilder {
    failure
        .relations(
            actual.map(crate::renderer::Rendered::from),
            "has the expected current context type",
            "is not the expected current context type",
        )
        .expected(type_name::<E>())
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;
    use rootcause::{
        markers::{Dynamic, SendSync},
        prelude::*,
        report_attachment::ReportAttachment,
        report_attachments::ReportAttachments,
        report_collection::ReportCollection,
    };

    use crate::{assertions::HasLength, prelude::*};

    #[derive(Debug)]
    struct TestError(&'static str);

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(self.0)
        }
    }

    impl core::error::Error for TestError {}

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use rootcause::prelude::*;

        use super::TestError;
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            let report = report!(TestError("root")).attach("metadata");
            report
                .must()
                .have_child_count(0)
                .have_attachment_count(2)
                .have_current_context_type::<TestError>()
                .have_current_context_display_value("root")
                .have_current_context_debug_string(r#"TestError("root")"#);

            let report = report!("root");
            report
                .must()
                .not_have_current_context_type::<u32>()
                .have_current_context_satisfying::<&'static str, _>(|context| {
                    context.is_equal_to("root");
                });

            let report = report!(TestError("root"));
            report.must().have_current_context_satisfying(|context| {
                context.satisfies(
                    |it| &it.0,
                    |message| {
                        message.is_equal_to("root");
                    },
                );
            });
        }
    }

    mod renderer_contract {
        use indoc::formatdoc;
        use rootcause::{markers::Dynamic, prelude::*};

        use super::TestError;
        use crate::{
            prelude::*,
            test_support::{
                CustomValueRenderer, NoRenderer, RedactingRenderer, assert_custom_value,
                assert_redacted, assert_trait_impl,
            },
        };

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            fn assert_dynamic_report<'t, S, R>(_: &AssertThat<'t, S, Panic, R>)
            where
                AssertThat<'t, S, Panic, R>: RootcauseReportAssertions<R>
                    + RootcauseDynamicReportAssertions<Panic, R>
                    + RootcauseDynamicReportExtractAssertions<'t, R>,
            {
            }

            let report: Report<Dynamic> = report!("root");
            assert_dynamic_report(&assert_that!(report).with_renderer(NoRenderer));
            let report_ref = report.as_ref();
            assert_dynamic_report(&assert_that!(report_ref).with_renderer(NoRenderer));

            assert_trait_impl!(
                AssertThat<'static, Report<TestError>, Capture, NoRenderer>
                    => RootcauseReportAssertions<NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Report<TestError>, Capture, NoRenderer>
                    => RootcauseTypedReportAssertions<'static, TestError, Capture, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, rootcause::ReportRef<'static, TestError>, Panic, NoRenderer>
                    => RootcauseTypedReportAssertions<'static, TestError, Panic, NoRenderer>
            );
        }

        #[test]
        fn current_context_matcher_needs_no_renderer_of_its_own() {
            fn assert_expectation<S, E: Expectation<S, NoRenderer>>(_: &E) {}

            let matcher = matchers::report::current_context(matchers::anything());
            assert_expectation::<Report<TestError>, _>(&matcher);
            assert_expectation::<rootcause::ReportRef<'static, TestError>, _>(&matcher);
            let negated = matchers::report::DoesNotHaveCurrentContextType::<u32>::new();
            assert_expectation::<Report<Dynamic>, _>(&negated);
            assert_expectation::<Report<TestError>, _>(&negated);
        }

        #[test]
        fn counts_render_and_redact_through_the_active_renderer() {
            let subject = report!(TestError("private-context"));
            macro_rules! case {
                ($method:ident, $actual:literal, $relation:literal) => {{
                    let failures = assert_that!(subject)
                        .with_renderer(CustomValueRenderer)
                        .with_location(false)
                        .capture(|it| it.$method(9));
                    assert_that!(&failures[0]).has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Expression: `subject`

                        Actual: custom({})

                        {}

                        Expected: custom(9)
                        -------- assertr --------
                    ", $actual, $relation});
                    assert_custom_value(failures[0].actual.as_ref().unwrap(), &$actual);
                    assert_custom_value(failures[0].expected.as_ref().unwrap(), &9_usize);

                    let failures = assert_that!(subject)
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .capture(|it| it.$method(9));
                    assert_redacted(&failures[0], &["private-context", "9"]);
                }};
            }
            case!(has_child_count, 0_usize, "is not the expected child count");
            case!(
                has_attachment_count,
                1_usize,
                "is not the expected attachment count"
            );
        }
    }

    #[test]
    fn report_collections_and_attachments_have_lengths() {
        let mut collection: ReportCollection<Dynamic, SendSync> = ReportCollection::new();
        assert_that!(&collection).has_length(0).is_empty();
        collection.push(report!("child").into_cloneable());
        assert_that!(collection).has_length(1).is_not_empty();

        let mut attachments = ReportAttachments::new_sendsync();
        attachments.push(ReportAttachment::new("metadata").into_dynamic());
        assert_that!(HasLength::length(&attachments)).is_equal_to(1);
        assert_that!(HasLength::is_empty(&attachments)).is_false();
    }

    #[test]
    fn report_ref_supports_report_assertions() {
        let report = report!(TestError("root")).attach("metadata");
        let report_ref = report.as_ref();

        assert_that!(report_ref)
            .has_child_count(0)
            .has_attachment_count(2)
            .has_current_context_type::<TestError>()
            .has_current_context_display_value("root")
            .has_current_context_debug_string(r#"TestError("root")"#);
    }

    mod has_child_count {
        use rootcause::prelude::*;

        use super::TestError;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(assert_that!(report), has_child_count(1));
            assert_caller_location!(assert_that!(report.as_ref()), has_child_count(1));
        }

        #[test]
        fn succeeds_when_expected_count_matches() {
            let mut report = report!(TestError("root"));
            report
                .children_mut()
                .push(report!(TestError("child")).into_dynamic().into_cloneable());

            assert_that!(report).has_child_count(1);
        }
    }

    mod has_attachment_count {
        use rootcause::prelude::*;

        use super::TestError;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(assert_that!(report), has_attachment_count(0));
            assert_caller_location!(assert_that!(report.as_ref()), has_attachment_count(0));
        }

        #[test]
        fn succeeds_when_expected_count_matches() {
            let report = report!(TestError("root")).attach("metadata");

            assert_that!(report).has_attachment_count(2);
        }
    }

    mod has_current_context_type {
        use indoc::formatdoc;
        use rootcause::prelude::*;

        use super::TestError;
        use crate::prelude::*;

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
            assert_that!(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .has_current_context_type::<String>();
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `report!(TestError("root"))`

                Actual: {actual_type}

                is not the expected current context type

                Expected: alloc::string::String
                -------- assertr --------
            "#, actual_type = core::any::type_name::<TestError>()});
        }
    }

    mod does_not_have_current_context_type {
        use indoc::formatdoc;
        use rootcause::prelude::*;

        use super::TestError;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(
                assert_that!(report),
                does_not_have_current_context_type::<TestError>()
            );
            assert_caller_location!(
                assert_that!(report.as_ref()),
                does_not_have_current_context_type::<TestError>()
            );
        }

        #[test]
        fn succeeds_when_type_differs() {
            assert_that!(report!(TestError("root"))).does_not_have_current_context_type::<String>();
            assert_that!(report!("root")).does_not_have_current_context_type::<TestError>();
        }

        #[test]
        fn panics_when_type_matches() {
            assert_that!(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .does_not_have_current_context_type::<TestError>();
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `report!(TestError("root"))`

                has the unexpected current context type

                Unexpected: {unexpected}
                -------- assertr --------
            "#, unexpected = core::any::type_name::<TestError>()});
        }
    }

    mod has_current_context_display_value {
        use indoc::formatdoc;
        use rootcause::prelude::*;

        use super::TestError;
        use crate::prelude::*;

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
            assert_that!(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .has_current_context_display_value("other");
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `report!(TestError("root"))`

                Actual: "root"

                is not the expected current context display value

                Expected: "other"
                -------- assertr --------
            "#});
        }
    }

    mod has_current_context_debug_string {
        use indoc::formatdoc;
        use rootcause::prelude::*;

        use super::TestError;
        use crate::prelude::*;

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
            assert_that!(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .has_current_context_debug_string("other");
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `report!(TestError("root"))`

                Actual: "TestError(\"root\")"

                is not the expected current context debug string

                Expected: "other"
                -------- assertr --------
            "#});
        }
    }

    mod has_current_context_satisfying {
        use super::*;

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
        fn succeeds_on_reports_and_report_refs() {
            let report = report!("root");
            assert_that!(report.as_ref()).has_current_context_satisfying::<&'static str, _>(
                |context| {
                    context.is_equal_to("root");
                },
            );
            assert_that!(report).has_current_context_satisfying::<&'static str, _>(|context| {
                context.is_equal_to("root");
            });
        }

        #[test]
        fn typed_reports_need_no_context_type() {
            let report = report!(TestError("root"));
            assert_that!(report.as_ref()).has_current_context_satisfying(|context| {
                context.satisfies(
                    |it| &it.0,
                    |message| {
                        message.is_equal_to("root");
                    },
                );
            });
        }

        #[test]
        fn typed_callback_failures_are_located_at_the_call() {
            let report = report!(TestError("root"));
            assert_caller_location!(
                assert_that!(report),
                has_current_context_satisfying(|context| {
                    context.satisfies(
                        |it| &it.0,
                        |message| {
                            message.is_equal_to("other");
                        },
                    );
                })
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
                        Expression: `report!("root")`
                        Projection: `.current_context()`

                        Expected: "other"

                          Actual: "root"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod current_context {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!("root");
            assert_caller_location!(assert_that!(report), current_context::<String>());
            assert_caller_location!(assert_that!(report.as_ref()), current_context::<String>());
        }

        #[test]
        fn succeeds_on_reports_and_report_refs() {
            let report = report!("root");
            let report_ref = report.as_ref();
            assert_that!(report_ref)
                .current_context::<&'static str>()
                .is_equal_to("root");
            assert_that!(report)
                .current_context::<&'static str>()
                .is_equal_to("root");
        }

        #[test]
        fn typed_reports_continue_with_their_context_in_every_mode() {
            let report = report!(TestError("root"));
            assert_that!(report).current_context().satisfies(
                |it| &it.0,
                |message| {
                    message.is_equal_to("root");
                },
            );
            let failures = assert_that!(report.as_ref())
                .with_location(false)
                .capture(|it| {
                    it.current_context().satisfies(
                        |it| &it.0,
                        |message| {
                            message.is_equal_to("other");
                        },
                    );
                    it
                });
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn typed_failures_are_located_at_the_continued_assertion() {
            let report = report!(TestError("root"));
            assert_caller_location!(
                assert_that!(report).current_context(),
                satisfies(
                    |it| &it.0,
                    |message| {
                        message.is_equal_to("other");
                    }
                )
            );
        }

        #[test]
        fn panics_when_type_does_not_match() {
            assert_that!(|| {
                assert_that!(report!("root"))
                    .with_location(false)
                    .current_context::<String>();
            })
            .panics()
            .message()
            // `has_current_context_type` pins the exact report of this shared explanation.
            .contains("Actual: &str")
            .contains("is not the expected current context type");
        }
    }
    mod current_context_matcher {
        use indoc::formatdoc;
        use rootcause::prelude::*;

        use super::TestError;
        use crate::{
            matchers::{each, predicate, report::current_context},
            prelude::*,
        };

        #[test]
        fn caller_location_is_as_expected() {
            let report = report!(TestError("root"));
            assert_caller_location!(
                assert_that!(report),
                matches(current_context(predicate(|it: &TestError| it.0 == "other")))
            );
        }

        #[test]
        fn matches_typed_reports_and_report_refs() {
            let reports = [report!(TestError("a")), report!(TestError("b"))];
            assert_that!(reports).matches(each(current_context(predicate(|it: &TestError| {
                !it.0.is_empty()
            }))));
            assert_that!(reports[0].as_ref())
                .matches(current_context(predicate(|it: &TestError| it.0 == "a")));
        }

        #[test]
        fn locates_evidence_at_the_current_context() {
            assert_that!(|| {
                assert_that!(report!(TestError("root")))
                    .with_location(false)
                    .matches(current_context(
                        predicate(|it: &TestError| it.0 == "other").described_as("is other"),
                    ));
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `report!(TestError("root"))`

                does not match

                Nested failures:
                  - At .current_context():
                    Actual: TestError(
                        "root",
                    )

                    does not satisfy the constraint

                    Constraint:
                        is other
                -------- assertr --------
            "#});
        }
    }
}
