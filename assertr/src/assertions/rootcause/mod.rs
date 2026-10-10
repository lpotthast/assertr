//! Assertions and length adapters for `rootcause` reports and report collections.

/// Assertions and extraction for reports.
pub mod report;

use rootcause::{report_attachments::ReportAttachments, report_collection::ReportCollection};

use crate::assertions::HasLength;

impl<C: ?Sized, T> HasLength for ReportCollection<C, T> {
    fn length(&self) -> usize {
        ReportCollection::len(self)
    }

    fn is_empty(&self) -> bool {
        ReportCollection::is_empty(self)
    }
}

impl<T> HasLength for ReportAttachments<T> {
    fn length(&self) -> usize {
        ReportAttachments::len(self)
    }

    fn is_empty(&self) -> bool {
        ReportAttachments::is_empty(self)
    }
}
