use assertr::prelude::*;

#[derive(Debug, PartialEq)]
struct Person {
    meta: Metadata,
}

#[derive(Debug, PartialEq)]
struct Metadata {
    alive: bool,
}

impl Person {
    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)]
    async fn get_metadata(&self) -> &Metadata {
        &self.meta
    }
}

#[tokio::test]
async fn is_able_to_access_derived_properties_without_breaking_the_call_chain() {
    let person = Person {
        meta: Metadata { alive: true },
    };

    assert_that!(person)
        .derive_async(Person::get_metadata)
        .await
        .derive(|it| &it.alive)
        .is_equal_to(true);
}
