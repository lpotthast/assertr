#[assertr_macros::fluent_aliases]
trait ReadyAssertions {
    #[fluent_alias(be_set)]
    fn is_ready(self) -> Self;

    #[cfg_attr(all(), fluent_alias(be_done))]
    fn is_done(self) -> Self;
}

fn main() {}
