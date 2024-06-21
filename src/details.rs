use std::fmt::Debug;

use crate::{mode::Mode, AssertThat};

pub(crate) struct DetailMessages<'a>(pub(crate) &'a [String]);

impl<'a> Debug for DetailMessages<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|it| DisplayString(it)))
            .finish()
    }
}

struct DisplayString<'a>(&'a str);

impl<'a> Debug for DisplayString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/*

1) Root Detail 1

    1a) Derived1 Detail 1
    1b) Derived1 Detail 2

    2) Mapped Detail 1

        2a) Derived2 Detail 1
        2b) Derived2 Detail 2
*/

pub(crate) trait WithDetail {
    fn collect_messages(&self, collection: &mut Vec<String>);
}

impl<'t, T, M: Mode> WithDetail for AssertThat<'t, T, M> {
    fn collect_messages(&self, collection: &mut Vec<String>) {
        let detail_messages = self.detail_messages.borrow();
        for m in detail_messages.iter() {
            collection.push(m.to_owned());
        }
        if let Some(parent) = self.parent {
            parent.collect_messages(collection);
        }
    }
}

impl<'t, T, M: Mode> AssertThat<'t, T, M> {
    /// Specify an additional messages to be displayed on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    pub fn with_detail_message(self, message: impl Into<String>) -> Self {
        self.detail_messages.borrow_mut().push(message.into());
        self
    }

    /// Specify an additional messages to be displayed on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    pub fn with_conditional_detail_message<Message: Into<String>>(
        self,
        condition: bool,
        message_provider: impl Fn(&Self) -> Message,
    ) -> Self {
        if condition {
            let message = message_provider(&self);
            self.detail_messages.borrow_mut().push(message.into());
        }
        self
    }

    pub fn add_detail_message<'a>(&self, message: String) {
        self.detail_messages.borrow_mut().push(message);
    }
}
