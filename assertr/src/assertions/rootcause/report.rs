use crate::assertions::core::strip_quotation_marks;
use crate::{AssertThat, Mode, mode::Panic, tracking::AssertionTracking};
use alloc::format;
use alloc::string::String;
use core::any::{TypeId, type_name};
use core::fmt::{Display, Write};
use indoc::writedoc;
use rootcause::markers::Dynamic;

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RootcauseReportAssertions {
    fn has_child_count(self, expected: usize) -> Self;

    fn has_attachment_count(self, expected: usize) -> Self;

    fn has_current_context_type<E: 'static>(self) -> Self;

    /// Tests the rootcause-formatted `Display` representation of the current context.
    ///
    /// This uses `Report::format_current_context()` instead of formatting a
    /// downcast context value directly, so rootcause context formatter hooks and
    /// preformatted contexts are honored. It also keeps the assertion chain on
    /// the report and does not require callers to name the concrete context type.
    fn has_current_context_display_value(self, expected: impl Display) -> Self;

    /// Tests the rootcause-formatted `Debug` representation of the current context.
    ///
    /// This uses `Report::format_current_context()` instead of formatting a
    /// downcast context value directly, so rootcause context formatter hooks and
    /// preformatted contexts are honored. It also keeps the assertion chain on
    /// the report and does not require callers to name the concrete context type.
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self;
}

impl<C: ?Sized, O, T, M: Mode> RootcauseReportAssertions
    for AssertThat<'_, rootcause::Report<C, O, T>, M>
where
    O: rootcause::markers::ReportOwnershipMarker,
{
    #[track_caller]
    fn has_child_count(self, expected: usize) -> Self {
        self.derive(rootcause::Report::as_ref)
            .has_child_count(expected);
        self
    }

    #[track_caller]
    fn has_attachment_count(self, expected: usize) -> Self {
        self.derive(rootcause::Report::as_ref)
            .has_attachment_count(expected);
        self
    }

    #[track_caller]
    fn has_current_context_type<E: 'static>(self) -> Self {
        self.derive(rootcause::Report::as_ref)
            .has_current_context_type::<E>();
        self
    }

    #[track_caller]
    fn has_current_context_display_value(self, expected: impl Display) -> Self {
        self.derive(rootcause::Report::as_ref)
            .has_current_context_display_value(expected);
        self
    }

    #[track_caller]
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self {
        self.derive(rootcause::Report::as_ref)
            .has_current_context_debug_string(expected);
        self
    }
}

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RootcauseReportRefAssertions {
    fn has_child_count(self, expected: usize) -> Self;

    fn has_attachment_count(self, expected: usize) -> Self;

    fn has_current_context_type<E: 'static>(self) -> Self;

    /// Tests the rootcause-formatted `Display` representation of the current context.
    ///
    /// This uses `ReportRef::format_current_context()` instead of formatting a
    /// downcast context value directly, so rootcause context formatter hooks and
    /// preformatted contexts are honored. It also keeps the assertion chain on
    /// the report reference and does not require callers to name the concrete
    /// context type.
    fn has_current_context_display_value(self, expected: impl Display) -> Self;

    /// Tests the rootcause-formatted `Debug` representation of the current context.
    ///
    /// This uses `ReportRef::format_current_context()` instead of formatting a
    /// downcast context value directly, so rootcause context formatter hooks and
    /// preformatted contexts are honored. It also keeps the assertion chain on
    /// the report reference and does not require callers to name the concrete
    /// context type.
    fn has_current_context_debug_string(self, expected: impl AsRef<str>) -> Self;
}

