use alloc::vec::Vec;
use core::{fmt, ops::Deref, slice};

use super::AssertionFailure;
use crate::{
    assertions::{
        HasLength,
        collection::{Collection, RandomAccess, StableOrder},
    },
    renderer::CollectionPresentation,
};

/// The failures collected by one capture scope, in assertion order.
///
/// An empty collection means every assertion passed. Inspect the failures through slice access,
/// iteration, or ordinary collection assertions. `Display` and `Debug` produce the reports of all
/// failures, separated by an empty line.
#[derive(Clone, Default)]
#[must_use = "captured failures must be inspected"]
pub struct AssertionFailures {
    failures: Vec<AssertionFailure>,
    #[cfg(feature = "fluent")]
    pending_expression: Option<PendingExpression>,
}

/// Only failures from the fluent root are eligible. Child failures and explicit expressions
/// never enter this list. Entry locations distinguish nested verification calls.
#[cfg(feature = "fluent")]
#[derive(Clone)]
struct PendingExpression {
    location: &'static core::panic::Location<'static>,
    indexes: Vec<usize>,
}

impl PartialEq for AssertionFailures {
    fn eq(&self, other: &Self) -> bool {
        self.failures == other.failures
    }
}

impl Eq for AssertionFailures {}

impl AssertionFailures {
    pub(crate) const fn new() -> Self {
        Self {
            failures: Vec::new(),
            #[cfg(feature = "fluent")]
            pending_expression: None,
        }
    }

    pub(crate) fn push(
        &mut self,
        failure: AssertionFailure,
        #[cfg(feature = "fluent")] expression: Option<&'static core::panic::Location<'static>>,
    ) {
        #[cfg(feature = "fluent")]
        if let Some(location) = expression {
            let pending = self
                .pending_expression
                .get_or_insert_with(|| PendingExpression {
                    location,
                    indexes: Vec::new(),
                });
            debug_assert_eq!(
                pending.location, location,
                "failures belong to one fluent root"
            );
            pending.indexes.push(self.failures.len());
        }
        self.failures.push(failure);
    }

    #[cfg(feature = "fluent")]
    pub(crate) fn attach_expression(
        &mut self,
        expression: &'static str,
        location: &'static core::panic::Location<'static>,
    ) {
        if let Some(pending) = self.pending_expression.take()
            && pending.location == location
        {
            for index in pending.indexes {
                self.failures[index].expression = Some(expression);
            }
        }
    }

    /// Takes ownership of the collected failures.
    #[must_use]
    pub fn into_vec(self) -> Vec<AssertionFailure> {
        self.failures
    }
}

impl From<Vec<AssertionFailure>> for AssertionFailures {
    fn from(failures: Vec<AssertionFailure>) -> Self {
        Self {
            failures,
            #[cfg(feature = "fluent")]
            pending_expression: None,
        }
    }
}

impl From<AssertionFailure> for AssertionFailures {
    fn from(failure: AssertionFailure) -> Self {
        Self::from(alloc::vec![failure])
    }
}

impl Deref for AssertionFailures {
    type Target = [AssertionFailure];

    fn deref(&self) -> &Self::Target {
        &self.failures
    }
}

impl AsRef<[AssertionFailure]> for AssertionFailures {
    fn as_ref(&self) -> &[AssertionFailure] {
        &self.failures
    }
}

impl IntoIterator for AssertionFailures {
    type Item = AssertionFailure;
    type IntoIter = alloc::vec::IntoIter<AssertionFailure>;

    fn into_iter(self) -> Self::IntoIter {
        self.failures.into_iter()
    }
}

impl<'a> IntoIterator for &'a AssertionFailures {
    type Item = &'a AssertionFailure;
    type IntoIter = slice::Iter<'a, AssertionFailure>;

    fn into_iter(self) -> Self::IntoIter {
        self.failures.iter()
    }
}

impl HasLength for AssertionFailures {
    fn length(&self) -> usize {
        self.failures.len()
    }
}

