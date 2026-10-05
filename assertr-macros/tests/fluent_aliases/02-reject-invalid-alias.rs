#[assertr_macros::fluent_aliases]
trait ReadyAssertions {
    #[fluent_alias("be ready")]
    fn is_ready(self) -> Self;

    #[fluent_alias("")]
    fn is_done(self) -> Self;
}

fn main() {}
