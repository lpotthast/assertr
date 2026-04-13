pub mod report;

use crate::assertions::HasLength;
use rootcause::report_attachments::ReportAttachments;
use rootcause::report_collection::ReportCollection;

impl<C: ?Sized, T> HasLength for ReportCollection<C, T> {
    fn length(&self) -> usize {
        ReportCollection::len(self)
    }

    fn is_empty(&self) -> bool {
        ReportCollection::is_empty(self)
    }
}

impl<C: ?Sized, T> HasLength for &ReportCollection<C, T> {
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

impl<T> HasLength for &ReportAttachments<T> {
    fn length(&self) -> usize {
        ReportAttachments::len(self)
    }

    fn is_empty(&self) -> bool {
        ReportAttachments::is_empty(self)
    }
}

pub mod prelude {
    pub use super::report::RootcauseDynamicReportAssertions;
    pub use super::report::RootcauseDynamicReportExtractAssertions;
    pub use super::report::RootcauseDynamicReportRefAssertions;
    pub use super::report::RootcauseDynamicReportRefExtractAssertions;
    pub use super::report::RootcauseReportAssertions;
    pub use super::report::RootcauseReportRefAssertions;
}
