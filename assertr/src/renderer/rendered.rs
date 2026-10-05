use alloc::{borrow::Cow, boxed::Box, format, string::String, vec::Vec};
use core::fmt::{self, Debug, Write};

use super::{GroupStyle, TypeHint, omission, type_info::short_rust_type_name};

/// One rendered diagnostic value, including its structural body and retained type information.
///
/// Values are rendered into this owned tree when a failure is built. A leaf's text is produced
/// exactly once by the active [`ValueRenderer`](super::ValueRenderer). Structural syntax and
/// truncation markers remain data until an adapter explicitly prints the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Rendered {
    /// The structural body of the value.
    pub body: RenderedBody,

    /// The canonical Rust type name of the value, or `None` for verbatim diagnostic text.
    pub type_name: Option<&'static str>,

    /// How the type is named when its hint is shown.
    pub hint: TypeHint,

    /// Whether text reports prefix the body with the type hint.
    pub shows_type_hint: bool,

    /// Whether this node keeps compact structural layout when embedded in pretty output.
    ///
    /// The active value renderer has already produced every leaf in the corresponding compact
    /// mode. This flag retains the structural presentation selected by an internal compact
    /// adapter, such as an inline range or a map key.
    pub compact: bool,
}

impl Rendered {
    pub(crate) fn typed(
        body: RenderedBody,
        type_name: &'static str,
        hint: TypeHint,
        shows_type_hint: bool,
    ) -> Self {
        Self {
            body,
            type_name: Some(type_name),
            hint,
            shows_type_hint,
            compact: false,
        }
    }

    pub(crate) fn verbatim(text: String) -> Self {
        Self {
            body: RenderedBody::Text {
                text,
                omitted_characters: 0,
            },
            type_name: None,
            hint: TypeHint::Short,
            shows_type_hint: false,
            compact: false,
        }
    }

    /// Writes the human-readable representation of this value.
    ///
    /// Pretty output uses the same indentation and trailing commas as Rust's alternate `Debug`
    /// builders, except where a node retains an explicit compact layout. Compact output separates
    /// children with `, `. Type hints and omission markers are derived from the metadata stored in
    /// the tree. Leaves are never rendered again.
    ///
    /// # Errors
    ///
    /// Returns the error reported by `w` if it cannot accept the complete representation.
    pub fn write(&self, w: &mut dyn Write, pretty: bool) -> fmt::Result {
        if pretty {
            write!(w, "{:#?}", Printed(self))
        } else {
            write!(w, "{:?}", Printed(self))
        }
    }

    pub(crate) fn text(&self, pretty: bool) -> String {
        let mut output = String::new();
        self.write(&mut output, pretty)
            .expect("writing a rendered value to a String cannot fail");
        output
    }

    /// Borrows the structural body of this diagnostic value.
    #[must_use]
    pub const fn body(&self) -> &RenderedBody {
        &self.body
    }

