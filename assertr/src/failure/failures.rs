use alloc::vec::Vec;
use core::{fmt, ops::Deref, slice};

use super::{
    AssertionFailure,
    adapter::{Adapter, ToHumanReadableText},
};
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
/// iteration, or ordinary collection assertions. `Display` and `Debug` produce the default plain
/// report, while adapters can process each failure or the aggregate explicitly.
#[derive(Clone, Default)]
#[must_use = "captured failures must be inspected"]
pub struct AssertionFailures {
    failures: Vec<AssertionFailure>,
    #[cfg(feature = "fluent")]
    pending_expression: Option<PendingExpression>,
}

/// Only failures from the fluent root are eligible. Child failures and explicit expressions
/// never enter this list. Entry locations distinguish nested verification calls after the macro
/// checks that the original callback accepts an assertion chain.
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

    /// Borrows the collected failures in assertion order.
    #[must_use]
    pub fn as_slice(&self) -> &[AssertionFailure] {
        &self.failures
    }

    /// Returns the number of collected failures.
    #[must_use]
    pub fn len(&self) -> usize {
        self.failures.len()
    }

    /// Returns whether all assertions passed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
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
        self.as_slice()
    }
}

impl AsRef<[AssertionFailure]> for AssertionFailures {
    fn as_ref(&self) -> &[AssertionFailure] {
        self.as_slice()
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
        self.len()
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
        match ToHumanReadableText.adapt(self) {
            Ok(report) => fmt::Display::fmt(&report, f),
            Err(never) => match never {},
        }
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
    use crate::failure::adapter::{Adapter, AdapterExt};
    use crate::prelude::*;
    use core::error::Error;

    #[test]
    fn reports_use_the_default_adapter_for_single_and_aggregate_failures() {
        let failures = assert_that!(1)
            .with_location(false)
            .capture(|it| it.is_equal_to(2).is_equal_to(3));
        let single = ToHumanReadableText.render(&failures[0]);
        assert_eq!(format!("{}", failures[0]), single.as_str());
        assert_eq!(format!("{:?}", failures[0]), single.as_str());
        let expected = format!("{}\n{}", single, ToHumanReadableText.render(&failures[1]));
        assert_eq!(format!("{failures}"), expected);
        assert_eq!(format!("{failures:?}"), expected);
        assert_eq!(
            ToHumanReadableText.adapt(&failures).unwrap().as_str(),
            expected
        );
        assert_eq!(
            ToHumanReadableText
                .map_err(|never| never)
                .adapt(&failures)
                .unwrap()
                .as_str(),
            expected
        );
        assert!(failures.source().is_none());
        assert!(failures[0].source().is_none());
        assert_eq!(format!("{:?}", AssertionFailures::default()), "");
    }

    #[test]
    fn failures_support_collection_assertions_and_owned_conversion() {
        let failures = assert_that!(1).capture(|it| it.is_equal_to(2));
        let expected = failures[0].clone();
        assert_that!(failures)
            .has_length(1)
            .contains_exactly(core::slice::from_ref(&expected));
        assert_that!(failures)
            .get_at(0)
            .is_equal_to(expected.clone());
        assert_eq!(failures.as_ref(), failures.as_slice());
        assert_eq!((&failures).into_iter().count(), 1);
        assert_eq!(AssertionFailures::from(expected), failures);
        assert_eq!(
            AssertionFailures::from(failures.clone().into_vec()),
            failures
        );
        assert_eq!(failures.into_iter().count(), 1);
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
        assert!(boxed().unwrap_err().is::<AssertionFailures>());
        let failure = aggregate().unwrap_err().into_iter().next().unwrap();
        let error: Box<dyn Error + Send + Sync> = failure.into();
        assert!(error.is::<AssertionFailure>());
    }

    #[test]
    #[cfg(feature = "fluent")]
    fn pending_expressions_preserve_equality_cloning_and_owned_conversion() {
        let failures = 1.verify(|it| it.with_location(false).is_equal_to(2));
        assert_eq!(failures[0].expression, None);
        assert_eq!(
            AssertionFailures::from(failures.clone().into_vec()),
            failures
        );
        assert_eq!(AssertionFailures::from(failures[0].clone()), failures);
        assert_eq!(
            failures.clone().into_iter().next().unwrap().expression,
            None
        );

        let location = failures.pending_expression.as_ref().unwrap().location;
        let mut cloned = failures.clone();
        cloned.attach_expression("receiver", location);
        assert_eq!(cloned[0].expression, Some("receiver"));
        assert_eq!(failures[0].expression, None);
        assert!(cloned.pending_expression.is_none());
        cloned.attach_expression("another receiver", location);
        assert_eq!(cloned[0].expression, Some("receiver"));
    }
}
