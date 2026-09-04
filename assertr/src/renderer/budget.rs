/// Limits the amount of diagnostic output produced by one assertion chain.
///
/// The default allows 256 items in each repeated diagnostic group and retains 4,096 characters
/// from each rendered leaf value. A group can be a collection, a map, or per-item assertion
/// evidence. The leaf limit applies separately to every collection element and map key or value.
/// An opaque value counts as one leaf, so the limit can truncate its whole representation. It is
/// not a total character limit for the collection or failure. Truncated output always includes the
/// number of omitted items or characters. Use [`RenderingBudget::unlimited`] to retain complete
/// output regardless of size.
///
/// ```
/// use assertr::prelude::*;
///
/// let failures = assert_that!([123_456, 234_567, 345_678])
///     .with_rendering_budget(
///         RenderingBudget::builder()
///             .max_items(2)
///             .max_leaf_characters(3)
///             .build(),
///     )
///     .with_location(false)
///     .capture(|it| it.contains(0));
///
/// assert!(ToHumanReadableText.render(&failures[0]).contains(concat!(
///         "Actual: [\n",
///         "    123... 3 more characters ...,\n",
///         "    234... 3 more characters ...,\n",
///         "] (... 1 more element ...)\n",
///         "\n",
///         "does not contain\n",
///         "\n",
///         "Expected: 0\n",
///     )));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderingBudget {
    /// Maximum number of items rendered from one repeated diagnostic group.
    max_items: usize,

    /// Maximum number of characters retained from one rendered leaf value.
    max_leaf_characters: usize,
}

impl RenderingBudget {
    /// The default rendering budget.
    pub const DEFAULT: Self = Self {
        max_items: 256,
        max_leaf_characters: 4_096,
    };

    /// Creates a builder initialized with the default limits.
    #[must_use]
    pub const fn builder() -> RenderingBudgetBuilder {
        RenderingBudgetBuilder {
            budget: Self::DEFAULT,
        }
    }

    /// Creates a budget that never truncates rendering output.
    #[must_use]
    pub const fn unlimited() -> Self {
        Self {
            max_items: usize::MAX,
            max_leaf_characters: usize::MAX,
        }
    }

    /// Returns the maximum items rendered from one repeated diagnostic group.
    #[must_use]
    pub const fn max_items(self) -> usize {
        self.max_items
    }

    /// Returns the maximum characters retained from one rendered leaf value.
    #[must_use]
    pub const fn max_leaf_characters(self) -> usize {
        self.max_leaf_characters
    }
}

impl Default for RenderingBudget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Builds a [`RenderingBudget`] with named setters for either limit.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderingBudgetBuilder {
    budget: RenderingBudget,
}

impl RenderingBudgetBuilder {
    /// Sets the maximum items rendered from one repeated diagnostic group.
    #[must_use]
    pub const fn max_items(mut self, maximum: usize) -> Self {
        self.budget.max_items = maximum;
        self
    }

    /// Sets the maximum characters retained from each rendered leaf value.
    #[must_use]
    pub const fn max_leaf_characters(mut self, maximum: usize) -> Self {
        self.budget.max_leaf_characters = maximum;
        self
    }

    /// Returns the configured rendering budget.
    #[must_use]
    pub const fn build(self) -> RenderingBudget {
        self.budget
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    mod builder {
        use super::*;

        #[test]
        fn uses_defaults_for_unchanged_limits() {
            assert_that!(RenderingBudget::builder().build())
                .is_equal_to(RenderingBudget::default());
        }

        #[test]
        fn sets_each_named_limit() {
            let budget = RenderingBudget::builder()
                .max_items(17)
                .max_leaf_characters(29)
                .build();

            assert_that!(budget.max_items()).is_equal_to(17);
            assert_that!(budget.max_leaf_characters()).is_equal_to(29);
        }
    }

    mod unlimited {
        use super::*;

        #[test]
        fn sets_both_limits_to_the_largest_value() {
            let budget = RenderingBudget::unlimited();

            assert_that!(budget.max_items()).is_equal_to(usize::MAX);
            assert_that!(budget.max_leaf_characters()).is_equal_to(usize::MAX);
        }
    }
}
