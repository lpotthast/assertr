//! Naming rules for automatically generated fluent aliases.

/// Derives an imperative alias from the assertion method's third-person verb.
pub(super) fn automatic_alias(name: &str) -> Option<String> {
    match name {
        "contains" => return Some("contain".to_owned()),
        "exists" => return Some("exist".to_owned()),
        "satisfies" => return Some("satisfy".to_owned()),
        _ => {}
    }

    let (prefix, replacement) = [
        ("does_not_", "not_"),
        ("is_", "be_"),
        ("has_", "have_"),
        ("contains_", "contain_"),
        ("starts_", "start_"),
        ("ends_", "end_"),
        ("exists_", "exist_"),
        ("satisfies_", "satisfy_"),
    ]
    .into_iter()
    .find(|(prefix, _)| name.starts_with(prefix))?;

    Some(format!("{replacement}{}", &name[prefix.len()..]))
}

#[cfg(test)]
mod tests {
    use super::automatic_alias;

    #[test]
    fn derives_supported_aliases() {
        assert_eq!(automatic_alias("is_empty").as_deref(), Some("be_empty"));
        assert_eq!(
            automatic_alias("has_length").as_deref(),
            Some("have_length")
        );
        assert_eq!(
            automatic_alias("does_not_contain").as_deref(),
            Some("not_contain")
        );
        assert_eq!(
            automatic_alias("contains_value").as_deref(),
            Some("contain_value")
        );
        assert_eq!(automatic_alias("contains").as_deref(), Some("contain"));
        assert_eq!(
            automatic_alias("starts_with").as_deref(),
            Some("start_with")
        );
        assert_eq!(automatic_alias("ends_with").as_deref(), Some("end_with"));
        assert_eq!(automatic_alias("exists_in").as_deref(), Some("exist_in"));
        assert_eq!(automatic_alias("exists").as_deref(), Some("exist"));
        assert_eq!(
            automatic_alias("satisfies_all").as_deref(),
            Some("satisfy_all")
        );
        assert_eq!(automatic_alias("satisfies").as_deref(), Some("satisfy"));
    }

    #[test]
    fn leaves_unsupported_names_without_an_alias() {
        assert_eq!(automatic_alias("map"), None);
    }
}