    /// Returns the canonical Rust type name, or `None` for verbatim diagnostic text.
    #[must_use]
    pub const fn type_name(&self) -> Option<&'static str> {
        self.type_name
    }

    /// Returns how the type is named when its hint is shown.
    #[must_use]
    pub const fn hint(&self) -> TypeHint {
        self.hint
    }

    /// Returns whether text reports prefix the body with the type hint.
    ///
    /// The hint is only printed when a [`type_name`](Self::type_name) is also present.
    #[must_use]
    pub const fn shows_type_hint(&self) -> bool {
        self.shows_type_hint
    }

    /// Returns whether this node keeps compact structural layout when embedded in pretty output.
    #[must_use]
    pub const fn is_compact(&self) -> bool {
        self.compact
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

    /// Key/value entries rendered as a synthetic list of tuples.
    #[non_exhaustive]
    EntryList {
        /// The retained entries.
        entries: Vec<(Rendered, Rendered)>,
        /// The number of entries omitted by the item budget.
        omitted: usize,
        /// Whether the entries were sorted by rendered text for deterministic diagnostics.
        sorted: bool,
    },

    /// A tuple. This is used by synthetic entry lists so both the key and value remain nodes.
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

/// Converts a lazy rendering adapter or verbatim diagnostic value into an owned [`Rendered`] tree.
///
/// Conversions from text, formatting arguments, and primitive values are verbatim. They do not
/// apply a renderer or budget and must not receive unrendered assertion evidence. Use
/// [`RenderingContext::value`](super::RenderingContext::value) for tested values, including
/// lengths, counts, expected indices, and errors. Paths, structural positions, omission summaries,
/// type names, and explicit diagnostic prose remain owned by Assertr.
pub trait IntoRendered {
    /// Renders the value once, using pretty leaf formatting where the renderer distinguishes it.
    fn into_rendered(self) -> Rendered;

    /// Renders the value once, using compact leaf formatting and compact structural layout.
    ///
    /// Use this for evidence embedded inline in surrounding text, where multi-line pretty output
    /// would break the layout. Assertr renders map keys in
    /// [`PathSegment::Key`](crate::failure::PathSegment::Key) path segments this way, for example
    /// `entry["key"]`. The leaf renderer receives a formatter with `f.alternate() == false`.
    ///
    /// The default implementation forwards to [`into_rendered`](Self::into_rendered), which suits
    /// verbatim conversions that have no separate compact form.
    fn into_rendered_compact(self) -> Rendered
    where
        Self: Sized,
    {
        self.into_rendered()
    }
}

impl IntoRendered for fmt::Arguments<'_> {
    fn into_rendered(self) -> Rendered {
        Rendered::verbatim(format!("{self}"))
    }
}

impl IntoRendered for String {
    fn into_rendered(self) -> Rendered {
        Rendered::verbatim(self)
    }
}

impl IntoRendered for &str {
    fn into_rendered(self) -> Rendered {
        Rendered::verbatim(self.into())
    }
}

impl IntoRendered for Cow<'_, str> {
    fn into_rendered(self) -> Rendered {
        Rendered::verbatim(self.into_owned())
    }
}

macro_rules! impl_verbatim_numbers {
    ($($type:ty),+ $(,)?) => {$ (
        impl IntoRendered for $type {
            fn into_rendered(self) -> Rendered {
                Rendered::verbatim(format!("{self}"))
            }
        }
    )+ };
}

impl_verbatim_numbers!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool, char,
);

/// A rendered tuple of two diagnostic values.
///
/// This supports the entry shown by `does_not_contain_entry` without flattening either child.
impl<A: IntoRendered, B: IntoRendered> IntoRendered for (A, B) {
    fn into_rendered(self) -> Rendered {
        Rendered {
            body: RenderedBody::Tuple {
                items: alloc::vec![self.0.into_rendered(), self.1.into_rendered()],
            },
            type_name: None,
            hint: TypeHint::Short,
            shows_type_hint: false,
            compact: false,
        }
    }

    fn into_rendered_compact(self) -> Rendered {
        Rendered {
            body: RenderedBody::Tuple {
                items: alloc::vec![
                    self.0.into_rendered_compact(),
                    self.1.into_rendered_compact(),
                ],
            },
            type_name: None,
            hint: TypeHint::Short,
            shows_type_hint: false,
            compact: true,
        }
    }
}

struct Printed<'a>(&'a Rendered);

impl Debug for Printed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = self.0;
        if rendered.compact && f.alternate() {
            return f.write_str(&rendered.text(false));
        }
        if rendered.shows_type_hint
            && let Some(type_name) = rendered.type_name
        {
            match rendered.hint {
                TypeHint::Full => write!(f, "{type_name} ")?,
                TypeHint::Short => write!(f, "{} ", short_rust_type_name(type_name))?,
                TypeHint::Label(label) => write!(f, "{label} ")?,
            }
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
            RenderedBody::EntryList {
                entries,
                omitted,
                sorted,
            } => {
                f.debug_list()
                    .entries(
                        entries
                            .iter()
                            .map(|(key, value)| PrintedTuple([key, value])),
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

struct PrintedTuple<'a>([&'a Rendered; 2]);

impl Debug for PrintedTuple<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&Printed(self.0[0]))
            .field(&Printed(self.0[1]))
            .finish()
    }
}

