use alloc::{borrow::Cow, boxed::Box, format, string::String, vec::Vec};
use core::fmt::{self, Debug, Display};

use super::{GroupStyle, omission};

/// One rendered diagnostic value, including its structural body and retained type information.
///
/// Values are rendered into this owned tree when a failure is built. A leaf's text is produced
/// exactly once by the active [`ValueRenderer`](super::ValueRenderer). Structural syntax and
/// truncation markers remain data until the tree is printed.
///
/// [`Display`] prints the tree as diagnostic text. The plain form (`{}`) is compact and separates
/// children with `, `. The alternate form (`{:#}`) uses the same indentation and trailing commas
/// as Rust's alternate `Debug` builders. Type hints and omission markers are derived from the
/// metadata stored in the tree. Leaves are never rendered again. [`Debug`] shows the tree's
/// structure instead.
///
/// Verbatim conversions from strings, formatting arguments, and primitive values bypass the
/// renderer and budget. Use them only for structural text or caller-authored prose. Render
/// assertion evidence through a [`RenderingContext`](super::RenderingContext) instead.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Rendered {
    /// The structural body of the value.
    pub body: RenderedBody,

    /// The canonical Rust type name of the value, or `None` for verbatim diagnostic text and
    /// synthetic groups.
    pub type_name: Option<&'static str>,

    /// Whether text output prefixes the body with the short type hint. The hint is only printed
    /// when a [`type_name`](Self::type_name) is also present.
    pub shows_type_hint: bool,
}

impl Rendered {
    /// A body representing a concrete value of type `T`, without a visible type hint.
    pub(crate) fn typed<T: ?Sized>(body: RenderedBody) -> Self {
        Self {
            type_name: Some(core::any::type_name::<T>()),
            ..Self::untyped(body)
        }
    }

    /// A body without an outer Rust type, such as verbatim text or a synthetic group.
    pub(crate) const fn untyped(body: RenderedBody) -> Self {
        Self {
            body,
            type_name: None,
            shows_type_hint: false,
        }
    }

    fn verbatim(text: String) -> Self {
        Self::untyped(RenderedBody::Text {
            text,
            omitted_characters: 0,
        })
    }

    /// Controls whether text output prefixes this value with its short type hint, such as
    /// `BTreeMap` or `[String]`.
    ///
    /// The complete Rust type name remains attached when `show` is `false`.
    #[must_use]
    pub fn show_type_hint(mut self, show: bool) -> Self {
        self.shows_type_hint = show;
        self
    }
}

/// The structural body of a [`Rendered`] diagnostic value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderedBody {
    /// Text produced by a value renderer, or verbatim diagnostic text.
    #[non_exhaustive]
    Text {
        /// The retained text, without an omission marker.
        text: String,
        /// The number of characters omitted by the leaf budget.
        omitted_characters: usize,
    },

    /// Elements rendered with list or set syntax.
    #[non_exhaustive]
    Group {
        /// The delimiters used for the group.
        style: GroupStyle,
        /// The retained elements.
        items: Vec<Rendered>,
        /// The number of elements omitted by the item budget.
        omitted: usize,
        /// Whether the elements were sorted by rendered text for deterministic diagnostics.
        sorted: bool,
    },

    /// Key/value entries rendered with map syntax.
    #[non_exhaustive]
    Map {
        /// The retained entries.
        entries: Vec<(Rendered, Rendered)>,
        /// The number of entries omitted by the item budget.
        omitted: usize,
        /// Whether the entries were sorted by rendered text for deterministic diagnostics.
        sorted: bool,
    },

    /// A tuple, such as one key/value entry of a synthetic entry list.
    #[non_exhaustive]
    Tuple {
        /// The tuple's items.
        items: Vec<Rendered>,
    },

    /// A one-field tuple variant, such as `Some(value)` or `Err(error)`.
    #[non_exhaustive]
    Variant {
        /// The variant name.
        name: &'static str,
        /// The rendered field.
        value: Box<Rendered>,
    },

    /// A named struct with rendered fields.
    #[non_exhaustive]
    Struct {
        /// The struct name.
        name: &'static str,
        /// The rendered fields in declaration order.
        fields: Vec<(&'static str, Rendered)>,
    },

    /// A field whose contents cannot be inspected, such as `<locked>` or `<borrowed>`.
    Placeholder(&'static str),
}

impl From<fmt::Arguments<'_>> for Rendered {
    fn from(text: fmt::Arguments<'_>) -> Self {
        Self::verbatim(format!("{text}"))
    }
}

