use alloc::{string::String, vec::Vec};

use crate::{AssertThat, ChainRecords, mode::Mode};

impl ChainRecords<'_> {
    /// Appends the messages shown with a failure raised on this chain.
    ///
    /// Local messages come first, in insertion order, followed by the messages inherited when
    /// `capture` detached this chain, and finally the messages of every linked ancestor.
    pub(crate) fn collect_messages(&self, collection: &mut Vec<String>) {
        collection.extend(self.detail_messages.borrow().iter().cloned());
        collection.extend(self.inherited_messages.iter().cloned());
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
    /// `condition` is evaluated immediately and exactly once. `message_provider` runs immediately,
    /// at most once, and only when the condition returns `true`.
    #[must_use]
    pub fn with_conditional_detail_message<Message: Into<String>>(
        self,
        condition: impl FnOnce(&Self) -> bool,
        message_provider: impl FnOnce(&Self) -> Message,
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

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::prelude::*;

    mod with_conditional_detail_message {
        use super::*;

        #[test]
        fn accepts_closures_that_consume_their_captures() {
            let condition = String::from("add");
            let message = String::from("consumed");

            let failures = assert_that!(1)
                .with_conditional_detail_message(move |_| condition == "add", move |_| message)
                .capture(|it| it.is_equal_to(2));

            assert_that!(failures[0].messages).contains_exactly(["consumed"]);
        }

        #[test]
        fn skips_the_message_provider_when_the_condition_is_false() {
            let failures = assert_that!(1)
                .with_conditional_detail_message(
                    |_| false,
                    |_| -> String { panic!("the provider must not run") },
                )
                .capture(|it| it.is_equal_to(2));

            assert_that!(failures[0].messages).is_empty();
        }
    }
}
