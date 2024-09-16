pub mod array;
pub mod bool;
pub mod debug;
pub mod display;
pub mod iter;
pub mod length;
pub mod num;
pub mod option;
pub mod partial_eq;
pub mod partial_ord;
pub mod range;
pub mod ref_cell;
pub mod result;
pub mod slice;
pub mod str_slice;

pub mod prelude {
    pub use super::array;
    pub use super::array::ArrayAssertions;
    pub use super::bool;
    pub use super::bool::BoolAssertions;
    pub use super::debug;
    pub use super::debug::DebugAssertions;
    pub use super::display;
    pub use super::display::DisplayAssertions;
    pub use super::iter;
    pub use super::iter::IntoIteratorAssertions;
    pub use super::length;
    pub use super::length::LengthAssertions;
    pub use super::num;
    pub use super::num::NumAssertions;
    pub use super::option;
    pub use super::option::OptionAssertions;
    pub use super::partial_eq;
    pub use super::partial_eq::PartialEqAssertions;
    pub use super::partial_ord;
    pub use super::partial_ord::PartialOrdAssertions;
    pub use super::range; // TODO
    pub use super::ref_cell; // TODO
    pub use super::result;
    pub use super::result::ResultAssertions;
    pub use super::slice;
    pub use super::slice::SliceAssertions;
    pub use super::str_slice;
    pub use super::str_slice::StrSliceAssertions;
}
