//! Explicit stdout logging for human-readable failure reports.
//!
//! [`StdOutLogger`] writes and flushes [`HumanReadableText`] on the calling thread, returning
//! any I/O error to its caller. It requires the `std` feature and can terminate an adapter chain
//! that a caller runs explicitly on a captured failure.
//!
//! Neither capture mode nor panic-mode failure handling invokes this logger automatically.
//! Its output is `()`, so it cannot be selected as a panic presentation adapter.

use std::io::{self, Write};

use super::{super::Adapter, HumanReadableText};

/// Writes human-readable assertion failures to standard output.
///
/// This is an explicit sink for processing captured failures. Panic presentation never selects
/// it automatically, and its `()` output cannot be installed as a panic presentation.
///
/// The text is flushed before the adapter returns so a subsequent panic cannot leave the report
/// buffered. I/O failures are returned to the enclosing adapter chain.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdOutLogger;

impl Adapter<HumanReadableText> for StdOutLogger {
    type Output = ();
    type Error = io::Error;

    fn adapt(&self, text: &HumanReadableText) -> Result<Self::Output, Self::Error> {
        write_text(io::stdout().lock(), text)
    }
}

fn write_text(mut output: impl Write, text: &HumanReadableText) -> io::Result<()> {
    output.write_all(text.as_str().as_bytes())?;
    output.flush()
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::failure::adapter::ToHumanReadableText;
    use crate::failure::{FailureBuilder, FailureKind};

    #[test]
    fn writes_the_complete_human_readable_text() {
        let failure = FailureBuilder::detached::<i32>(FailureKind::Equality)
            .actual(1)
            .expected(2)
            .build();
        let text = ToHumanReadableText.render(&failure);
        let mut output = Vec::new();

        write_text(&mut output, &text).unwrap();

        assert_eq!(output, text.as_str().as_bytes());
    }
}
