use assertr::assertions::collection::CollectionStyle;
use assertr::prelude::*;
use core::task::Poll;

struct NoRenderer;

#[test]
fn poll_assertions_is_implemented_without_poll_renderer_support() {
    fn assert_implemented<'t, T: 't, M: Mode, R>()
    where
        AssertThat<'t, Poll<T>, M, R>: PollAssertions<'t, T, M, R>,
    {
    }

    assert_implemented::<'static, (), Panic, NoRenderer>();
}

#[test]
fn core_assertion_traits_are_implemented_without_renderer_support() {
    fn assert_bool<'t, M: Mode, R>()
    where
        AssertThat<'t, bool, M, R>: BoolAssertions<R>,
    {
    }

    fn assert_char<'t, M: Mode, R>()
    where
        AssertThat<'t, char, M, R>: CharAssertions<R>,
    {
    }

    fn assert_length<'t, T: assertr::assertions::HasLength + 't, M: Mode, R>()
    where
        AssertThat<'t, T, M, R>: LengthAssertions,
    {
    }

    fn assert_str<'t, T: AsRef<str> + 't, M: Mode, R>()
    where
        AssertThat<'t, T, M, R>: StrAssertions,
    {
    }

    fn assert_range_bound<'t, B: 't, T: core::ops::RangeBounds<B> + 't, M: Mode, R>()
    where
        AssertThat<'t, T, M, R>: RangeBoundAssertions<B, T, R>,
    {
    }

    fn assert_range<'t, T: 't, M: Mode, R>()
    where
        AssertThat<'t, T, M, R>: RangeAssertions<T, R>,
    {
    }

    #[cfg(feature = "std")]
    fn assert_fn_once<'t, F: 't, O, R>()
    where
        AssertThat<'t, F, Panic, R>: FnOnceAssertions<'t, O, R>,
    {
    }

    #[cfg(feature = "std")]
    fn assert_async_fn_once<'t, F: 't, O, R>()
    where
        AssertThat<'t, F, Panic, R>: AsyncFnOnceAssertions<'t, O, R>,
    {
    }

    #[cfg(feature = "std")]
    fn assert_panic_value<'t, R>()
    where
        AssertThat<'t, assertr::PanicValue, Panic, R>: PanicValueAssertions<'t, R>,
    {
    }

    #[cfg(feature = "std")]
    fn assert_path<'t, T: std::ops::Deref<Target = std::path::Path> + 't, M: Mode, R>()
    where
        AssertThat<'t, T, M, R>: PathAssertions<Subject = T, Renderer = R>,
    {
    }

    assert_bool::<'static, Panic, NoRenderer>();
    assert_char::<'static, Panic, NoRenderer>();
    assert_length::<'static, Vec<u8>, Panic, NoRenderer>();
    assert_str::<'static, String, Panic, NoRenderer>();
    assert_range_bound::<'static, i32, core::ops::Range<i32>, Panic, NoRenderer>();
    assert_range::<'static, i32, Panic, NoRenderer>();
    #[cfg(feature = "std")]
    assert_fn_once::<'static, fn() -> (), (), NoRenderer>();
    #[cfg(feature = "std")]
    assert_async_fn_once::<'static, fn() -> core::future::Ready<()>, (), NoRenderer>();
    #[cfg(feature = "std")]
    assert_panic_value::<'static, NoRenderer>();
    #[cfg(feature = "std")]
    assert_path::<'static, std::path::PathBuf, Panic, NoRenderer>();
}

#[test]
#[cfg(feature = "num")]
fn numeric_assertions_is_implemented_without_renderer_support() {
    fn assert_implemented<'t, T: num_traits::Num + 't, M: Mode, R>()
    where
        AssertThat<'t, T, M, R>: NumAssertions<T>,
    {
    }

    assert_implemented::<'static, i32, Panic, NoRenderer>();
}

#[test]
#[cfg(feature = "http")]
fn header_value_assertion_traits_are_implemented_without_renderer_support() {
    fn assert_checking<'t, M: Mode, R>()
    where
        AssertThat<'t, http::HeaderValue, M, R>: HttpHeaderValueAssertions<'t, M, R>,
    {
    }

    fn assert_extracting<'t, R>()
    where
        AssertThat<'t, http::HeaderValue, Panic, R>: HttpHeaderValueExtractAssertions<'t, R>,
    {
    }

    assert_checking::<'static, Panic, NoRenderer>();
    assert_extracting::<'static, NoRenderer>();
}

