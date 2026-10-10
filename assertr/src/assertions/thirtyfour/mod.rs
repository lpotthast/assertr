//! Fallible browser reads of the `thirtyfour` integration.
//!
//! Assertions on a `WebElement` come from
//! [`ThirtyfourWebElementAssertions`](crate::assertions::ThirtyfourWebElementAssertions), which
//! lives in [`assertions`](crate::assertions) and the prelude. This module holds the [`read`]
//! helpers. They return `WebDriver` errors instead of failing, so they also work in ordinary code
//! and as fresh observations of eventual assertions.

pub(crate) mod element;
pub mod read;
