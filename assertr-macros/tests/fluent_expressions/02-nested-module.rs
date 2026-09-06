#[renamed_assertr::fluent_expressions]
mod tests {
    use renamed_assertr::prelude::*;

    pub(crate) fn direct() -> Option<&'static str> {
        let failures = 42.verify(|it| it.is_equal_to(43));
        failures[0].expression
    }

    pub(crate) mod nested {
        use super::*;

        pub(crate) fn capture() -> Option<&'static str> {
            let failures = 42.verify(|it| it.is_equal_to(43));
            failures[0].expression
        }
    }
}

fn main() {
    use renamed_assertr::prelude::*;

    assert_that!(tests::direct()).is_equal_to(Some("42"));
    assert_that!(tests::nested::capture()).is_equal_to(Some("42"));
}