impl From<String> for Rendered {
    fn from(text: String) -> Self {
        Self::verbatim(text)
    }
}

impl From<&str> for Rendered {
    fn from(text: &str) -> Self {
        Self::verbatim(text.into())
    }
}

impl From<Cow<'_, str>> for Rendered {
    fn from(text: Cow<'_, str>) -> Self {
        Self::verbatim(text.into_owned())
    }
}

macro_rules! impl_verbatim_primitives {
    ($($type:ty),+ $(,)?) => {$ (
        impl From<$type> for Rendered {
            fn from(value: $type) -> Self {
                Self::verbatim(format!("{value}"))
            }
        }
    )+ };
}

impl_verbatim_primitives!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool, char,
);

/// A rendered tuple of two diagnostic values, such as the entry shown by `does_not_contain_entry`.
impl<A: Into<Rendered>, B: Into<Rendered>> From<(A, B)> for Rendered {
    fn from((first, second): (A, B)) -> Self {
        Self::untyped(RenderedBody::Tuple {
            items: alloc::vec![first.into(), second.into()],
        })
    }
}

impl Display for Rendered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&Printed(self), f)
    }
}

/// Prints a tree through `Debug` so the standard builders supply both layouts.
struct Printed<'a>(&'a Rendered);

impl Debug for Printed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = self.0;
        if rendered.shows_type_hint
            && let Some(type_name) = rendered.type_name
        {
            write!(f, "{} ", short_rust_type_name(type_name))?;
        }

        match &rendered.body {
            RenderedBody::Text {
                text,
                omitted_characters,
            } => {
                f.write_str(text)?;
                if *omitted_characters != 0 {
                    f.write_str(&omission(*omitted_characters, "character"))?;
                }
            }
            RenderedBody::Group {
                style,
                items,
                omitted,
                sorted,
            } => {
                match style {
                    GroupStyle::List => {
                        f.debug_list().entries(items.iter().map(Printed)).finish()?;
                    }
                    GroupStyle::Set => f.debug_set().entries(items.iter().map(Printed)).finish()?,
                }
                write_suffix(f, items.len(), *omitted, "element", *sorted)?;
            }
            RenderedBody::Map {
                entries,
                omitted,
                sorted,
            } => {
                f.debug_map()
                    .entries(
                        entries
                            .iter()
                            .map(|(key, value)| (Printed(key), Printed(value))),
                    )
                    .finish()?;
                write_suffix(f, entries.len(), *omitted, "entry", *sorted)?;
            }
            RenderedBody::Tuple { items } => {
                let mut tuple = f.debug_tuple("");
                for item in items {
                    tuple.field(&Printed(item));
                }
                tuple.finish()?;
            }
            RenderedBody::Variant { name, value } => {
                f.debug_tuple(name).field(&Printed(value)).finish()?;
            }
            RenderedBody::Struct { name, fields } => {
                let mut structure = f.debug_struct(name);
                for (name, value) in fields {
                    structure.field(name, &Printed(value));
                }
                structure.finish()?;
            }
            RenderedBody::Placeholder(text) => f.write_str(text)?,
        }
        Ok(())
    }
}

/// Writes the omission and sorting markers following a group or map body.
///
/// Sorting is only reported when at least two items were considered, counting omitted ones,
/// because a single item has no order that sorting could have changed.
fn write_suffix(
    f: &mut fmt::Formatter<'_>,
    retained: usize,
    omitted: usize,
    noun: &str,
    sorted: bool,
) -> fmt::Result {
    if omitted != 0 {
        write!(f, " ({})", omission(omitted, noun))?;
    }
    if sorted && retained.saturating_add(omitted) > 1 {
        f.write_str(" (sorted for rendering)")?;
    }
    Ok(())
}