impl<C: ?Sized, O, T, M: Mode> RootcauseReportRefAssertions
    for AssertThat<'_, rootcause::ReportRef<'_, C, O, T>, M>
{
    #[track_caller]
    fn has_child_count(self, expected: usize) -> Self {
        self.track_assertion();
        let actual = self.actual().children().len();

        if actual != expected {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected child report count: {expected:?}

                      Actual child report count: {actual:?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn has_attachment_count(self, expected: usize) -> Self {
        self.track_assertion();
        let actual = self.actual().attachments().len();

        if actual != expected {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected attachment count: {expected:?}

                      Actual attachment count: {actual:?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn has_current_context_type<E: 'static>(self) -> Self {
        self.track_assertion();
        assert_current_context_type::<E, _, _>(
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
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected current context display: {expected:?}

                      Actual current context display: {actual:?}
                "}
            });
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
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected current context debug: {expected:?}

                      Actual current context debug: {actual:?}
                "}
            });
        }
        self
    }
}

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RootcauseDynamicReportAssertions<'t, M: Mode> {
    fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
    where
        E: 'static,
        A: for<'a> FnOnce(AssertThat<'a, &'a E, M>);
}

impl<'t, O, T, M: Mode> RootcauseDynamicReportAssertions<'t, M>
    for AssertThat<'t, rootcause::Report<Dynamic, O, T>, M>
where
    O: rootcause::markers::ReportOwnershipMarker,
{
    #[track_caller]
    fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
    where
        E: 'static,
        A: for<'a> FnOnce(AssertThat<'a, &'a E, M>),
    {
        self.track_assertion();

        if self.actual().downcast_current_context::<E>().is_some() {
            self.satisfies_ref(
                |report| {
                    report
                        .downcast_current_context::<E>()
                        .expect("context type was checked")
                },
                assertions,
            )
        } else {
            assert_current_context_type::<E, _, _>(
                &self,
                self.actual().current_context_type_id(),
                self.actual().current_context_type_name(),
            );
            self
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RootcauseDynamicReportRefAssertions<'r, M: Mode> {
    fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
    where
        E: 'static,
        A: for<'a> FnOnce(AssertThat<'a, &'r E, M>);
}

impl<'t, 'r, O, T, M: Mode> RootcauseDynamicReportRefAssertions<'r, M>
    for AssertThat<'t, rootcause::ReportRef<'r, Dynamic, O, T>, M>
where
    'r: 't,
{
    #[track_caller]
    fn has_current_context_satisfying<E, A>(self, assertions: A) -> Self
    where
        E: 'static,
        A: for<'a> FnOnce(AssertThat<'a, &'r E, M>),
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
            assert_current_context_type::<E, _, _>(
                &self,
                self.actual().current_context_type_id(),
                self.actual().current_context_type_name(),
            );
            self
        }
    }
}

#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RootcauseDynamicReportRefExtractAssertions<'t, 'r> {
    /// Asserts that this dynamic report reference's current context has type
    /// `E` and returns an assertion context for it.
    ///
    /// This uses rootcause's dynamic current-context downcast and reports type
    /// mismatches through assertr's formatted failure output.
    fn has_current_context<E: 'static>(&'t self) -> AssertThat<'t, &'r E, Panic>;
}

impl<'t, 'r, O, T> RootcauseDynamicReportRefExtractAssertions<'t, 'r>
    for AssertThat<'t, rootcause::ReportRef<'r, Dynamic, O, T>, Panic>
where
    'r: 't,
{
    #[track_caller]
    fn has_current_context<E: 'static>(&'t self) -> AssertThat<'t, &'r E, Panic> {
        self.track_assertion();

        if self.actual().downcast_current_context::<E>().is_some() {
            self.derive(|report| {
                report
                    .downcast_current_context::<E>()
                    .expect("context type was checked")
            })
        } else {
            assert_current_context_type::<E, _, _>(
                self,
                self.actual().current_context_type_id(),
                self.actual().current_context_type_name(),
            );
            unreachable!("Panic mode always panics on fail")
        }
    }
}

