use alloc::{format, string::String, vec::Vec};
use core::{fmt, fmt::Debug};

use crate::{
    details,
    renderer::{
        DebugRenderer, GroupStyle, RenderedValue, RenderedValues, RenderingBudget,
        RenderingContext, Typed, ValueRenderer,
    },
};

/// Differences recorded during an [`AssertrPartialEq`] comparison.
///
/// Its [`Debug`] representation is used as structured diagnostic detail.
pub struct Differences {
    pub(crate) differences: Vec<String>,
}

impl Default for Differences {
    fn default() -> Self {
        Self::new()
    }
}

impl Differences {
    /// Creates an empty difference list.
    #[must_use]
    pub fn new() -> Self {
        Self {
            differences: Vec::new(),
        }
    }
}

impl Debug for Differences {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.differences.iter().map(|it| details::DisplayString(it)))
            .finish()
    }
}

/// Collects differences and provides the renderer for an [`AssertrPartialEq`] comparison.
///
/// Comparison implementations should record a difference only when returning `false`. The
/// context is optional because callers that need only a boolean result may omit diagnostics.
pub struct EqContext<'r, R = DebugRenderer> {
    pub(crate) differences: Differences,
    rendering: RenderingContext<'r, R>,
}

impl Default for EqContext<'static, DebugRenderer> {
    fn default() -> Self {
        Self::new()
    }
}

impl EqContext<'static, DebugRenderer> {
    /// Creates an empty context using [`DebugRenderer`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_renderer(&DebugRenderer)
    }
}

impl<'r, R> EqContext<'r, R> {
    /// Creates an empty context using `renderer`.
    #[must_use]
    pub fn with_renderer(renderer: &'r R) -> Self {
        Self::with_rendering(RenderingContext::new(renderer, RenderingBudget::DEFAULT))
    }

    pub(crate) fn with_rendering(rendering: RenderingContext<'r, R>) -> Self {
        Self {
            differences: Differences::default(),
            rendering,
        }
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            differences: Differences::default(),
            rendering: self.rendering,
        }
    }

    /// Appends a complete, pre-rendered difference.
    pub fn add_difference(&mut self, difference: String) {
        self.differences.differences.push(difference);
    }

    /// Records that `field_name` differs without rendering either value.
    pub fn add_field_difference_without_values(&mut self, field_name: &str) {
        self.differences
            .differences
            .push(format!("\"{field_name}\": values are not equal"));
    }

    /// Records a field difference using the context's renderer for both values.
    pub fn add_field_difference_rendered<A: ?Sized, E: ?Sized>(
        &mut self,
        field_name: &str,
        expected: &E,
        actual: &A,
    ) where
        R: ValueRenderer<A> + ValueRenderer<E>,
    {
        let expected = self.render_value(expected);
        let actual = self.render_value(actual);
        let difference = format!("\"{field_name}\": expected {expected:#?}, but was {actual:#?}");
        self.differences.differences.push(difference);
    }

    /// Records a field difference using the values' [`Debug`] implementations.
    ///
    /// Prefer [`EqContext::add_field_difference_rendered`] when using a custom renderer.
    pub fn add_field_difference(
        &mut self,
        field_name: &str,
        expected: impl Debug,
        actual: impl Debug,
    ) {
        self.differences.differences.push(format!(
            "\"{field_name}\": expected {expected:#?}, but was {actual:#?}"
        ));
    }

    /// Wraps one value so its [`Debug`] output uses the context's renderer and rendering budget.
    ///
    /// The adapter has no destructor, so the context can be mutated again as soon as the adapter
    /// is no longer used.
    pub fn render_value<'a, T: ?Sized>(&'a self, value: &'a T) -> Typed<RenderedValue<'a, T, R>>
    where
        R: ValueRenderer<T>,
    {
        self.rendering.value(value)
    }

    /// Wraps a slice of references so their [`Debug`] output uses the context's renderer and the
    /// requested structural style.
    ///
    /// The adapter has no destructor, so the context can be mutated again as soon as the adapter
    /// is no longer used.
    pub fn render_values<'a, T: ?Sized>(
        &'a self,
        values: &'a [&'a T],
        style: GroupStyle,
    ) -> RenderedValues<'a, T, [&'a T], R>
    where
        R: ValueRenderer<T>,
    {
        self.rendering.borrowed_values(values, style)
    }
}

