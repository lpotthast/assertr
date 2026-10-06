//! Naming rules for automatically generated fluent aliases.

/// Method prefixes whose verbs are already imperative and therefore need no separate alias.
const ALREADY_FLUENT_PREFIXES: [&str; 1] = ["get_"];

/// Namespace prefixes that are kept verbatim. The alias rule applies to the remainder
/// (`into_iter_contains` -> `into_iter_contain`, `into_iter_is_empty` -> `into_iter_be_empty`).
const PASSTHROUGH_PREFIXES: [&str; 1] = ["into_iter_"];

/// Derives an imperative alias from the assertion method's third-person verb.
///
/// Negated methods put `not` first in their alias, matching the English imperative ("must not be
/// equal to", "must not have changed"): `is_not_*` becomes `not_be_*`, `has_not_*` becomes
/// `not_have_*`, and `does_not_*` becomes `not_*`. The possessive `has_no_*` keeps its word order
/// as `have_no_*` ("must have no remaining elements"). The negated prefixes are matched before
/// their positive counterparts so `is_not_empty` never degrades to `be_not_empty`. A namespace
/// prefix from [`PASSTHROUGH_PREFIXES`] is kept in front of the derived alias. Methods beginning
/// with a prefix from [`ALREADY_FLUENT_PREFIXES`] retain their original spelling without a second
/// alias. Returns `None` for such names and for names outside these rules.
pub(super) fn automatic_alias(name: &str) -> Option<String> {
    if ALREADY_FLUENT_PREFIXES
        .into_iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return None;
    }

    if let Some(namespace) = PASSTHROUGH_PREFIXES
        .into_iter()
        .find(|prefix| name.starts_with(prefix))
    {
        return automatic_alias(&name[namespace.len()..])
            .map(|alias| format!("{namespace}{alias}"));
    }

    // `match` is a keyword and `be_matching` belongs to `is_matching`, so `matches` gets the
    // explicit `match_expectation`.
    let exact = match name {
        "matches" => Some("match_expectation"),
        "contains" => Some("contain"),
        "exists" => Some("exist"),
        "panics" => Some("panic"),
        "satisfies" => Some("satisfy"),
        _ => None,
    };
    if let Some(alias) = exact {
        return Some(alias.to_owned());
    }

    let (prefix, replacement) = [
        ("does_not_", "not_"),
        ("is_not_", "not_be_"),
        ("has_not_", "not_have_"),
        ("is_", "be_"),
        ("has_", "have_"),
        ("contains_", "contain_"),
        ("starts_", "start_"),
        ("ends_", "end_"),
        ("exists_", "exist_"),
        ("needs_", "need_"),
        ("panics_", "panic_"),
        ("satisfies_", "satisfy_"),
    ]
    .into_iter()
    .find(|(prefix, _)| name.starts_with(prefix))?;

    Some(format!("{replacement}{}", &name[prefix.len()..]))
}

#[cfg(test)]
mod tests {
    use renamed_assertr::prelude::*;

    use super::automatic_alias;

    #[test]
    fn derives_supported_aliases() {
        assert_that!(automatic_alias("is_empty").as_deref()).is_equal_to(Some("be_empty"));
        assert_that!(automatic_alias("has_length").as_deref()).is_equal_to(Some("have_length"));
        assert_that!(automatic_alias("does_not_contain").as_deref())
            .is_equal_to(Some("not_contain"));
        assert_that!(automatic_alias("contains_value").as_deref())
            .is_equal_to(Some("contain_value"));
        assert_that!(automatic_alias("matches").as_deref()).is_equal_to(Some("match_expectation"));
        assert_that!(automatic_alias("contains").as_deref()).is_equal_to(Some("contain"));
        assert_that!(automatic_alias("starts_with").as_deref()).is_equal_to(Some("start_with"));
        assert_that!(automatic_alias("ends_with").as_deref()).is_equal_to(Some("end_with"));
        assert_that!(automatic_alias("exists_in").as_deref()).is_equal_to(Some("exist_in"));
        assert_that!(automatic_alias("exists").as_deref()).is_equal_to(Some("exist"));
        assert_that!(automatic_alias("satisfies_all").as_deref()).is_equal_to(Some("satisfy_all"));
        assert_that!(automatic_alias("satisfies").as_deref()).is_equal_to(Some("satisfy"));
    }

    #[test]
    fn puts_not_first_in_negated_aliases() {
        assert_that!(automatic_alias("is_not_equal_to").as_deref())
            .is_equal_to(Some("not_be_equal_to"));
        assert_that!(automatic_alias("is_not_empty").as_deref()).is_equal_to(Some("not_be_empty"));
        assert_that!(automatic_alias("has_not_changed").as_deref())
            .is_equal_to(Some("not_have_changed"));
        assert_that!(automatic_alias("does_not_exist").as_deref()).is_equal_to(Some("not_exist"));
        assert_that!(automatic_alias("does_not_panic_async").as_deref())
            .is_equal_to(Some("not_panic_async"));
    }

    #[test]
    fn derives_verb_aliases_for_panics_and_needs() {
        assert_that!(automatic_alias("panics").as_deref()).is_equal_to(Some("panic"));
        assert_that!(automatic_alias("panics_async").as_deref()).is_equal_to(Some("panic_async"));
        assert_that!(automatic_alias("needs_drop").as_deref()).is_equal_to(Some("need_drop"));
    }

    #[test]
    fn keeps_namespace_prefixes_in_front_of_the_alias() {
        assert_that!(automatic_alias("into_iter_contains").as_deref())
            .is_equal_to(Some("into_iter_contain"));
        assert_that!(automatic_alias("into_iter_is_empty").as_deref())
            .is_equal_to(Some("into_iter_be_empty"));
        assert_that!(automatic_alias("into_iter_is_not_empty").as_deref())
            .is_equal_to(Some("into_iter_not_be_empty"));
        assert_that!(automatic_alias("into_iter_has_length").as_deref())
            .is_equal_to(Some("into_iter_have_length"));
        assert_that!(automatic_alias("into_iter_does_not_contain_matching").as_deref())
            .is_equal_to(Some("into_iter_not_contain_matching"));
        assert_that!(automatic_alias("into_iter_starts_with").as_deref())
            .is_equal_to(Some("into_iter_start_with"));
        assert_that!(automatic_alias("into_iter_map")).is_none();
    }

    #[test]
    fn leaves_already_imperative_names_without_a_second_alias() {
        assert_that!(automatic_alias("get_some")).is_none();
        assert_that!(automatic_alias("get_json")).is_none();
    }

    #[test]
    fn keeps_the_word_order_of_possessive_negations() {
        assert_that!(automatic_alias("has_no_remaining_elements").as_deref())
            .is_equal_to(Some("have_no_remaining_elements"));
    }

    #[test]
    fn leaves_unsupported_names_without_an_alias() {
        assert_that!(automatic_alias("map")).is_none();
    }
}
