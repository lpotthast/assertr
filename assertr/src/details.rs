use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;

use crate::{mode::Mode, AssertThat};

pub(crate) struct DetailMessages<'a>(pub(crate) &'a [String]);

impl<'a> Debug for DetailMessages<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|it| DisplayString(it)))
            .finish()
    }
}

pub(crate) struct DisplayString<'a>(pub(crate) &'a str);

impl<'a> Debug for DisplayString<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}

pub(crate) trait WithDetail {
    fn collect_messages(&self, collection: &mut Vec<String>);
}

impl<'t, T, M: Mode> WithDetail for AssertThat<'t, T, M> {
    fn collect_messages(&self, collection: &mut Vec<String>) {
        for m in self.detail_messages.borrow().iter() {
            collection.push(m.to_owned());
        }
        if let Some(parent) = self.parent {
            parent.collect_messages(collection);
        }
    }
}

impl<'t, T, M: Mode> AssertThat<'t, T, M> {
    /// Add a message to be displayed on assertion failure.
    pub fn with_detail_message(self, message: impl Into<String>) -> Self {
        self.detail_messages.borrow_mut().push(message.into());
        self
    }

    /// Add a message to be displayed on assertion failure bound by the given condition.
    pub fn with_conditional_detail_message<Message: Into<String>>(
        self,
        condition: impl Fn(&Self) -> bool,
        message_provider: impl Fn(&Self) -> Message,
    ) -> Self {
        if condition(&self) {
            let message = message_provider(&self);
            self.detail_messages.borrow_mut().push(message.into());
        }
        self
    }

    /// Add a message to be displayed on assertion failure.
    ///
    /// Use this variant instead of the `with_` variants when not in a call-chain context,
    /// and you don't want to call an ownership-taking function.
    pub fn add_detail_message(&self, message: impl Into<String>) {
        self.detail_messages.borrow_mut().push(message.into());
    }
}
