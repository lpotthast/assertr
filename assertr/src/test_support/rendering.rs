//! Renderers and structured evidence checks shared by diagnostic tests.

use crate::{
    AssertionFailure, ValueRenderer, failure::adapter::ToHumanReadableText, renderer::Rendered,
};
use core::fmt;

pub(crate) fn rendered_text(value: &Rendered) -> alloc::string::String {
    let mut text = alloc::string::String::new();
    value
        .write(&mut text, true)
        .expect("writing a rendered value to a String cannot fail");
    text
}

pub(crate) struct NoRenderer;

pub(crate) const SENTINEL: &str = "<rendered>";

#[derive(Clone, Copy)]
pub(crate) struct SentinelRenderer;

impl<T: ?Sized> ValueRenderer<T> for SentinelRenderer {
    fn fmt(&self, _value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(SENTINEL)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RendererActual(pub(crate) u32);

#[derive(Clone, Copy)]
pub(crate) struct RendererExpected(pub(crate) u32);

impl PartialEq<RendererExpected> for RendererActual {
    fn eq(&self, other: &RendererExpected) -> bool {
        self.0 == other.0
    }
}

/// Only renders numeric evidence, never referenced identity targets.
pub(crate) struct NumericRenderer;
impl ValueRenderer<usize> for NumericRenderer {
    fn fmt(&self, value: &usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(value, f)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CustomValueRenderer;
impl<T: fmt::Debug + ?Sized> ValueRenderer<T> for CustomValueRenderer {
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "custom({value:?})")
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RedactingRenderer;
impl<T: ?Sized> ValueRenderer<T> for RedactingRenderer {
    fn fmt(&self, _: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

pub(crate) fn assert_custom_value<T: fmt::Debug + ?Sized>(rendered: &Rendered, value: &T) {
    assert_eq!(rendered.type_name, Some(core::any::type_name::<T>()));
    assert_eq!(rendered_text(rendered), alloc::format!("custom({value:?})"));
}

pub(crate) fn assert_redacted(failure: &AssertionFailure, secrets: &[&str]) {
    fn check_tree(failure: &AssertionFailure, secrets: &[&str]) {
        // AssertionFailure's own Debug prints a report. Inspect its Rendered fields directly,
        // including evidence held in constraints and paths, then recurse into every child.
        let tree = alloc::format!(
            "{:?} {:?} {:?} {:?} {:?} {:?}",
            failure.actual,
            failure.expected,
            failure.unexpected,
            failure.facts,
            failure.constraint,
            failure.path,
        );
        for secret in secrets {
            assert!(!tree.contains(secret), "tree contains {secret}: {tree}");
        }
        for child in &failure.children {
            check_tree(child, secrets);
        }
    }
    check_tree(failure, secrets);
    let report = ToHumanReadableText.render(failure);
    assert!(report.contains("<redacted>"));
    for secret in secrets {
        assert!(
            !report.contains(secret),
            "report contains {secret}: {report}"
        );
    }
}

pub(crate) fn assert_custom_fact(failure: &AssertionFailure, label: &str, expected: usize) {
    let fact = failure
        .facts
        .iter()
        .find(|fact| fact.label == label)
        .unwrap();
    assert_custom_value(&fact.value, &expected);
}
