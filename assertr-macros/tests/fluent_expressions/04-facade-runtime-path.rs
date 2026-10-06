use renamed_assertr::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static FINISHED: AtomicUsize = AtomicUsize::new(0);

/// Emulates a facade crate that re-exports the runtime with `pub use assertr;`.
mod my_facade {
    pub mod assertr {
        pub use renamed_assertr::*;

        /// Counts completed entries, proving that generated code resolves the given path rather than
        /// the dependency found in the manifest.
        pub mod __private {
            pub mod fluent_expressions {
                pub use renamed_assertr::__private::fluent_expressions::*;

                #[track_caller]
                pub fn finish<T>(
                    result: T,
                    attach: impl FnOnce(T, &'static core::panic::Location<'static>) -> T,
                ) -> T {
                    crate::FINISHED.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                    renamed_assertr::__private::fluent_expressions::finish(result, attach)
                }
            }
        }
    }
}

#[my_facade::assertr::fluent_expressions(crate = crate::my_facade::assertr)]
mod checks {
    use super::*;

    pub fn run() {
        let failures = 42.verify(|it| it.is_equal_to(43));
        assert_that!(failures[0].expression).is_equal_to(Some("42"));
        let failures = 42.verify_owned(|it| it.is_equal_to(42));
        assert_that!(failures).is_empty();
        42.must().is_equal_to(42);
    }
}

fn main() {
    checks::run();
    assert_that!(FINISHED.load(Ordering::Relaxed)).is_equal_to(2);
}
