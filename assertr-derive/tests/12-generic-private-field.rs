use assertr::prelude::*;

#[derive(Debug, AssertrEq)]
pub struct GenericPrivate<T> {
    private: T,
    pub id: u32,
}

fn main() {
    let subject = GenericPrivate {
        private: "not part of the matcher",
        id: 42,
    };

    subject
        .must()
        .be_equal_to(GenericPrivateAssertrEq { id: eq(42) });
}
