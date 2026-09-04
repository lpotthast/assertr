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

    pub(crate) fn compact(mut self) -> Self {
        self.compact = true;
        self
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
}

/// The structural body of a [`Rendered`] diagnostic value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderedBody {
    /// Text produced by a value renderer, or verbatim diagnostic text.
    Text {
        /// The retained text, without an omission marker.
        text: String,
        /// The number of characters omitted by the leaf budget.
        omitted_characters: usize,
    },

    /// Elements rendered with list or set syntax.
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
    Map {
        /// The retained entries.
        entries: Vec<(Rendered, Rendered)>,
        /// The number of entries omitted by the item budget.
        omitted: usize,
        /// Whether the entries were sorted by rendered text for deterministic diagnostics.
        sorted: bool,
    },

    /// Key/value entries rendered as a synthetic list of tuples.
    EntryList {
        /// The retained entries.
        entries: Vec<(Rendered, Rendered)>,
        /// The number of entries omitted by the item budget.
        omitted: usize,
        /// Whether the entries were sorted by rendered text for deterministic diagnostics.
        sorted: bool,
    },

    /// A tuple. This is used by synthetic entry lists so both the key and value remain nodes.
    Tuple {
        /// The tuple's items.
        items: Vec<Rendered>,
    },

    /// A one-field tuple variant, such as `Some(value)` or `Err(error)`.
    Variant {
        /// The variant name.
        name: &'static str,
        /// The rendered field.
        value: Box<Rendered>,
    },

    /// A named struct with rendered fields.
    Struct {
        /// The struct name.
        name: &'static str,
        /// The rendered fields in declaration order.
        fields: Vec<(&'static str, Rendered)>,
    },

    /// A field whose contents cannot be inspected, such as `<locked>` or `<borrowed>`.
    Placeholder(&'static str),
}

/// Converts a lazy rendering adapter or verbatim diagnostic value into an owned [`Rendered`]
/// tree.
pub trait IntoRendered {
    /// Renders the value once, using pretty leaf formatting where the renderer distinguishes it.
    fn into_rendered(self) -> Rendered;

    /// Internal compact-leaf counterpart used for map-key headings and specialized adapters.
    #[doc(hidden)]
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
        if rendered.shows_type_hint {
            let type_name = rendered
                .type_name
                .expect("only typed rendered values can show a type hint");
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
                write_suffix(f, *omitted, "element", *sorted)?;
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
                write_suffix(f, *omitted, "entry", *sorted)?;
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
                write_suffix(f, *omitted, "entry", *sorted)?;
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

fn write_suffix(
    f: &mut fmt::Formatter<'_>,
    omitted: usize,
    noun: &str,
    sorted: bool,
) -> fmt::Result {
    if omitted != 0 {
        write!(f, " ({})", omission(omitted, noun))?;
    }
    if sorted {
        f.write_str(" (sorted for rendering)")?;
    }
    Ok(())
}
