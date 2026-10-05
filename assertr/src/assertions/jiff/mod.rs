//! Assertions for `jiff` durations, spans, and zoned date-times.

/// Generates the `IsZero`, `IsNegative`, and `IsPositive` expectations for a signed `jiff` type.
///
/// `present` optionally wraps every rendered value, for example to keep a compact form.
macro_rules! sign_expectations {
    (subject: $subject:ident, zero: $zero:expr $(, present: $present:path)? $(,)?) => {
        #[doc = concat!("Checks whether a `", stringify!($subject), "` is zero.")]
        pub struct IsZero;
        impl<R> Expectation<$subject, R> for IsZero {
            type Success<'a>
                = ()
            where
                Self: 'a,
                $subject: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a,
                $subject: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a $subject,
                _context: &AssertionContext<'_, R>,
            ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
                if actual.is_zero() { Ok(()) } else { Err(()) }
            }
        }
        impl<R> ExpectationDiagnostics<$subject, R> for IsZero
        where
            R: ValueRenderer<$subject>,
        {
            const KIND: FailureKind = FailureKind::Equality;
            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a $subject, Self::Rejection<'a>)>,
                failure: FailureBuilder<Target>,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder<Target> {
                let render = context.render();
                let zero = $zero;
                match rejected {
                    None => failure
                        .relation("is zero")
                        .expected($($present)?(render.value(&zero))),
                    Some((actual, ())) => failure
                        .actual($($present)?(render.value(actual)))
                        .expected($($present)?(render.value(&zero))),
                }
            }
        }

        sign_expectations!(@sign $subject, IsNegative, is_negative, "negative" $(, $present)?);
        sign_expectations!(@sign $subject, IsPositive, is_positive, "positive" $(, $present)?);
    };
    (@sign $subject:ident, $name:ident, $predicate:ident, $sign:literal $(, $present:path)?) => {
        #[doc = concat!("Checks whether a `", stringify!($subject), "` is ", $sign, ".")]
        pub struct $name;
        impl<R> Expectation<$subject, R> for $name {
            type Success<'a>
                = ()
            where
                Self: 'a,
                $subject: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a,
                $subject: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a $subject,
                _context: &AssertionContext<'_, R>,
            ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
                if actual.$predicate() { Ok(()) } else { Err(()) }
            }
        }
        impl<R> ExpectationDiagnostics<$subject, R> for $name
        where
            R: ValueRenderer<$subject>,
        {
            const KIND: FailureKind = FailureKind::Ordering;
            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a $subject, Self::Rejection<'a>)>,
                failure: FailureBuilder<Target>,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder<Target> {
                let render = context.render();
                match rejected {
                    None => failure.relation(concat!("is ", $sign)),
                    Some((actual, ())) => failure
                        .actual($($present)?(render.value(actual)))
                        .relation(concat!("is not ", $sign)),
                }
            }
        }
    };
}

/// Assertions for signed durations.
pub mod signed_duration;
/// Assertions for spans.
pub mod span;
/// Assertions for zoned date-times.
pub mod zoned;

/// Jiff assertion traits.
pub mod prelude {
    pub use super::signed_duration::SignedDurationAssertions;
    pub use super::span::SpanAssertions;
    pub use super::zoned::ZonedAssertions;
}
