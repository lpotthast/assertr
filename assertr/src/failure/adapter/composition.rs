use core::{convert::Infallible, fmt};

use super::Adapter;
use crate::AssertionFailure;

/// A linear composition that passes the first adapter's output to the next adapter.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct Then<First, Next> {
    first: First,
    next: Next,
}

impl<First, Next> Then<First, Next> {
    /// Creates a linear composition.
    pub const fn new(first: First, next: Next) -> Self {
        Self { first, next }
    }
}

impl<Input: ?Sized, First, Next> Adapter<Input> for Then<First, Next>
where
    First: Adapter<Input>,
    Next: Adapter<First::Output>,
{
    type Output = Next::Output;
    type Error = ThenError<First::Error, Next::Error>;

    fn adapt(&self, input: &Input) -> Result<Self::Output, Self::Error> {
        let intermediate = self.first.adapt(input).map_err(ThenError::First)?;
        self.next.adapt(&intermediate).map_err(ThenError::Next)
    }
}

/// Identifies which stage of a [`Then`] or [`Tap`] composition failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThenError<FirstError, NextError> {
    /// The first adapter failed, so the next adapter was not run.
    First(FirstError),
    /// The next adapter failed after the first adapter succeeded.
    Next(NextError),
}

impl<FirstError: fmt::Display, NextError: fmt::Display> fmt::Display
    for ThenError<FirstError, NextError>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::First(error) => write!(f, "first adapter failed: {error}"),
            Self::Next(error) => write!(f, "next adapter failed: {error}"),
        }
    }
}

/// A composition that runs a sink on an intermediate output and then preserves that output.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct Tap<Adapter, Sink> {
    adapter: Adapter,
    sink: Sink,
}

impl<Adapter, Sink> Tap<Adapter, Sink> {
    /// Creates a tap composition.
    pub const fn new(adapter: Adapter, sink: Sink) -> Self {
        Self { adapter, sink }
    }
}

impl<Input: ?Sized, A, Sink> Adapter<Input> for Tap<A, Sink>
where
    A: Adapter<Input>,
    Sink: Adapter<A::Output, Output = ()>,
{
    type Output = A::Output;
    type Error = ThenError<A::Error, Sink::Error>;

    fn adapt(&self, input: &Input) -> Result<Self::Output, Self::Error> {
        let output = self.adapter.adapt(input).map_err(ThenError::First)?;
        self.sink.adapt(&output).map_err(ThenError::Next)?;
        Ok(output)
    }
}

/// Sends the same input to two adapters in deterministic left-to-right order.
///
/// Both branches are attempted even when the left branch returns an error.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct FanOut<Left, Right> {
    left: Left,
    right: Right,
}

impl<Left, Right> FanOut<Left, Right> {
    /// Creates a two-branch fan-out.
    pub const fn new(left: Left, right: Right) -> Self {
        Self { left, right }
    }
}

impl<Input: ?Sized, Left, Right> Adapter<Input> for FanOut<Left, Right>
where
    Left: Adapter<Input>,
    Right: Adapter<Input>,
{
    type Output = (Left::Output, Right::Output);
    type Error = FanOutError<Left::Error, Right::Error>;

    fn adapt(&self, input: &Input) -> Result<Self::Output, Self::Error> {
        let left = self.left.adapt(input);
        let right = self.right.adapt(input);

        match (left, right) {
            (Ok(left), Ok(right)) => Ok((left, right)),
            (Err(error), Ok(_)) => Err(FanOutError::Left(error)),
            (Ok(_), Err(error)) => Err(FanOutError::Right(error)),
            (Err(left), Err(right)) => Err(FanOutError::Both { left, right }),
        }
    }
}

/// Aggregates errors from a two-branch [`FanOut`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FanOutError<LeftError, RightError> {
    /// Only the left branch failed.
    Left(LeftError),
    /// Only the right branch failed.
    Right(RightError),
    /// Both branches failed.
    Both {
        /// The left branch's error.
        left: LeftError,
        /// The right branch's error.
        right: RightError,
    },
}

