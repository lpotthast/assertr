#[renamed_assertr::fluent_expressions(krate = renamed_assertr)]
fn unknown_argument() {}

#[renamed_assertr::fluent_expressions(crate = "renamed_assertr")]
fn quoted_path() {}

#[renamed_assertr::fluent_expressions(crate = renamed_assertr, crate = renamed_assertr)]
fn duplicate_path() {}

fn main() {}
