use assertr::prelude::*;

#[test]
fn is_able_to_use_json_conversion() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Person {
        age: u32,
    }

    let person = Person { age: 42 };

    let expected = r#"{"age":42}"#;

    assert_that(&person).map(json()).is_equal_to(expected);
    assert_that(&person).as_json().is_equal_to(expected);
}

#[test]
fn is_able_to_use_toml_conversion() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Config {
        value: u32,
        list: Vec<u32>,
    }

    let config = Config {
        value: 42,
        list: vec![1, 2],
    };

    assert_that(&config)
        .map(toml())
        .is_equal_to(indoc::formatdoc! {r#"
        value = 42
        list = [1, 2]
    "#});

    assert_that(&config)
        .as_toml()
        .is_equal_to(indoc::formatdoc! {r#"
        value = 42
        list = [1, 2]
    "#});
}
