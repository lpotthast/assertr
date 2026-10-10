//! Naming rules for automatically generated fluent aliases.

/// Prefixes replaced in front of the remaining name. Negated prefixes come before their positive
/// counterparts so `is_not_empty` never degrades to `be_not_empty`.
const PREFIXES: [(&str, &str); 5] = [
    ("does_not_", "not_"),
    ("is_not_", "not_be_"),
    ("has_not_", "not_have_"),
    ("is_", "be_"),
    ("has_", "have_"),
];

/// Third-person verbs and their imperative forms, applied to the whole name or its first word.
const VERBS: [(&str, &str); 7] = [
    ("contains", "contain"),
    ("ends", "end"),
    ("exists", "exist"),
    ("needs", "need"),
    ("panics", "panic"),
    ("satisfies", "satisfy"),
    ("starts", "start"),
];

/// Derives an imperative alias from the assertion method's third-person verb.
///
/// Negated methods put `not` first in their alias, matching the English imperative ("must not be
/// equal to", "must not have changed"): `is_not_*` becomes `not_be_*`, `has_not_*` becomes
/// `not_have_*`, and `does_not_*` becomes `not_*`. The possessive `has_no_*` keeps its word order
/// as `have_no_*` ("must have no remaining elements"). Returns `None` for names outside these
/// rules, for example extractions named after what they continue with, such as `some`, `first`, or
/// `json`.
pub(super) fn automatic_alias(name: &str) -> Option<String> {
    // `match` is a keyword and `be_matching` belongs to `is_matching`, so `matches` gets the
    // explicit `match_expectation`.
    if name == "matches" {
        return Some("match_expectation".to_owned());
    }

    if let Some((prefix, replacement)) = PREFIXES
        .into_iter()
        .find(|(prefix, _)| name.starts_with(prefix))
    {
        return Some(format!("{replacement}{}", &name[prefix.len()..]));
    }

    VERBS.into_iter().find_map(|(verb, imperative)| {
        let rest = name.strip_prefix(verb)?;
        (rest.is_empty() || rest.starts_with('_')).then(|| format!("{imperative}{rest}"))
    })
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
    fn leaves_extraction_names_without_an_alias() {
        assert_that!(automatic_alias("some")).is_none();
        assert_that!(automatic_alias("first")).is_none();
        assert_that!(automatic_alias("json")).is_none();
        assert_that!(automatic_alias("resolved_path")).is_none();
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
