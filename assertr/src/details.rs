use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use crate::{AssertThat, ChainRecords, mode::Mode};

pub(crate) trait WithDetail {
    fn collect_messages(&self, collection: &mut Vec<String>);
}

impl<T, M: Mode, R> WithDetail for AssertThat<'_, T, M, R> {
    fn collect_messages(&self, collection: &mut Vec<String>) {
        self.state.records.collect_messages(collection);
    }
}

impl WithDetail for ChainRecords<'_> {
    fn collect_messages(&self, collection: &mut Vec<String>) {
        for m in self.detail_messages.borrow().iter() {
            collection.push(m.to_owned());
        }
        if let Some(parent) = self.parent {
            parent.collect_messages(collection);
        }
    }
}

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Adds a message that is shown with every failure of this chain from here on.
    ///
    /// Messages are collected from the failing assertion upward through all of its parents, so a
    /// message set on the subject also shows up in failures of assertions derived from it through
    /// `satisfies` and friends. Use it for context that belongs to the test ("ids must be sorted").
    /// Assertion implementations attach the evidence of one particular failure through
    /// [`FailureBuilder::fact`](crate::failure::FailureBuilder::fact) instead.
    #[must_use]
    pub fn with_detail_message(self, message: impl Into<String>) -> Self {
        self.add_detail_message(message);
        self
    }

    /// Adds a message for subsequent failures when `condition` is true.
    ///
    /// `condition` is evaluated immediately. `message_provider` runs immediately only when the
    /// condition returns `true`.
    #[must_use]
    pub fn with_conditional_detail_message<Message: Into<String>>(
        self,
        condition: impl Fn(&Self) -> bool,
        message_provider: impl Fn(&Self) -> Message,
    ) -> Self {
        if condition(&self) {
            self.add_detail_message(message_provider(&self));
        }
        self
    }

    /// Adds a message for every subsequent failure of this chain.
    ///
    /// Unlike the `with_` variants, this method borrows the assertion. Assertion implementations
    /// must not use it for per-failure diagnostics. Those belong to
    /// [`FailureBuilder::fact`](crate::failure::FailureBuilder::fact), which scopes them to a
    /// single failure.
    pub fn add_detail_message(&self, message: impl Into<String>) {
        let message = message.into();
        self.state
            .records
            .detail_messages
            .borrow_mut()
            .push(message);
    }
}
