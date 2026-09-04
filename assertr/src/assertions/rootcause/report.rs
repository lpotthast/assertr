use crate::assertions::core::strip_quotation_marks;
use crate::failure::FailureKind;
use crate::{AssertThat, Mode, mode::Panic};
use alloc::format;
use core::any::{TypeId, type_name};
use core::fmt::Display;
use rootcause::markers::Dynamic;

/// Assertions for owned rootcause reports.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RootcauseReportAssertions<R = crate::DebugRenderer> {
    /// Asserts that the report has exactly `expected` direct children.
    fn has_child_count(self, expected: usize) -> Self
    where
        R: Clone;

    /// Asserts that the report has exactly `expected` attachments.
    fn has_attachment_count(self, expected: usize) -> Self
    where
        R: Clone;

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
        R: Clone;

    /// Asserts that the rootcause-formatted `Debug` representation of the current context equals
    /// `expected`.
    ///
    /// This uses `Report::format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type. One leading and one
    /// trailing double quote, when present, are removed before comparison.
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: Clone;
}

impl<C: ?Sized, O, T, M: Mode, R> RootcauseReportAssertions<R>
    for AssertThat<'_, rootcause::Report<C, O, T>, M, R>
where
    O: rootcause::markers::ReportOwnershipMarker,
{
    #[track_caller]
    fn has_child_count(self, expected: usize) -> Self
    where
        R: Clone,
    {
        self.derive_owned(rootcause::Report::as_ref)
            .has_child_count(expected);
        self
    }

    #[track_caller]
    fn has_attachment_count(self, expected: usize) -> Self
    where
        R: Clone,
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
        R: Clone,
    {
        self.derive_owned(rootcause::Report::as_ref)
            .has_current_context_display_value(expected);
        self
    }

    #[track_caller]
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: Clone,
    {
        self.derive_owned(rootcause::Report::as_ref)
            .has_current_context_debug_string(expected);
        self
    }
}

/// Assertions for borrowed rootcause report references.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RootcauseReportRefAssertions {
    /// Asserts that the report has exactly `expected` direct children.
    fn has_child_count(self, expected: usize) -> Self;

    /// Asserts that the report has exactly `expected` attachments.
    fn has_attachment_count(self, expected: usize) -> Self;

    /// Asserts that the report's current context has type `E`.
    fn has_current_context_type<E: 'static>(self) -> Self;

    /// Asserts that the rootcause-formatted `Display` representation of the current context equals
    /// `expected`.
    ///
    /// This uses `ReportRef::format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type.
    fn has_current_context_display_value(self, expected: impl Display) -> Self;

    /// Asserts that the rootcause-formatted `Debug` representation of the current context equals
    /// `expected`.
    ///
    /// This uses `ReportRef::format_current_context()`, honoring rootcause formatter hooks and
    /// preformatted contexts without requiring the concrete context type. One leading and one
    /// trailing double quote, when present, are removed before comparison.
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self;
}

impl<C: ?Sized, O, T, M: Mode, R> RootcauseReportRefAssertions
    for AssertThat<'_, rootcause::ReportRef<'_, C, O, T>, M, R>
{
    #[track_caller]
    fn has_child_count(self, expected: usize) -> Self {
        self.track_assertion();
        let actual = self.actual().children().len();

        if actual != expected {
            self.failure(FailureKind::Length)
                .actual(format_args!("{actual:?}"))
                .relation("is not the expected child count")
                .expected(format_args!("{expected:?}"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_attachment_count(self, expected: usize) -> Self {
        self.track_assertion();
        let actual = self.actual().attachments().len();

        if actual != expected {
            self.failure(FailureKind::Length)
                .actual(format_args!("{actual:?}"))
                .relation("is not the expected attachment count")
                .expected(format_args!("{expected:?}"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_current_context_type<E: 'static>(self) -> Self {
        self.track_assertion();
        assert_current_context_type::<E, _, _, _>(
            &self,
            self.actual().current_context_type_id(),
            self.actual().current_context_type_name(),
        );
        self
    }

    #[track_caller]
    fn has_current_context_display_value(self, expected: impl Display) -> Self {
        self.track_assertion();
        let actual = format!("{}", self.actual().format_current_context());
        let expected = format!("{expected}");

        if actual != expected {
            self.failure(FailureKind::Equality)
                .actual(format_args!("{actual:?}"))
                .relation("is not the expected current context display value")
                .expected(format_args!("{expected:?}"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self {
        self.track_assertion();
        let actual = format!("{:?}", self.actual().format_current_context());
        let actual = strip_quotation_marks(actual.as_str());
        let expected = strip_quotation_marks(expected.as_ref());

        if actual != expected {
            self.failure(FailureKind::Equality)
                .actual(format_args!("{actual:?}"))
                .relation("is not the expected current context debug string")
                .expected(format_args!("{expected:?}"))
                .raise();
        }
        self
    }
}

/// Assertions over the dynamically typed current context of an owned report.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
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
        self.track_assertion();

        if self.actual().downcast_current_context::<E>().is_some() {
            self.satisfies(
                |report| {
                    report
                        .downcast_current_context::<E>()
                        .expect("context type was checked")
                },
                assertions,
            )
        } else {
            assert_current_context_type::<E, _, _, _>(
                &self,
                self.actual().current_context_type_id(),
                self.actual().current_context_type_name(),
            );
            self
        }
    }
}

/// Assertions over the dynamically typed current context of a report reference.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
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
        self.track_assertion();

        if self.actual().downcast_current_context::<E>().is_some() {
            self.satisfies(
                |report| {
                    report
                        .downcast_current_context::<E>()
                        .expect("context type was checked")
                },
                assertions,
            )
        } else {
            assert_current_context_type::<E, _, _, _>(
                &self,
                self.actual().current_context_type_id(),
                self.actual().current_context_type_name(),
            );
            self
        }
    }
}

/// Panic-mode extraction from a dynamic report reference.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
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
        self.track_assertion();

        if self.actual().downcast_current_context::<E>().is_some() {
            self.derive(|report| {
                report
                    .downcast_current_context::<E>()
                    .expect("context type was checked")
            })
        } else {
            assert_current_context_type::<E, _, _, _>(
                self,
                self.actual().current_context_type_id(),
                self.actual().current_context_type_name(),
            );
            unreachable!("Panic mode always panics on fail")
        }
    }
}

/// Panic-mode extraction from an owned dynamic report.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
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
        self.track_assertion();

        if self.actual().downcast_current_context::<E>().is_some() {
            self.derive(|report| {
                report
                    .downcast_current_context::<E>()
                    .expect("context type was checked")
            })
        } else {
            assert_current_context_type::<E, _, _, _>(
                self,
                self.actual().current_context_type_id(),
                self.actual().current_context_type_name(),
            );
            unreachable!("Panic mode always panics on fail")
        }
    }
}

#[track_caller]
fn assert_current_context_type<E: 'static, T, M: Mode, R>(
    assertion: &AssertThat<'_, T, M, R>,
    actual_type_id: TypeId,
    actual_type_name: &'static str,
) {
    if actual_type_id != TypeId::of::<E>() {
        assertion
            .failure(FailureKind::Variant)
            .actual(format_args!("{actual_type_name}"))
            .relation("is not the expected current context type")
            .expected(format_args!("{}", type_name::<E>()))
            .raise();
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
                    RootcauseReportRefAssertions
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
        fn succeeds_when_debug_string_matches() {
            assert_that!(report!(TestError("root")))
                .has_current_context_debug_string(r#"TestError("root")"#);
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
