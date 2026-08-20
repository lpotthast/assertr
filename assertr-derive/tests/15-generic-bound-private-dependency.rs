use assertr::prelude::*;
use core::marker::PhantomData;

pub trait Related<U> {}

impl Related<u8> for i32 {}

#[derive(Debug, AssertrEq)]
pub struct InlineBound<T: Related<U>, U> {
    pub value: T,
    marker: PhantomData<U>,
}

#[derive(Debug, AssertrEq)]
pub struct WhereBound<T, U>
where
    T: Related<U>,
{
    pub value: T,
    marker: PhantomData<U>,
}

fn main() {
    let inline = InlineBound {
        value: 42_i32,
        marker: PhantomData::<u8>,
    };
    let _ = &inline.marker;
    inline.must().be_equal_to(InlineBoundAssertrEq {
        value: eq(42),
    });
    let _: InlineBoundAssertrEq<i32> = InlineBoundAssertrEq::default();

    let where_bound = WhereBound {
        value: 42_i32,
        marker: PhantomData::<u8>,
    };
    let _ = &where_bound.marker;
    where_bound.must().be_equal_to(WhereBoundAssertrEq {
        value: eq(42),
    });
    let _: WhereBoundAssertrEq<i32> = WhereBoundAssertrEq::default();
}
