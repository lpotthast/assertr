use renamed_assertr::prelude::*;
use assertr_derive::AssertrEq;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub struct __AssertrRenderer0;

#[derive(Debug, PartialEq)]
pub struct Field0Renderer;

#[derive(Debug, AssertrEq)]
pub struct RendererNameCollision {
    pub value: __AssertrRenderer0,
}

#[derive(Debug, AssertrEq)]
pub struct FieldRendererNameCollision {
    pub value: Field0Renderer,
}

fn main() {
    let subject = RendererNameCollision {
        value: __AssertrRenderer0,
    };

    subject
        .must()
        .be_equal_to(RendererNameCollisionAssertrEq {
            value: eq(__AssertrRenderer0),
        });

    let subject = FieldRendererNameCollision {
        value: Field0Renderer,
    };

    subject
        .must()
        .be_equal_to(FieldRendererNameCollisionAssertrEq {
            value: eq(Field0Renderer),
        });
}