#[test]
#[cfg(feature = "jiff")]
fn jiff_assertion_traits_are_implemented_without_renderer_support() {
    fn assert_signed_duration<'t, M: Mode, R>()
    where
        AssertThat<'t, jiff::SignedDuration, M, R>: SignedDurationAssertions<R>,
    {
    }

    fn assert_span<'t, M: Mode, R>()
    where
        AssertThat<'t, jiff::Span, M, R>: SpanAssertions<R>,
    {
    }

    fn assert_zoned<'t, M: Mode, R>()
    where
        AssertThat<'t, jiff::Zoned, M, R>: ZonedAssertions<R>,
    {
    }

    assert_signed_duration::<'static, Panic, NoRenderer>();
    assert_span::<'static, Panic, NoRenderer>();
    assert_zoned::<'static, Panic, NoRenderer>();
}

#[test]
#[cfg(feature = "program")]
fn program_assertion_traits_are_implemented_without_renderer_support() {
    fn assert_checking<'t, 'a: 't, M: Mode, R>()
    where
        AssertThat<'t, assertr::assertions::program::Program<'a>, M, R>:
            ProgramAssertions<'t, 'a, M, R>,
    {
    }

    fn assert_extracting<'t, R>()
    where
        AssertThat<'t, assertr::assertions::program::Program<'static>, Panic, R>:
            ProgramAssertionsRequiringPanicMode<'t, R>,
    {
    }

    assert_checking::<'static, 'static, Panic, NoRenderer>();
    assert_extracting::<'static, NoRenderer>();
}

#[test]
#[cfg(feature = "std")]
fn command_assertions_is_implemented_without_renderer_support() {
    fn assert_implemented<'t, M: Mode, R>()
    where
        AssertThat<'t, std::process::Command, M, R>: CommandAssertions<R>,
    {
    }

    assert_implemented::<'static, Panic, NoRenderer>();
}

#[test]
#[cfg(feature = "reqwest")]
fn reqwest_extract_assertions_is_implemented_without_body_result_renderer_support() {
    use reqwest::ResponseBuilderExt as _;

    fn assert_implemented<'t, R>()
    where
        AssertThat<'t, reqwest::Response, Panic, R>: ReqwestResponseExtractAssertions<'t, R>,
    {
    }

    assert_implemented::<'static, NoRenderer>();

    let response = reqwest::Response::from(
        http::Response::builder()
            .url("http://localhost/text".parse().expect("valid url"))
            .body("text")
            .expect("valid response"),
    );
    let future = assert_that_owned!(response)
        .with_debug_format(|_, f| f.write_str("response"))
        .get_text();
    drop(future);

    #[cfg(feature = "serde-json")]
    {
        struct StringRenderer;

        impl ValueRenderer<String> for StringRenderer {
            fn fmt(&self, value: &String, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Debug::fmt(value, f)
            }
        }

        let response = reqwest::Response::from(
            http::Response::builder()
                .url("http://localhost/json".parse().expect("valid url"))
                .body("null")
                .expect("valid response"),
        );
        let future = assert_that_owned!(response)
            .with_renderer(StringRenderer)
            .get_json::<serde_json::Value>();
        drop(future);
    }
}

#[test]
#[cfg(feature = "tokio")]
fn tokio_watch_assertion_traits_are_implemented_without_renderer_support() {
    fn assert_implemented<'t, T: 't, R>()
    where
        AssertThat<'t, tokio::sync::watch::Receiver<T>, Panic, R>:
            TokioWatchReceiverAssertions<T, R> + TokioWatchReceiverExtractAssertions<T, R>,
    {
    }

    assert_implemented::<'static, (), NoRenderer>();
}

#[test]
#[cfg(feature = "rootcause")]
fn rootcause_assertion_traits_are_implemented_without_renderer_support() {
    use rootcause::markers::Dynamic;
    use rootcause::prelude::*;

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

#[derive(Clone, Copy)]
struct AuditedRenderer;

impl ValueRenderer<bool> for AuditedRenderer {
    fn fmt(&self, value: &bool, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BOOL({value})")
    }
}

impl ValueRenderer<char> for AuditedRenderer {
    fn fmt(&self, value: &char, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CHAR({value})")
    }
}

impl ValueRenderer<i32> for AuditedRenderer {
    fn fmt(&self, value: &i32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "I32({value})")
    }
}

#[cfg(feature = "std")]
impl ValueRenderer<std::path::PathBuf> for AuditedRenderer {
    fn fmt(
        &self,
        value: &std::path::PathBuf,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "PATH({})", value.display())
    }
}

#[cfg(feature = "http")]
impl ValueRenderer<http::HeaderValue> for AuditedRenderer {
    fn fmt(
        &self,
        _value: &http::HeaderValue,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str("HEADER_VALUE")
    }
}

