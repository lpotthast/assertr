use crate::AssertThat;
use crate::mode::Mode;
use crate::prelude::PartialEqAssertions;

pub trait ReqwestResponseAssertions {
    fn has_status_code(self, expected: reqwest::StatusCode) -> Self;
}

impl<M: Mode> ReqwestResponseAssertions for AssertThat<'_, reqwest::Response, M> {
    fn has_status_code(self, expected: reqwest::StatusCode) -> Self {
        self.derive(|it| it.status()).is_equal_to(expected);
        self
    }
}

#[cfg(test)]
mod tests {
    struct MockServer {
        server: mockito::ServerGuard,
    }

    impl MockServer {
        async fn new() -> Self {
            let mut server = mockito::Server::new_async().await;

            let _get_hello_mock = server
                .mock("GET", "/hello")
                .with_status(200)
                .with_header("content-type", "text/plain")
                .with_header("x-api-key", "1234")
                .with_body("world")
                .create();

            Self { server }
        }
    }

    mod has_status_code {
        use super::MockServer;
        use crate::assert_that;
        use crate::prelude::*;

        #[tokio::test]
        async fn succeeds_when_status_code_matches() {
            let server = MockServer::new().await;
            let response = reqwest::get(format!("{}/hello", server.server.url()))
                .await
                .unwrap();

            assert_that(response).has_status_code(reqwest::StatusCode::OK);
        }
    }
}
