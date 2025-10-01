pub mod array;
pub mod bool;
pub mod char;
pub mod debug;
pub mod display;
pub mod r#fn;
pub mod iter;
pub mod length;
pub mod option;
pub mod partial_eq;
pub mod partial_ord;
pub mod poll;
pub mod range;
pub mod ref_cell;
pub mod result;
pub mod slice;
pub mod str_slice;

pub mod prelude {
    pub use super::array::ArrayAssertions;
    pub use super::bool::BoolAssertions;
    pub use super::char::CharAssertions;
    pub use super::debug::DebugAssertions;
    pub use super::display::DisplayAssertions;
    pub use super::r#fn::AsyncFnOnceAssertions;
    pub use super::r#fn::FnOnceAssertions;
    pub use super::iter::IntoIteratorAssertions;
    pub use super::iter::IteratorAssertions;
    pub use super::length::LengthAssertions;
    pub use super::option::OptionAssertions;
    pub use super::partial_eq::PartialEqAssertions;
    pub use super::partial_ord::PartialOrdAssertions;
    pub use super::poll::PollAssertions;
    pub use super::range::RangeAssertions;
    pub use super::range::RangeBoundAssertions;
    pub use super::ref_cell::RefCellAssertions;
    pub use super::result::ResultAssertions;
    pub use super::slice::SliceAssertions;
    pub use super::str_slice::StrSliceAssertions;
}

pub(crate) fn strip_quotation_marks(mut str: &str) -> &str {
    if str.starts_with('"') {
        str = str.strip_prefix('"').unwrap();
    }
    if str.ends_with('"') {
        str = str.strip_suffix('"').unwrap();
    }
    str
}