#[cfg(feature = "jiff")]
impl ValueRenderer<jiff::SignedDuration> for AuditedRenderer {
    fn fmt(
        &self,
        _value: &jiff::SignedDuration,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str("SIGNED_DURATION")
    }
}

#[cfg(feature = "jiff")]
impl ValueRenderer<jiff::Span> for AuditedRenderer {
    fn fmt(&self, _value: &jiff::Span, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SPAN")
    }
}

#[cfg(feature = "jiff")]
impl ValueRenderer<jiff::Zoned> for AuditedRenderer {
    fn fmt(&self, _value: &jiff::Zoned, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ZONED")
    }
}

#[cfg(feature = "program")]
impl ValueRenderer<assertr::assertions::program::Program<'_>> for AuditedRenderer {
    fn fmt(
        &self,
        _value: &assertr::assertions::program::Program<'_>,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str("PROGRAM")
    }
}

#[cfg(feature = "tokio")]
impl ValueRenderer<tokio::sync::watch::error::RecvError> for AuditedRenderer {
    fn fmt(
        &self,
        value: &tokio::sync::watch::error::RecvError,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "WATCH_ERROR({value:?})")
    }
}

fn assert_single_failure_contains(failures: &[AssertionFailure], expected: &str) {
    assert_that!(failures).has_length(1);
    assert_that!(failures[0].to_string()).contains(expected);
}

#[test]
fn render_values_honors_the_required_collection_style() {
    let values = [&1, &2];
    let assertion = assert_that!(0).with_renderer(AuditedRenderer);

    let list = format!(
        "{:?}",
        assertion.render_values(&values, CollectionStyle::List)
    );
    let set = format!(
        "{:?}",
        assertion.render_values(&values, CollectionStyle::Set)
    );

    assert_that!(list).is_equal_to("[I32(1), I32(2)]");
    assert_that!(set).is_equal_to("{I32(1), I32(2)}");
}

#[test]
fn bool_and_char_assertions_use_the_active_renderer() {
    let bool_failures = assert_that!(false)
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(BoolAssertions::is_true);
    assert_single_failure_contains(&bool_failures, "BOOL(false)");

    let char_failures = assert_that!('A')
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(CharAssertions::is_lowercase);
    assert_single_failure_contains(&char_failures, "CHAR(A)");
}

#[test]
fn range_assertions_render_the_structure_from_rendered_bounds() {
    let bound_failures = assert_that!(1..3)
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(|it| {
            it.contains_element(4);
            it
        });
    assert_single_failure_contains(&bound_failures, "I32(1)..I32(3)");
    assert_that!(bound_failures[0].to_string()).contains("I32(4)");

    let value_failures = assert_that!(1)
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(|it| it.is_in_range(2..=3));
    assert_single_failure_contains(&value_failures, "I32(1)");
    assert_that!(value_failures[0].to_string()).contains("I32(2)..=I32(3)");
}

#[test]
#[cfg(feature = "std")]
fn path_assertions_use_the_active_renderer() {
    let failures = assert_that!(std::path::PathBuf::from(
        "assertr-renderer-test-path-that-does-not-exist",
    ))
    .with_renderer(AuditedRenderer)
    .with_location(false)
    .capture(PathAssertions::exists);

    assert_single_failure_contains(
        &failures,
        "PATH(assertr-renderer-test-path-that-does-not-exist)",
    );
}

#[test]
#[cfg(feature = "http")]
fn header_ascii_assertions_use_the_active_renderer() {
    let value = http::HeaderValue::from_bytes(b"\xFF").expect("valid opaque header bytes");
    let failures = assert_that!(value)
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(|it| it.is_ascii().is_ascii_satisfying(|_| {}));

    assert_that!(&failures).has_length(2);
    assert_that!(failures[0].to_string()).contains("HEADER_VALUE");
    assert_that!(failures[1].to_string()).contains("HEADER_VALUE");

    #[cfg(feature = "std")]
    {
        assert_that_panic_by(|| {
            let value = http::HeaderValue::from_bytes(b"\xFF").expect("valid opaque header bytes");
            assert_that!(value)
                .with_renderer(AuditedRenderer)
                .with_location(false)
                .get_ascii();
        })
        .has_type::<String>()
        .contains("HEADER_VALUE");
    }
}

#[test]
#[cfg(feature = "http")]
fn header_boolean_projections_use_the_active_renderer() {
    let value = http::HeaderValue::from_static("visible");
    let failures = assert_that!(value)
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(HttpHeaderValueAssertions::is_sensitive);

    assert_single_failure_contains(&failures, "BOOL(false)");
}