impl<LeftError: fmt::Display, RightError: fmt::Display> fmt::Display
    for FanOutError<LeftError, RightError>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left(error) => write!(f, "left branch failed: {error}"),
            Self::Right(error) => write!(f, "right branch failed: {error}"),
            Self::Both { left, right } => {
                write!(f, "both branches failed: left: {left}; right: {right}")
            }
        }
    }
}

/// The empty branch set used by a newly created [`FailurePipeline`].
#[derive(Clone, Copy, Debug, Default)]
pub struct NoBranches;

impl<Input: ?Sized> Adapter<Input> for NoBranches {
    type Output = ();
    type Error = Infallible;

    fn adapt(&self, _input: &Input) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

/// A failure-rooted pipeline with one retained primary output and independent side branches.
///
/// Every branch receives the original [`AssertionFailure`]. The primary adapter runs first, then
/// side branches run in the order they were added. A primary error does not prevent the side
/// branches from being attempted.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct FailurePipeline<Primary, Branches = NoBranches> {
    primary: Primary,
    branches: Branches,
}

impl<Primary> FailurePipeline<Primary, NoBranches> {
    /// Creates a pipeline whose primary adapter determines the retained output.
    pub const fn new(primary: Primary) -> Self {
        Self {
            primary,
            branches: NoBranches,
        }
    }
}

impl<Primary, Branches> FailurePipeline<Primary, Branches> {
    /// Adds an independent side branch that receives the original assertion failure.
    pub fn branch<Side>(self, side: Side) -> FailurePipeline<Primary, FanOut<Branches, Side>> {
        FailurePipeline {
            primary: self.primary,
            branches: FanOut::new(self.branches, side),
        }
    }
}

impl<Primary, Branches> Adapter<AssertionFailure> for FailurePipeline<Primary, Branches>
where
    Primary: Adapter<AssertionFailure>,
    Branches: Adapter<AssertionFailure>,
{
    type Output = Primary::Output;
    type Error = FailurePipelineError<Primary::Error, Branches::Error>;

    fn adapt(&self, failure: &AssertionFailure) -> Result<Self::Output, Self::Error> {
        let primary = self.primary.adapt(failure);
        let branches = self.branches.adapt(failure);

        match (primary, branches) {
            (Ok(primary), Ok(_)) => Ok(primary),
            (Err(error), Ok(_)) => Err(FailurePipelineError::Primary(error)),
            (Ok(_), Err(error)) => Err(FailurePipelineError::Branch(error)),
            (Err(primary), Err(branch)) => Err(FailurePipelineError::Both { primary, branch }),
        }
    }
}

/// Aggregates errors from a [`FailurePipeline`]'s primary adapter and side branches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailurePipelineError<PrimaryError, BranchError> {
    /// Only the primary adapter failed.
    Primary(PrimaryError),
    /// Only a side branch failed.
    Branch(BranchError),
    /// Both the primary adapter and at least one side branch failed.
    Both {
        /// The primary adapter's error.
        primary: PrimaryError,
        /// The side branch error or aggregate error.
        branch: BranchError,
    },
}

