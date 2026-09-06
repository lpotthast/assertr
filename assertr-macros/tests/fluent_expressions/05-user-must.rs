struct UserType;

impl UserType {
    fn must(self) {}
}

#[renamed_assertr::fluent_expressions]
fn main() {
    UserType.must();
}
