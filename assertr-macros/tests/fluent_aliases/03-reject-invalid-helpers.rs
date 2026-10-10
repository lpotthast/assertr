#[assertr_macros::fluent_aliases]
trait ReadyAssertions {
    #[fluent_alias("be_set")]
    #[no_fluent_alias]
    fn is_ready(self) -> Self;

    #[fluent_alias("be_done")]
    #[cfg_attr(all(), fluent_alias("be_finished"))]
    fn is_done(self) -> Self;

    #[no_fluent_alias(always)]
    fn is_idle(self) -> Self;

    #[no_fluent_alias]
    const LIMIT: usize;

    // The automatic alias `be_open` collides with the method below.
    fn is_open(self) -> Self;

    fn be_open(self) -> Self;

    #[fluent_alias("is_closed")]
    fn is_closed(self) -> Self;
}

fn main() {}