#[test]
#[cfg(feature = "jiff")]
fn jiff_assertions_use_the_active_renderer() {
    let duration_failures = assert_that!(jiff::SignedDuration::from_secs(1))
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(SignedDurationAssertions::is_zero);
    assert_single_failure_contains(&duration_failures, "SIGNED_DURATION");

    let span_failures = assert_that!(jiff::Span::new().hours(1))
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(SpanAssertions::is_zero);
    assert_single_failure_contains(&span_failures, "SPAN");

    let zoned: jiff::Zoned = "2024-06-19 15:22[America/New_York]"
        .parse()
        .expect("valid zoned datetime");
    let zoned_failures = assert_that!(zoned)
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(|it| it.is_in_time_zone_named("Europe/Berlin"));
    assert_single_failure_contains(&zoned_failures, "ZONED");
}

#[test]
#[cfg(feature = "program")]
fn program_assertions_use_the_active_renderer() {
    const MISSING_PROGRAM: &str = "assertr-renderer-test-program-that-does-not-exist";

    let failures = assert_that!(assertr::assertions::program::Program::from(MISSING_PROGRAM))
        .with_renderer(AuditedRenderer)
        .with_location(false)
        .capture(ProgramAssertions::exists);

    assert_single_failure_contains(&failures, "PROGRAM");

    assert_that_panic_by(|| {
        assert_that!(assertr::assertions::program::Program::from(MISSING_PROGRAM,))
            .with_renderer(AuditedRenderer)
            .with_location(false)
            .exists_and();
    })
    .has_type::<String>()
    .contains("PROGRAM");
}

#[test]
#[cfg(feature = "tokio")]
fn tokio_watch_boolean_projection_uses_the_active_renderer() {
    let (_sender, receiver) = tokio::sync::watch::channel(());
    assert_that_panic_by(|| {
        assert_that!(receiver)
            .with_renderer(AuditedRenderer)
            .with_location(false)
            .has_changed();
    })
    .has_type::<String>()
    .contains("BOOL(false)");
}

#[derive(PartialEq)]
struct Secret(u32);