/// Removes reference prefixes, module qualification, and generic arguments from a
/// [`core::any::type_name`] while keeping the structure of composite types such as slices, arrays,
/// tuples, pointers, and function types.
///
/// For example, `alloc::collections::btree::map::BTreeMap<K, V>` is shown as `BTreeMap`, and
/// `&[alloc::string::String]` as `[String]`.
fn short_rust_type_name(type_name: &str) -> String {
    let type_name = type_name.trim_start_matches('&');
    let type_name = type_name.strip_prefix("mut ").unwrap_or(type_name);
    let mut short = String::with_capacity(type_name.len());
    let mut generic_depth = 0_usize;
    let mut chars = type_name.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '<' => generic_depth += 1,
            '>' if generic_depth > 0 => generic_depth -= 1,
            '-' if chars.peek() == Some(&'>') => {
                chars.next();
                if generic_depth == 0 {
                    short.push_str("->");
                }
            }
            _ if generic_depth > 0 => {}
            ':' if chars.peek() == Some(&':') => {
                chars.next();
                let unqualified = short
                    .trim_end_matches(|c: char| c.is_alphanumeric() || c == '_')
                    .len();
                short.truncate(unqualified);
            }
            _ => short.push(character),
        }
    }
    short
}

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};
    use core::fmt::Debug;

    use super::{GroupStyle, Rendered, RenderedBody, short_rust_type_name};
    use crate::prelude::*;

    fn leaf(text: &str) -> Rendered {
        Rendered::from(text)
    }

    mod type_hint {
        use super::*;

        #[test]
        fn is_shown_in_short_form_for_typed_values() {
            let rendered =
                Rendered::typed::<BTreeMap<String, i32>>(RenderedBody::Placeholder("<locked>"))
                    .show_type_hint(true);

            assert_that!(rendered.to_string()).is_equal_to("BTreeMap <locked>");
        }

        #[test]
        fn is_skipped_without_a_type_name() {
            let rendered = leaf("text").show_type_hint(true);

            assert_that!(rendered.to_string()).is_equal_to("text");
            assert_that!(format!("{rendered:#}")).is_equal_to("text");
        }
    }

    mod short_type_names {
        use super::*;

        #[test]
        fn omit_paths_references_and_generic_arguments() {
            let type_name = core::any::type_name::<&mut BTreeMap<String, Vec<i32>>>();

            assert_that!(short_rust_type_name(type_name)).is_equal_to("BTreeMap");
        }

        #[test]
        fn keep_the_structure_of_composite_types() {
            assert_that!(short_rust_type_name(core::any::type_name::<&[String]>()))
                .is_equal_to("[String]");
            assert_that!(short_rust_type_name(core::any::type_name::<[String; 3]>()))
                .is_equal_to("[String; 3]");
            assert_that!(short_rust_type_name(
                core::any::type_name::<[Vec<String>; 3]>()
            ))
            .is_equal_to("[Vec; 3]");
            assert_that!(short_rust_type_name(core::any::type_name::<(
                String,
                &str,
                i32
            )>()))
            .is_equal_to("(String, &str, i32)");
            assert_that!(short_rust_type_name(core::any::type_name::<*const String>()))
                .is_equal_to("*const String");
            assert_that!(short_rust_type_name(core::any::type_name::<
                fn(Vec<i32>) -> String,
            >()))
            .is_equal_to("fn(Vec) -> String");
            assert_that!(short_rust_type_name(core::any::type_name::<&dyn Debug>()))
                .is_equal_to("dyn Debug");
        }
    }

    mod sorted_suffix {
        use super::*;

        fn group(items: Vec<Rendered>, omitted: usize) -> Rendered {
            Rendered::untyped(RenderedBody::Group {
                style: GroupStyle::Set,
                items,
                omitted,
                sorted: true,
            })
        }

        fn map(entries: Vec<(Rendered, Rendered)>, omitted: usize) -> Rendered {
            Rendered::untyped(RenderedBody::Map {
                entries,
                omitted,
                sorted: true,
            })
        }

        #[test]
        fn is_omitted_for_a_single_element() {
            assert_that!(group(vec![leaf("1")], 0).to_string()).is_equal_to("{1}");
            assert_that!(group(Vec::new(), 0).to_string()).is_equal_to("{}");
        }

        #[test]
        fn is_omitted_for_a_single_entry() {
            assert_that!(map(vec![(leaf("1"), leaf("2"))], 0).to_string()).is_equal_to("{1: 2}");
        }

        #[test]
        fn is_shown_for_multiple_retained_elements() {
            assert_that!(group(vec![leaf("1"), leaf("2")], 0).to_string())
                .is_equal_to("{1, 2} (sorted for rendering)");
        }

        #[test]
        fn counts_omitted_elements() {
            assert_that!(group(vec![leaf("1")], 1).to_string())
                .is_equal_to("{1} (... 1 more element ...) (sorted for rendering)");
            assert_that!(map(Vec::new(), 2).to_string())
                .is_equal_to("{} (... 2 more entries ...) (sorted for rendering)");
        }
    }
}