pub(super) fn tuple_text(key: &Rendered, value: &Rendered, pretty: bool) -> String {
    if pretty {
        format!("{:#?}", PrintedTuple([key, value]))
    } else {
        format!("{:?}", PrintedTuple([key, value]))
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

impl IntoRendered for Rendered {
    fn into_rendered(self) -> Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec, vec::Vec};

    use crate::prelude::*;

    use super::{GroupStyle, Rendered, RenderedBody, TypeHint};

    fn leaf(text: &str) -> Rendered {
        Rendered::verbatim(String::from(text))
    }

    fn untyped(body: RenderedBody) -> Rendered {
        let mut rendered = leaf("");
        rendered.body = body;
        rendered
    }

    mod accessors {
        use super::*;

        #[test]
        fn expose_every_field() {
            let rendered = Rendered::typed(
                RenderedBody::Placeholder("<locked>"),
                "alloc::vec::Vec<i32>",
                TypeHint::Full,
                true,
            );

            assert_that!(rendered.body()).is_equal_to(RenderedBody::Placeholder("<locked>"));
            assert_that!(rendered.type_name()).is_equal_to(Some("alloc::vec::Vec<i32>"));
            assert_that!(rendered.hint()).is_equal_to(TypeHint::Full);
            assert_that!(rendered.shows_type_hint()).is_true();
            assert_that!(rendered.is_compact()).is_false();
        }
    }

    mod type_hint {
        use super::*;

        #[test]
        fn is_shown_for_typed_values() {
            let rendered = Rendered::typed(
                RenderedBody::Placeholder("<locked>"),
                "i32",
                TypeHint::Short,
                true,
            );

            assert_that!(rendered.text(false)).is_equal_to("i32 <locked>");
        }

        #[test]
        fn is_skipped_without_a_type_name() {
            let mut rendered = leaf("text");
            rendered.shows_type_hint = true;

            assert_that!(rendered.text(false)).is_equal_to("text");
            assert_that!(rendered.text(true)).is_equal_to("text");
        }
    }

    mod sorted_suffix {
        use super::*;

        fn group(items: Vec<Rendered>, omitted: usize) -> Rendered {
            untyped(RenderedBody::Group {
                style: GroupStyle::Set,
                items,
                omitted,
                sorted: true,
            })
        }

        fn map(entries: Vec<(Rendered, Rendered)>, omitted: usize) -> Rendered {
            untyped(RenderedBody::Map {
                entries,
                omitted,
                sorted: true,
            })
        }

        #[test]
        fn is_omitted_for_a_single_element() {
            assert_that!(group(vec![leaf("1")], 0).text(false)).is_equal_to("{1}");
            assert_that!(group(Vec::new(), 0).text(false)).is_equal_to("{}");
        }

        #[test]
        fn is_omitted_for_a_single_entry() {
            assert_that!(map(vec![(leaf("1"), leaf("2"))], 0).text(false)).is_equal_to("{1: 2}");
        }

        #[test]
        fn is_shown_for_multiple_retained_elements() {
            assert_that!(group(vec![leaf("1"), leaf("2")], 0).text(false))
                .is_equal_to("{1, 2} (sorted for rendering)");
        }

        #[test]
        fn counts_omitted_elements() {
            assert_that!(group(vec![leaf("1")], 1).text(false))
                .is_equal_to("{1} (... 1 more element ...) (sorted for rendering)");
            assert_that!(map(Vec::new(), 2).text(false))
                .is_equal_to("{} (... 2 more entries ...) (sorted for rendering)");
        }
    }
}