#[test]
fn non_debug_type_can_use_debug_format_closure() {
    let failures = assert_that!(Secret(1))
        .with_debug_format(|value, f| f.write_fmt(format_args!("Secret({})", value.0)))
        .with_location(false)
        .capture(|it| it.is_equal_to(Secret(2)));

    assert_that!(failures.as_slice()).contains_exactly_satisfying([
        |it: AssertThat<AssertionFailure, Capture>| {
            it.has_display_value(indoc::formatdoc! {"
                -------- assertr --------
                Expression: `Secret(1)`

                Expected: Secret(2)

                  Actual: Secret(1)
                -------- assertr --------
            "});
        },
    ]);
}

#[test]
fn debug_format_closure_works_in_derived_chain() {
    // Regression: `with_debug_format` produces `CustomRenderer<F>`, which must be `Clone`
    // for any assertion that derives a child `AssertThat` (here, `derive_owned`).
    assert_that!(Secret(7))
        .with_debug_format(|value: &Secret, f| f.write_fmt(format_args!("Secret({})", value.0)))
        .derive_owned(|s| Secret(s.0))
        .is_equal_to(Secret(7));
}

#[derive(Clone, Copy)]
struct SecretAndU32Renderer;

impl ValueRenderer<Secret> for SecretAndU32Renderer {
    fn fmt(&self, value: &Secret, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Secret({})", value.0)
    }
}

impl ValueRenderer<u32> for SecretAndU32Renderer {
    fn fmt(&self, value: &u32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "u32({value})")
    }
}

#[test]
fn custom_renderer_threads_through_satisfies() {
    // `satisfies` requires `R: Clone` and propagates the renderer to the child `AssertThat`.
    // The custom renderer must therefore stay live for the inner closure's assertions.
    assert_that!(Secret(7))
        .with_renderer(SecretAndU32Renderer)
        .satisfies(
            |s| &s.0,
            |inner| {
                inner.is_equal_to(7u32);
            },
        );
}

#[test]
fn custom_renderer_renders_failures_inside_satisfies() {
    let failures = assert_that!(Secret(1))
        .with_renderer(SecretAndU32Renderer)
        .with_location(false)
        .capture(|it| {
            it.satisfies(
                |s| &s.0,
                |inner| {
                    inner.is_equal_to(2u32);
                },
            )
        });

    assert_that!(failures.as_slice()).contains_exactly_satisfying([
        |it: AssertThat<AssertionFailure, Capture>| {
            it.has_display_value(indoc::formatdoc! {"
                -------- assertr --------
                Expected: u32(2)

                  Actual: u32(1)
                -------- assertr --------
            "});
        },
    ]);
}

struct Actual(u32);
struct Expected(u32);

impl PartialEq<Expected> for Actual {
    fn eq(&self, other: &Expected) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Copy)]
struct NamedRenderer;

impl ValueRenderer<Actual> for NamedRenderer {
    fn fmt(&self, value: &Actual, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Actual({})", value.0))
    }
}

impl ValueRenderer<Expected> for NamedRenderer {
    fn fmt(&self, value: &Expected, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Expected({})", value.0))
    }
}

#[test]
fn named_renderer_can_render_heterogeneous_comparisons() {
    let failures = assert_that!(Actual(1))
        .with_renderer(NamedRenderer)
        .with_location(false)
        .capture(|it| it.is_equal_to(Expected(2)));

    assert_that!(failures.as_slice()).contains_exactly_satisfying([
        |it: AssertThat<AssertionFailure, Capture>| {
            it.has_display_value(indoc::formatdoc! {"
                -------- assertr --------
                Expression: `Actual(1)`

                Expected: Expected(2)
        
                  Actual: Actual(1)
                -------- assertr --------
            "});
        },
    ]);
}

mod collection_renderer_equality {
    use super::*;
    use assertr::{AssertrPartialEq, EqContext};
    use std::collections::VecDeque;

    #[derive(Clone, Copy)]
    struct CollectionActual(u32);

    #[derive(Clone, Copy)]
    struct CollectionExpected(u32);

    #[derive(Clone, Copy)]
    struct CollectionRenderer;

    impl AssertrPartialEq<CollectionExpected, CollectionRenderer> for CollectionActual {
        fn eq(
            &self,
            other: &CollectionExpected,
            _ctx: Option<&mut EqContext<'_, CollectionRenderer>>,
        ) -> bool {
            self.0 == other.0
        }
    }

    impl ValueRenderer<CollectionActual> for CollectionRenderer {
        fn fmt(
            &self,
            value: &CollectionActual,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            f.write_fmt(format_args!("Actual({})", value.0))
        }
    }

    impl ValueRenderer<CollectionExpected> for CollectionRenderer {
        fn fmt(
            &self,
            value: &CollectionExpected,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            f.write_fmt(format_args!("Expected({})", value.0))
        }
    }

    fn is_actual_two(assertion: AssertThat<CollectionActual, Capture, CollectionRenderer>) {
        assertion.is_equal_to(CollectionExpected(2));
    }

    #[test]
    fn slice_membership_uses_renderer_specific_equality() {
        let actual = [CollectionActual(1), CollectionActual(2)];

        assert_that!(actual.as_slice())
            .with_renderer(CollectionRenderer)
            .contains(CollectionExpected(2));
    }

    #[test]
    fn exact_slice_comparison_uses_renderer_specific_equality() {
        let actual = [CollectionActual(1), CollectionActual(2)];

        assert_that!(actual.as_slice())
            .with_renderer(CollectionRenderer)
            .contains_exactly([CollectionExpected(1), CollectionExpected(2)]);
    }

    #[test]
    fn iterator_membership_uses_renderer_specific_equality() {
        assert_that_owned!(vec![CollectionActual(1), CollectionActual(2)].into_iter())
            .with_renderer(CollectionRenderer)
            .contains(CollectionExpected(2));
        assert_that_owned!(vec![CollectionActual(1), CollectionActual(2)].into_iter())
            .with_renderer(CollectionRenderer)
            .contains_exactly([CollectionExpected(1), CollectionExpected(2)]);
    }

    #[test]
    fn iterator_sequence_assertions_use_renderer_specific_equality() {
        assert_that_owned!(
            vec![
                CollectionActual(1),
                CollectionActual(2),
                CollectionActual(3)
            ]
            .into_iter()
        )
        .with_renderer(CollectionRenderer)
        .contains_contiguous([CollectionExpected(2), CollectionExpected(3)]);

        assert_that_owned!(
            vec![
                CollectionActual(1),
                CollectionActual(2),
                CollectionActual(3)
            ]
            .into_iter()
        )
        .with_renderer(CollectionRenderer)
        .starts_with([CollectionExpected(1), CollectionExpected(2)]);

        assert_that_owned!(
            vec![
                CollectionActual(1),
                CollectionActual(2),
                CollectionActual(3)
            ]
            .into_iter()
        )
        .with_renderer(CollectionRenderer)
        .ends_with([CollectionExpected(2), CollectionExpected(3)]);

        assert_that_owned!(vec![CollectionActual(1), CollectionActual(2)].into_iter())
            .with_renderer(CollectionRenderer)
            .contains_contiguous_satisfying([is_actual_two]);
    }

    #[test]
    fn into_iterator_membership_uses_renderer_specific_equality() {
        assert_that!(vec![CollectionActual(1), CollectionActual(2)])
            .with_renderer(CollectionRenderer)
            .into_iter_contains(CollectionExpected(2))
            .into_iter_contains_all([CollectionExpected(2), CollectionExpected(1)]);
    }

    #[test]
    fn borrowed_iterator_sequence_assertions_use_renderer_specific_equality() {
        assert_that!(vec![
            CollectionActual(1),
            CollectionActual(2),
            CollectionActual(3)
        ])
        .with_renderer(CollectionRenderer)
        .into_iter_starts_with([CollectionExpected(1)])
        .into_iter_contains_contiguous([CollectionExpected(2), CollectionExpected(3)])
        .into_iter_ends_with([CollectionExpected(3)]);
    }

    #[test]
    fn vec_deque_comparison_uses_renderer_specific_equality() {
        assert_that!(VecDeque::from([CollectionActual(1), CollectionActual(2),]))
            .with_renderer(CollectionRenderer)
            .contains(CollectionExpected(2));
    }
}

mod wrapper_renderer {
    use super::*;
    use std::cell::RefCell;
    #[cfg(feature = "std")]
    use std::collections::{HashMap, HashSet};
    #[cfg(feature = "std")]
    use std::ffi::OsStr;
    #[cfg(feature = "std")]
    use std::sync::Mutex;

    #[derive(PartialEq, Eq, Hash)]
    struct Secret(u32);

    #[derive(Clone, Copy)]
    struct SecretRenderer;

    impl ValueRenderer<Secret> for SecretRenderer {
        fn fmt(&self, value: &Secret, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("Secret({})", value.0))
        }
    }

    impl ValueRenderer<&'static str> for SecretRenderer {
        fn fmt(&self, value: &&'static str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("{value:?}"))
        }
    }

    impl ValueRenderer<str> for SecretRenderer {
        fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("{value:?}"))
        }
    }

    #[cfg(feature = "std")]
    impl ValueRenderer<OsStr> for SecretRenderer {
        fn fmt(&self, value: &OsStr, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Debug::fmt(value, f)
        }
    }

    #[test]
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn ref_cell_assertion_uses_custom_renderer() {
        let cell = RefCell::new(Secret(7));
        let failures = assert_that!(&cell)
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| it.is_borrowed());

        assert_that!(failures.as_slice()).contains_exactly_satisfying([
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_display_value(indoc::formatdoc! {"
                    -------- assertr --------
                    Expression: `&cell`

                    Actual: RefCell {{
                        value: Secret(7),
                    }} is not borrowed.

                    Expected: RefCell to have an active borrow.
                    -------- assertr --------
                "});
            },
        ]);
    }

    #[test]
    fn enum_wrappers_are_rendered_from_their_leaf_values() {
        let option_failures = assert_that!(Some(Secret(7)))
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(OptionAssertions::is_none);
        assert_single_failure_contains(&option_failures, "Some(\n    Secret(7),\n)");

        let result_failures = assert_that!(Result::<(), Secret>::Err(Secret(8)))
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(ResultAssertions::is_ok);
        assert_single_failure_contains(&result_failures, "Err(\n    Secret(8),\n)");

        let poll_failures = assert_that!(Poll::Ready(Secret(9)))
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(PollAssertions::is_pending);
        assert_single_failure_contains(&poll_failures, "Ready(\n    Secret(9),\n)");
    }

    #[test]
    fn valueless_variants_need_no_renderer_capability() {
        let failures = assert_that!(Option::<Secret>::None)
            .with_renderer(NoRenderer)
            .with_location(false)
            .capture(OptionAssertions::is_some);

        assert_single_failure_contains(&failures, "Actual: None");
    }

    #[test]
    #[cfg(feature = "std")]
    fn hashset_assertion_uses_custom_renderer() {
        let actual: HashSet<Secret> = HashSet::from([Secret(1)]);
        let failures = assert_that!(actual)
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| it.contains(Secret(2)));

        assert_that!(failures.as_slice()).contains_exactly_satisfying([
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_display_value(indoc::formatdoc! {"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: HashSet {{
                        Secret(1),
                    }}

                    does not contain expected: Secret(2)
                    -------- assertr --------
                "});
            },
        ]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hashmap_contains_value_uses_custom_renderer() {
        let mut map: HashMap<&'static str, Secret> = HashMap::new();
        map.insert("alpha", Secret(1));

        let failures = assert_that!(map)
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| it.contains_value(Secret(2)));

        assert_that!(failures.as_slice()).contains_exactly_satisfying([
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_display_value(indoc::formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "alpha": Secret(1),
                    }}

                    does not contain expected value: Secret(2)
                    -------- assertr --------
                "#});
            },
        ]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hashmap_satisfying_assertions_use_custom_renderer() {
        fn is_secret_two(it: AssertThat<Secret, Capture, SecretRenderer>) {
            it.is_equal_to(Secret(2));
        }

        let map = HashMap::from([("alpha", Secret(1))]);
        let failures = assert_that!(map)
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| {
                it.contains_entry_satisfying("alpha", is_secret_two)
                    .contains_exactly_entries_satisfying([("alpha", is_secret_two)])
            });

        assert_that!(failures.as_slice()).contains_exactly_satisfying([
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_display_value(indoc::formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "alpha": Secret(1),
                    }}

                    does not contain an entry satisfying the assertions at key: "alpha"

                    Details: [
                        Value at key "alpha" does not satisfy the assertions:
                        Expected: Secret(2)
                    {nested_padding}
                          Actual: Secret(1),
                    ]
                    -------- assertr --------
                "#, nested_padding = "    "});
            },
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_display_value(indoc::formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "alpha": Secret(1),
                    }}

                    does not exactly contain entries satisfying the assertions

                    Expected keys: [
                        "alpha",
                    ]

                    Details: [
                        Value at key "alpha" does not satisfy its assertions:
                        Expected: Secret(2)
                    {nested_padding}
                          Actual: Secret(1),
                    ]
                    -------- assertr --------
                "#, nested_padding = "    "});
            },
        ]);
    }

    #[test]
    #[cfg(feature = "std")]
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn mutex_is_locked_uses_custom_renderer() {
        let mutex = Mutex::new(Secret(11));
        let failures = assert_that!(mutex)
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| it.is_locked());

        assert_that!(failures.as_slice()).contains_exactly_satisfying([
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_display_value(indoc::formatdoc! {"
                    -------- assertr --------
                    Expression: `mutex`

                    Expected: Mutex {{ data: Secret(11), poisoned: false }}

                    to be locked, but it wasn't!
                    -------- assertr --------
                "});
            },
        ]);
    }

    #[test]
    #[cfg(feature = "tokio")]
    fn rw_lock_is_rendered_from_its_value_renderer() {
        let failures = assert_that!(tokio::sync::RwLock::new(Secret(12)))
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(TokioRwLockAssertions::is_read_locked);

        assert_single_failure_contains(&failures, "RwLock { data: Secret(12) }");
    }

    #[test]
    #[cfg(feature = "std")]
    fn command_arguments_are_rendered_from_the_os_str_renderer() {
        let mut command = std::process::Command::new("program");
        command.arg("--actual");

        let failures = assert_that!(command)
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| it.has_arg("--expected"));

        assert_single_failure_contains(&failures, "\"--actual\"");
    }
}

