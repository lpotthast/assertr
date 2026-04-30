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
    #[allow(clippy::unused_async)]
    async fn into_metadata(self) -> Metadata {
        self.meta
    }
}

#[tokio::test]
async fn is_able_to_access_derived_properties_without_breaking_the_call_chain() {
    let person = Person {
        meta: Metadata { alive: true },
    };

    assert_that!(person)
        .map_async(|it| it.unwrap_owned().into_metadata())
        .await
        .map(|it| it.borrowed().alive.into())
        .is_equal_to(true);
}