impl<PrimaryError: fmt::Display, BranchError: fmt::Display> fmt::Display
    for FailurePipelineError<PrimaryError, BranchError>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primary(error) => write!(f, "primary adapter failed: {error}"),
            Self::Branch(error) => write!(f, "side branch failed: {error}"),
            Self::Both { primary, branch } => {
                write!(
                    f,
                    "primary and side branches failed: primary: {primary}; side: {branch}"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{rc::Rc, string::String, vec, vec::Vec};
    use core::cell::RefCell;

    use super::*;
    use crate::failure::adapter::AdapterExt;
    use crate::failure::{FailureBuilder, FailureKind};

    #[derive(Clone)]
    struct NumberStep {
        events: Rc<RefCell<Vec<&'static str>>>,
        name: &'static str,
        increment: u8,
        error: Option<&'static str>,
    }

    impl Adapter<u8> for NumberStep {
        type Output = u8;
        type Error = &'static str;

        fn adapt(&self, input: &u8) -> Result<Self::Output, Self::Error> {
            self.events.borrow_mut().push(self.name);
            self.error.map_or(Ok(*input + self.increment), Err)
        }
    }

    struct NumberSink {
        events: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Adapter<u8> for NumberSink {
        type Output = ();
        type Error = Infallible;

        fn adapt(&self, _input: &u8) -> Result<Self::Output, Self::Error> {
            self.events.borrow_mut().push("sink");
            Ok(())
        }
    }

    #[test]
    fn then_passes_the_output_forward_and_stops_at_the_first_error() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let first = NumberStep {
            events: Rc::clone(&events),
            name: "first",
            increment: 1,
            error: None,
        };
        let second = NumberStep {
            events: Rc::clone(&events),
            name: "second",
            increment: 2,
            error: None,
        };

        assert_eq!(first.clone().then(second).adapt(&3), Ok(6));
        assert_eq!(*events.borrow(), vec!["first", "second"]);

        events.borrow_mut().clear();
        let failing = NumberStep {
            error: Some("no value"),
            ..first
        };
        let result = failing
            .then(NumberStep {
                events: Rc::clone(&events),
                name: "not run",
                increment: 0,
                error: None,
            })
            .adapt(&3);

        assert_eq!(result, Err(ThenError::First("no value")));
        assert_eq!(*events.borrow(), vec!["first"]);
    }

    #[test]
    fn tap_runs_the_sink_and_preserves_the_intermediate_output() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let adapter = NumberStep {
            events: Rc::clone(&events),
            name: "adapter",
            increment: 4,
            error: None,
        }
        .tap(NumberSink {
            events: Rc::clone(&events),
        });

        assert_eq!(adapter.adapt(&3), Ok(7));
        assert_eq!(*events.borrow(), vec!["adapter", "sink"]);
    }

    #[test]
    fn fan_out_attempts_both_branches_and_aggregates_their_errors() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let branch = |name, error| NumberStep {
            events: Rc::clone(&events),
            name,
            increment: 0,
            error: Some(error),
        };

        let result =
            FanOut::new(branch("left", "left error"), branch("right", "right error")).adapt(&3);

        assert_eq!(
            result,
            Err(FanOutError::Both {
                left: "left error",
                right: "right error",
            })
        );
        assert_eq!(*events.borrow(), vec!["left", "right"]);
    }

    #[derive(Clone)]
    struct FailureStep {
        observations: Rc<RefCell<Vec<(&'static str, usize)>>>,
        name: &'static str,
        error: Option<&'static str>,
    }

    impl Adapter<AssertionFailure> for FailureStep {
        type Output = String;
        type Error = &'static str;

        fn adapt(&self, failure: &AssertionFailure) -> Result<Self::Output, Self::Error> {
            self.observations
                .borrow_mut()
                .push((self.name, core::ptr::from_ref(failure) as usize));
            self.error.map_or(Ok(self.name.into()), Err)
        }
    }

    #[test]
    fn failure_pipeline_attempts_every_root_branch_on_the_same_failure() {
        let observations = Rc::new(RefCell::new(Vec::new()));
        let step = |name, error| FailureStep {
            observations: Rc::clone(&observations),
            name,
            error,
        };
        let pipeline = FailurePipeline::new(step("primary", Some("primary error")))
            .branch(step("first branch", Some("branch error")))
            .branch(step("second branch", None));
        let failure = FailureBuilder::detached::<i32>(FailureKind::Equality)
            .actual(1)
            .expected(2)
            .build();

        assert!(pipeline.adapt(&failure).is_err());
        let observations = observations.borrow();
        assert_eq!(
            observations
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>(),
            vec!["primary", "first branch", "second branch"]
        );
        assert!(
            observations
                .iter()
                .all(|(_, address)| *address == core::ptr::from_ref(&failure) as usize)
        );
    }
}
