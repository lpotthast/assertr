#![allow(dead_code)]

// This test verifies AssertrEq compiles for a public non-Debug field when the user supplies a
// custom renderer for the actual type, matcher type, and field, without requiring DebugRenderer
// for that field.

use assertr::prelude::*;

#[derive(PartialEq)]
pub struct NonDebug(u32);

#[derive(AssertrEq)]
pub struct Subject {
    pub hidden: NonDebug,
}

#[derive(Clone, Copy)]
struct Renderer;

impl ValueRenderer<Subject> for Renderer {
    fn fmt(&self, value: &Subject, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Subject({})", value.hidden.0))
    }
}

impl ValueRenderer<SubjectAssertrEq> for Renderer {
    fn fmt(
        &self,
        _value: &SubjectAssertrEq,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str("SubjectAssertrEq(..)")
    }
}

impl ValueRenderer<NonDebug> for Renderer {
    fn fmt(&self, value: &NonDebug, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Hidden({})", value.0))
    }
}

fn main() {
    Subject {
        hidden: NonDebug(1),
    }
    .must()
    .with_renderer(Renderer)
    .with_location(false)
    .be_equal_to(SubjectAssertrEq {
        hidden: eq(NonDebug(1)),
    });
}
