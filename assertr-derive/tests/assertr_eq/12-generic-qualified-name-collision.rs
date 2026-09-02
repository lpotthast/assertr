use renamed_assertr::prelude::*;
use assertr_derive::AssertrEq;

mod named {
    #[derive(Debug, PartialEq)]
    pub struct T;
}

#[derive(Debug, AssertrEq)]
pub struct GenericNameCollision<T> {
    private: T,
    pub value: named::T,
}

fn main() {
    let subject = GenericNameCollision {
        private: 42,
        value: named::T,
    };

    subject.must().be_equal_to(GenericNameCollisionAssertrEq {
        value: eq(named::T),
    });
}