#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RootcauseDynamicReportExtractAssertions<'t> {
    /// Asserts that this dynamic report's current context has type `E` and
    /// returns an assertion context for it.
    ///
    /// This uses rootcause's dynamic current-context downcast and reports type
    /// mismatches through assertr's formatted failure output.
    fn has_current_context<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic>;
}

impl<'t, O, T> RootcauseDynamicReportExtractAssertions<'t>
    for AssertThat<'t, rootcause::Report<Dynamic, O, T>, Panic>
{
    #[track_caller]
    fn has_current_context<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic> {
        self.track_assertion();

        if self.actual().downcast_current_context::<E>().is_some() {
            self.derive(|report| {
                report
                    .downcast_current_context::<E>()
                    .expect("context type was checked")
            })
        } else {
            assert_current_context_type::<E, _, _>(
                self,
                self.actual().current_context_type_id(),
                self.actual().current_context_type_name(),
            );
            unreachable!("Panic mode always panics on fail")
        }
    }
}

fn assert_current_context_type<E: 'static, T, M: Mode>(
    assertion: &AssertThat<'_, T, M>,
    actual_type_id: TypeId,
    actual_type_name: &'static str,
) {
    if actual_type_id != TypeId::of::<E>() {
        let expected_type_name = type_name::<E>();

        assertion.fail(|w: &mut String| {
            writedoc! {w, r"
                Expected current context type: {expected_type_name}

                  Actual current context type: {actual_type_name}
            "}
        });
    }
}

#[cfg(test)]
mod tests {
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
                Actual: rootcause::report_collection::owned::limit_field_access::ReportCollection{trailing_space}


                does not have the correct length

                Expected: 1
                  Actual: 0
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
                Expected child report count: 1

                  Actual child report count: 0
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
                Expected attachment count: 1

                  Actual attachment count: 2
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
                Expected current context type: alloc::string::String

                  Actual current context type: {actual_type}
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
                Expected current context display: "other"

                  Actual current context display: "root"
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
                Expected current context debug: "other"

                  Actual current context debug: "TestError(\"root\")"
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
            fn succeeds_when_callback_assertions_pass_in_panic_mode() {
                assert_that!(report!("root")).has_current_context_satisfying::<&'static str, _>(
                    |context| {
                        context.is_equal_to(&"root");
                    },
                );
            }

            #[test]
            fn captures_failures_from_callback_assertions_in_capture_mode() {
                let failures = assert_that!(report!("root"))
                    .with_capture()
                    .with_location(false)
                    .has_current_context_satisfying::<&'static str, _>(|context| {
                        context.is_equal_to(&"other");
                    })
                    .capture_failures();

                assert_that!(failures)
                    .has_length(1)
                    .contains_exactly::<String>([formatdoc! {r#"
                        -------- assertr --------
                        Expected: "other"

                          Actual: "root"
                        -------- assertr --------
                    "#}]);
            }

            #[test]
            fn succeeds_on_report_ref() {
                let report = report!("root");
                let report_ref = report.as_ref();

                assert_that!(report_ref).has_current_context_satisfying::<&'static str, _>(
                    |context| {
                        context.is_equal_to(&"root");
                    },
                );
            }
        }

        mod has_current_context {
            use super::*;

            #[test]
            fn succeeds_when_type_matches() {
                assert_that!(report!("root"))
                    .has_current_context::<&'static str>()
                    .is_equal_to(&"root");
            }

            #[test]
            fn succeeds_on_report_ref() {
                let report = report!("root");
                let report_ref = report.as_ref();

                assert_that!(report_ref)
                    .has_current_context::<&'static str>()
                    .is_equal_to(&"root");
            }

            #[test]
            fn panics_when_type_does_not_match() {
                assert_that_panic_by(|| {
                    assert_that!(report!("root"))
                        .with_location(false)
                        .has_current_context::<String>();
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected current context type: alloc::string::String

                      Actual current context type: &str
                    -------- assertr --------
                "});
            }
        }
    }
}
