use alloc::{string::String, vec::Vec};

use crate::{failure::BANNER, renderer::omission};

/// Joins captured assertion failures for embedding into another assertion's detail message,
/// stripping the banner that surrounds each standalone rendered failure.
pub(crate) fn join_failures(failures: &[String], maximum: usize) -> String {
    let mut joined = failures
        .iter()
        .take(maximum)
        .map(|failure| {
            let failure = failure.strip_prefix(BANNER).unwrap_or(failure);
            let failure = failure.strip_suffix(BANNER).unwrap_or(failure);
            failure.trim_matches('\n')
        })
        .collect::<Vec<_>>()
        .join("\n");

    let omitted = failures.len().saturating_sub(maximum);
    if omitted != 0 {
        if !joined.is_empty() {
            joined.push('\n');
        }
        joined.push_str(&omission(omitted, "assertion failure"));
    }
    joined
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::join_failures;

    #[test]
    fn limits_captured_failures_and_reports_the_omitted_count() {
        let failures = [
            String::from("one"),
            String::from("two"),
            String::from("three"),
        ];

        assert_that!(join_failures(&failures, 1))
            .is_equal_to("one\n... 2 more assertion failures ...");
        assert_that!(join_failures(&failures, 0)).is_equal_to("... 3 more assertion failures ...");
    }
}