impl Collection for AssertionFailures {
    type Item = AssertionFailure;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &Self::Item> {
        self.iter()
    }
}

impl StableOrder for AssertionFailures {}

impl RandomAccess for AssertionFailures {
    fn element_at(&self, index: usize) -> Option<&Self::Item> {
        self.get(index)
    }
}

impl fmt::Display for AssertionFailures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, failure) in self.failures.iter().enumerate() {
            if index != 0 {
                f.write_str("\n")?;
            }
            fmt::Display::fmt(failure, f)?;
        }
        Ok(())
    }
}

impl fmt::Debug for AssertionFailures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl core::error::Error for AssertionFailures {}

#[cfg(test)]
mod tests {
    use core::error::Error;

    use crate::prelude::*;

    #[test]
    fn aggregate_reports_join_the_single_reports() {
        let failures = assert_that!(1)
            .with_location(false)
            .capture(|it| it.is_equal_to(2).is_equal_to(3));
        let single = failures[0].to_string();
        assert_that!(format!("{:?}", failures[0])).is_equal_to(&single);
        let expected = format!("{single}\n{}", failures[1]);
        assert_that!(format!("{failures}")).is_equal_to(&expected);
        assert_that!(format!("{failures:?}")).is_equal_to(&expected);
        assert_that!(failures.source()).is_none();
        assert_that!(failures[0].source()).is_none();
        assert_that!(format!("{:?}", AssertionFailures::default())).is_equal_to("");
    }

    #[test]
    fn failures_support_collection_assertions_and_owned_conversion() {
        let failures = assert_that!(1).capture(|it| it.is_equal_to(2));
        let expected = failures[0].clone();
        assert_that!(failures)
            .has_length(1)
            .contains_exactly(core::slice::from_ref(&expected));
        assert_that!(failures).at(0).is_equal_to(expected.clone());
        assert_that!((&failures).into_iter().count()).is_equal_to(1);
        assert_that_owned!(&AssertionFailures::from(expected)).is_equal_to(&failures);
        assert_that_owned!(&AssertionFailures::from(failures.clone().into_vec()))
            .is_equal_to(&failures);
        assert_that!(failures.into_iter().count()).is_equal_to(1);
    }

    #[test]
    fn errors_propagate_through_ordinary_results_and_boxed_errors() {
        fn aggregate() -> Result<(), AssertionFailures> {
            let failures = assert_that!(1).capture(|it| it.is_equal_to(2));
            Err(failures)
        }
        fn boxed() -> Result<(), Box<dyn Error + Send + Sync>> {
            aggregate()?;
            Ok(())
        }
        assert_that!(boxed().unwrap_err().is::<AssertionFailures>()).is_true();
        let failure = aggregate().unwrap_err().into_iter().next().unwrap();
        let error: Box<dyn Error + Send + Sync> = failure.into();
        assert_that!(error.is::<AssertionFailure>()).is_true();
    }

    #[test]
    #[cfg(feature = "fluent")]
    fn pending_expressions_preserve_equality_cloning_and_owned_conversion() {
        let failures = 1.verify(|it| it.with_location(false).is_equal_to(2));
        assert_that!(failures[0].expression).is_none();
        assert_that_owned!(&AssertionFailures::from(failures.clone().into_vec()))
            .is_equal_to(&failures);
        assert_that_owned!(&AssertionFailures::from(failures[0].clone())).is_equal_to(&failures);
        assert_that!(failures.clone().into_iter().next().unwrap().expression).is_none();

        let location = failures.pending_expression.as_ref().unwrap().location;
        let mut cloned = failures.clone();
        cloned.attach_expression("receiver", location);
        assert_that!(cloned[0].expression).is_equal_to(Some("receiver"));
        assert_that!(failures[0].expression).is_none();
        assert_that!(cloned.pending_expression.is_none()).is_true();
        cloned.attach_expression("another receiver", location);
        assert_that!(cloned[0].expression).is_equal_to(Some("receiver"));
    }
}
