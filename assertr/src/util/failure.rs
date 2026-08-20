use alloc::{string::String, vec::Vec};

use crate::failure::BANNER;

/// Joins captured assertion failures for embedding into another assertion's detail message,
/// stripping the banner that surrounds each standalone rendered failure.
pub(crate) fn join_failures(failures: &[String]) -> String {
    failures
        .iter()
        .map(|failure| {
            let failure = failure.strip_prefix(BANNER).unwrap_or(failure);
            let failure = failure.strip_suffix(BANNER).unwrap_or(failure);
            failure.trim_matches('\n')
        })
        .collect::<Vec<_>>()
        .join("\n")
}
