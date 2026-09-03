use alloc::{borrow::Cow, string::String};
use core::fmt::{self, Debug};

/// Controls the type hint attached to a rendered diagnostic value.
///
/// A type hint is presentation metadata rather than the value's canonical Rust type. Assertr
/// always captures the complete [`core::any::type_name`] independently. This policy selects the
/// hint derived from that name when a text diagnostic chooses to show it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TypeHint {
    /// Uses the complete [`core::any::type_name`] output as the hint.
    Full,

    /// Uses the Rust type's unqualified name as the hint.
    ///
    /// Module qualification, reference prefixes, and generic arguments are omitted. For example,
    /// `alloc::collections::btree::map::BTreeMap<K, V>` is shown as `BTreeMap`. Slices, arrays,
    /// tuples, and other composite types keep their structure and shorten every contained type
    /// the same way, so `&[alloc::string::String]` is shown as `[String]`.
    Short,

    /// Uses the given diagnostic label as the hint.
    Label(&'static str),
}

/// The canonical Rust type name and presentation hint of one concrete diagnostic value.
#[derive(Clone, Copy)]
pub(super) struct TypeInfo {
    pub(super) type_name: &'static str,
    pub(super) hint: TypeHint,
}

impl TypeInfo {
    pub(super) fn of<T: ?Sized>() -> Self {
        Self {
            type_name: core::any::type_name::<T>(),
            hint: TypeHint::Short,
        }
    }

    fn resolved_hint(self) -> Cow<'static, str> {
        match self.hint {
            TypeHint::Full => Cow::Borrowed(self.type_name),
            TypeHint::Short => Cow::Owned(short_rust_type_name(self.type_name)),
            TypeHint::Label(label) => Cow::Borrowed(label),
        }
    }
}

/// A [`Debug`] adapter that attaches Rust type metadata to a rendered diagnostic body.
///
/// [`RenderingContext`](super::RenderingContext) creates this adapter automatically whenever a
/// structural node represents a concrete Rust value. [`with_type_hint`](Self::with_type_hint)
/// customizes its metadata independently of [`show_type_hint`](Self::show_type_hint), which
/// controls only text output.
#[must_use]
pub struct Typed<D> {
    pub(super) info: TypeInfo,
    pub(super) body: D,
    pub(super) show_type_hint: bool,
}

impl<D> Typed<D> {
    pub(super) fn new<T: ?Sized>(body: D) -> Self {
        Self::from_info(body, TypeInfo::of::<T>())
    }

    pub(super) const fn from_info(body: D, info: TypeInfo) -> Self {
        Self {
            info,
            body,
            show_type_hint: false,
        }
    }

    /// Replaces this diagnostic value's type hint without changing whether text output shows it.
    pub fn with_type_hint(mut self, hint: TypeHint) -> Self {
        self.info.hint = hint;
        self
    }

    /// Controls whether text output shows this diagnostic value's type hint.
    ///
    /// The complete Rust type name and configured hint remain attached when `show` is `false`.
    /// This setting therefore does not affect future structured or machine-readable output.
    pub fn show_type_hint(mut self, show: bool) -> Self {
        self.show_type_hint = show;
        self
    }
}

impl<D: Debug> Debug for Typed<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.show_type_hint {
            write!(f, "{} ", self.info.resolved_hint())?;
        }
        Debug::fmt(&self.body, f)
    }
}

/// Removes reference prefixes, module qualification, and generic arguments from a
/// [`core::any::type_name`] while keeping the structure of composite types such as slices,
/// arrays, tuples, pointers, and function types.
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
    use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
    use core::fmt::Debug;

    use crate::prelude::*;

    use super::{TypeHint, Typed, short_rust_type_name};

    #[test]
    fn short_type_names_omit_paths_references_and_generic_arguments() {
        let type_name = core::any::type_name::<&mut BTreeMap<String, Vec<i32>>>();

        assert_that!(short_rust_type_name(type_name)).is_equal_to("BTreeMap");
    }

    #[test]
    fn short_type_names_keep_the_structure_of_composite_types() {
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

    #[test]
    fn type_info_always_keeps_the_complete_rust_name_and_a_hint() {
        let expected = core::any::type_name::<BTreeMap<String, Vec<i32>>>();

        for hint in [TypeHint::Full, TypeHint::Short, TypeHint::Label("Map")] {
            let typed = Typed::new::<BTreeMap<String, Vec<i32>>>("body").with_type_hint(hint);

            assert_that!(typed.info.type_name).is_equal_to(expected);
            assert_that!(typed.info.hint).is_equal_to(hint);
        }
    }

    #[test]
    fn type_hints_are_hidden_independently_of_the_stored_metadata() {
        let typed =
            Typed::new::<BTreeMap<String, Vec<i32>>>("body").with_type_hint(TypeHint::Label("Map"));

        assert_that!(format!("{typed:?}")).is_equal_to("\"body\"");
        assert_that!(typed.info.type_name)
            .is_equal_to(core::any::type_name::<BTreeMap<String, Vec<i32>>>());
        assert_that!(typed.info.hint).is_equal_to(TypeHint::Label("Map"));
    }

    #[test]
    fn shown_type_hints_resolve_only_during_formatting() {
        let rust_name = core::any::type_name::<BTreeMap<String, Vec<i32>>>();

        assert_that!(format!(
            "{:?}",
            Typed::new::<BTreeMap<String, Vec<i32>>>("body")
                .with_type_hint(TypeHint::Full)
                .show_type_hint(true)
        ))
        .is_equal_to(format!("{rust_name} \"body\""));
        assert_that!(format!(
            "{:?}",
            Typed::new::<BTreeMap<String, Vec<i32>>>("body").show_type_hint(true)
        ))
        .is_equal_to("BTreeMap \"body\"");
        assert_that!(format!(
            "{:?}",
            Typed::new::<BTreeMap<String, Vec<i32>>>("body")
                .with_type_hint(TypeHint::Label("Map"))
                .show_type_hint(true)
        ))
        .is_equal_to("Map \"body\"");
    }
}