#[cfg(feature = "derive")]
mod derive {
    use super::*;
    // Only the `MapParent` fixture below uses it, and that one needs assertr's `std` feature for
    // `assertr::cmp::hashmap`.
    #[cfg(feature = "std")]
    use std::collections::HashMap;

    #[derive(PartialEq)]
    pub struct Hidden(u32);

    #[derive(AssertrEq)]
    pub struct Subject {
        pub hidden: Hidden,
    }

    impl ValueRenderer<Subject> for NamedRenderer {
        fn fmt(&self, value: &Subject, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("Subject({})", value.hidden.0))
        }
    }

    impl ValueRenderer<SubjectAssertrEq> for NamedRenderer {
        fn fmt(
            &self,
            _value: &SubjectAssertrEq,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            f.write_str("SubjectAssertrEq(..)")
        }
    }

    impl ValueRenderer<Hidden> for NamedRenderer {
        fn fmt(&self, value: &Hidden, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("Hidden({})", value.0))
        }
    }

    #[test]
    fn derive_reports_differences_for_non_debug_fields_with_renderer() {
        let failures = assert_that!(Subject { hidden: Hidden(1) })
            .with_renderer(NamedRenderer)
            .with_location(false)
            .capture(|it| {
                it.is_equal_to(SubjectAssertrEq {
                    hidden: eq(Hidden(2)),
                })
            });

        assert_that!(failures.as_slice()).contains_exactly_satisfying([
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_display_value(indoc::formatdoc! {"
                    -------- assertr --------
                    Expression: `Subject {{ hidden: Hidden(1) }}`

                    Expected: SubjectAssertrEq(..)
            
                      Actual: Subject(1)

                    Details: [
                        Differences: [
                            \"hidden\": expected Hidden(2), but was Hidden(1),
                        ],
                    ]
                    -------- assertr --------
                "});
            },
        ]);
    }

    #[derive(Debug, AssertrEq)]
    pub struct Child {
        pub id: i32,
    }

    #[derive(Debug, AssertrEq)]
    pub struct NestedParent {
        #[assertr_eq(map_type = "ChildAssertrEq")]
        pub child: Child,
    }

    #[test]
    fn debug_renderer_renders_nested_generated_matchers() {
        let failures = assert_that!(NestedParent {
            child: Child { id: 1 },
        })
        .with_location(false)
        .capture(|it| {
            it.is_equal_to(NestedParentAssertrEq {
                child: eq(ChildAssertrEq { id: eq(2) }),
            })
        });

        assert_that!(failures[0].to_string().as_str()).contains(indoc::indoc! {r"
            Expected: NestedParentAssertrEq {
                child: Eq::Eq(ChildAssertrEq {
                    id: Eq::Eq(2),
                }),
            }
        "});
    }

    #[derive(Debug, AssertrEq)]
    pub struct VecParent {
        #[assertr_eq(
            map_type = "Vec<ChildAssertrEq>",
            compare_with = "::assertr::cmp::slice::compare",
            compare_bounds = "Child: ::assertr::cmp::slice::CompareElement<ChildAssertrEq, R>"
        )]
        pub children: Vec<Child>,
    }

    #[test]
    fn debug_renderer_renders_vec_of_generated_matchers() {
        let failures = assert_that!(VecParent {
            children: vec![Child { id: 1 }],
        })
        .with_location(false)
        .capture(|it| {
            it.is_equal_to(VecParentAssertrEq {
                children: eq(vec![ChildAssertrEq { id: eq(2) }]),
            })
        });

        assert_that!(failures[0].to_string().as_str()).contains(indoc::indoc! {r"
            Expected: VecParentAssertrEq {
                children: Eq::Eq([
                    ChildAssertrEq {
                        id: Eq::Eq(2),
                    },
                ]),
            }
        "});
    }

    #[cfg(feature = "std")]
    #[derive(Debug, AssertrEq)]
    pub struct MapParent {
        #[assertr_eq(
            map_type = "HashMap<String, ChildAssertrEq>",
            compare_with = "::assertr::cmp::hashmap::compare",
            compare_bounds = "Child: ::assertr::cmp::hashmap::CompareValue<ChildAssertrEq, R>"
        )]
        pub children: HashMap<String, Child>,
    }

    #[test]
    #[cfg(feature = "std")]
    fn debug_renderer_renders_hashmap_of_generated_matchers() {
        let failures = assert_that!(MapParent {
            children: HashMap::from([("first".to_string(), Child { id: 1 })]),
        })
        .with_location(false)
        .capture(|it| {
            it.is_equal_to(MapParentAssertrEq {
                children: eq(HashMap::from([(
                    "first".to_string(),
                    ChildAssertrEq { id: eq(2) },
                )])),
            })
        });

        assert_that!(failures[0].to_string().as_str()).contains(indoc::indoc! {r#"
            Expected: MapParentAssertrEq {
                children: Eq::Eq({
                    "first": ChildAssertrEq {
                        id: Eq::Eq(2),
                    },
                }),
            }
        "#});
    }
}

/// The reqwest response assertions never render their subject: they report the status, the header
/// names, or a header value, all of which are domain values with an obvious textual form. They are
/// still generic over the renderer, so a subject carrying a custom one keeps working.
#[cfg(feature = "reqwest")]
mod reqwest_response_renderer {
    use super::*;
    use reqwest::ResponseBuilderExt;

    struct StatusOnly;

    impl ValueRenderer<reqwest::Response> for StatusOnly {
        fn fmt(
            &self,
            value: &reqwest::Response,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            f.write_fmt(format_args!("<{}>", value.status().as_u16()))
        }
    }

    fn response(status: u16) -> reqwest::Response {
        let builder = http::Response::builder()
            .status(status)
            .url("http://localhost/hello".parse().expect("valid url"))
            .header("content-type", "text/plain");

        reqwest::Response::from(builder.body("").expect("valid response"))
    }

    #[test]
    fn assertions_accept_a_subject_with_a_custom_renderer() {
        assert_that!(response(200))
            .with_renderer(StatusOnly)
            .has_status_code(reqwest::StatusCode::OK)
            .is_success()
            .has_header("content-type")
            .does_not_have_header("x-api-key")
            .has_header_value("content-type", "text/plain");

        assert_that!(response(100))
            .with_renderer(StatusOnly)
            .is_informational();
        assert_that!(response(301))
            .with_renderer(StatusOnly)
            .is_redirection();
        assert_that!(response(404))
            .with_renderer(StatusOnly)
            .is_client_error();
        assert_that!(response(500))
            .with_renderer(StatusOnly)
            .is_server_error();
    }
}