/// Equality that can explain itself.
///
/// This is `PartialEq` with an additional [`EqContext`] that records human-readable differences
/// while comparing, which is how a failed `is_equal_to` lists what differed field by field. Every
/// `PartialEq` type implements it through a blanket implementation. The `AssertrEq` derive
/// implements it between a struct and its generated matcher. Implement it directly only for a type
/// that needs a custom comparison with diagnostics of its own and does not already implement
/// `PartialEq<Rhs>`. `R` is the renderer used to render the recorded differences.
pub trait AssertrPartialEq<Rhs: ?Sized = Self, R = DebugRenderer> {
    /// Compares `self` and `other`, recording differences in `ctx` when provided.
    #[must_use]
    fn eq(&self, other: &Rhs, ctx: Option<&mut EqContext<'_, R>>) -> bool;

    /// Compares `self` and `other` for inequality.
    ///
    /// The default negates [`AssertrPartialEq::eq`] and should normally be retained.
    #[must_use]
    fn ne(&self, other: &Rhs, ctx: Option<&mut EqContext<'_, R>>) -> bool {
        !self.eq(other, ctx)
    }
}

// AssertrPartialEq must be implemented for each type already being PartialEq,
// so that we can solely rely on, and call, this ctx-enabled version in our assertions.
impl<Rhs: ?Sized, T: PartialEq<Rhs>, R> AssertrPartialEq<Rhs, R> for T {
    fn eq(&self, other: &Rhs, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
        PartialEq::eq(self, other)
    }
    fn ne(&self, other: &Rhs, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
        PartialEq::ne(self, other)
    }
}

impl<T1, T2, R> AssertrPartialEq<[T2], R> for [T1]
where
    T1: AssertrPartialEq<T2, R>,
{
    fn eq(&self, other: &[T2], mut ctx: Option<&mut EqContext<'_, R>>) -> bool {
        self.len() == other.len()
            && self.iter().enumerate().all(|(i, t1)| {
                other
                    .get(i)
                    .is_some_and(|t2| AssertrPartialEq::eq(t1, t2, ctx.as_deref_mut()))
            })
    }

    fn ne(&self, other: &[T2], ctx: Option<&mut EqContext<'_, R>>) -> bool {
        !Self::eq(self, other, ctx)
    }
}

/// An expected field value in a partial comparison: a concrete value, or "anything".
///
/// The matcher structs generated by the `AssertrEq` derive consist of these. Build them with
/// [`eq`] and [`any`]. `T` does not need to be `PartialEq`, because it may itself be a matcher
/// type that is compared through [`AssertrPartialEq`].
#[derive(Default)]
pub enum Eq<T> {
    /// Matches any value.
    #[default]
    Any,
    /// Matches only the contained value.
    Eq(T),
}

/// Expects exactly `v`. See [`Eq`](enum@Eq).
pub fn eq<T>(v: T) -> Eq<T> {
    Eq::Eq(v)
}

/// Accepts any value. See [`Eq`](enum@Eq).
#[must_use]
pub fn any<T>() -> Eq<T> {
    Eq::Any
}

impl<T: Debug> Debug for Eq<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eq::Any => f.write_str("Eq::Any"),
            Eq::Eq(v) => f.write_fmt(format_args!("Eq::Eq({v:?})")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::test_support::{SENTINEL, SentinelRenderer};

    #[test]
    fn render_values_honors_the_requested_group_style() {
        let values = [&1, &2];
        let renderer = SentinelRenderer;
        let context = EqContext::with_renderer(&renderer);

        assert_that!(format!(
            "{:?}",
            context.render_values(&values, GroupStyle::List)
        ))
        .is_equal_to(format!("[{SENTINEL}, {SENTINEL}]"));
        assert_that!(format!(
            "{:?}",
            context.render_values(&values, GroupStyle::Set)
        ))
        .is_equal_to(format!("{{{SENTINEL}, {SENTINEL}}}"));
    }

    #[test]
    fn rendered_adapters_release_the_context_before_it_is_mutated() {
        let renderer = SentinelRenderer;
        let mut context = EqContext::with_renderer(&renderer);
        let value = 1;

        let adapter = context.render_value(&value);
        let difference = format!("rendered {adapter:?}");
        context.add_difference(difference);

        assert_that!(&context.differences.differences)
            .is_equal_to(vec![format!("rendered {SENTINEL}")]);
    }

    #[cfg(feature = "derive")]
    mod derive_renderer {
        use super::*;

        #[derive(PartialEq)]
        pub struct Hidden(u32);

        #[derive(AssertrEq)]
        pub struct Subject {
            pub hidden: Hidden,
        }

        #[test]
        fn reports_non_debug_field_differences_with_the_active_renderer() {
            let failures = assert_that!(Subject { hidden: Hidden(1) })
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| {
                    it.is_equal_to(SubjectAssertrEq {
                        hidden: eq(Hidden(2)),
                    })
                });

            assert_that!(failures[0].facts[0].value.as_str())
                .contains(format!("expected {SENTINEL}, but was {SENTINEL}"));
        }
    }

    /// Generated matchers nest inside other matchers, `Vec`s, and `HashMap`s. The default
    /// renderer must render the whole expected structure, including the `Eq::Eq` wrappers.
    #[cfg(feature = "derive")]
    mod generated_matchers {
        use super::*;

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
        fn nested_matchers_render_through_the_debug_renderer() {
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
        fn vec_matchers_render_through_the_debug_renderer() {
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
                map_type = "std::collections::HashMap<String, ChildAssertrEq>",
                compare_with = "::assertr::cmp::hashmap::compare",
                compare_bounds = "Child: ::assertr::cmp::hashmap::CompareValue<ChildAssertrEq, R>"
            )]
            pub children: std::collections::HashMap<String, Child>,
        }

        #[test]
        #[cfg(feature = "std")]
        fn hashmap_matchers_render_through_the_debug_renderer() {
            use std::collections::HashMap;

            let failures = assert_that!(MapParent {
                children: HashMap::from([(String::from("first"), Child { id: 1 })]),
            })
            .with_location(false)
            .capture(|it| {
                it.is_equal_to(MapParentAssertrEq {
                    children: eq(HashMap::from([(
                        String::from("first"),
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
}
