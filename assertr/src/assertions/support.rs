//! Crate-private building blocks shared by the assertion families.
//!
//! This module is declared first with `#[macro_use]`, so its macros are in scope for every
//! assertion family module declared after it.

use crate::actual::Actual;

/// Defines a unit expectation for a property of the subject that retains no observation.
///
/// The subject is either a concrete type (`for bool`) or one generic parameter with a single trait
/// bound (`for<T: Signed>`). `check` receives the borrowed subject. Diagnostics either describe
/// the property with a positive and a negated relation (`relations`), or compare with a computed
/// value as a direct equality (`expected`). Both render the subject through the active renderer.
macro_rules! property_expectation {
    (
        $(#[$attr:meta])*
        pub struct $name:ident for<$param:ident: $bound:path>;
        $($rest:tt)*
    ) => {
        property_expectation!(
            @define [$(#[$attr])*] $name [$param] [$param: $bound,] $param; $($rest)*
        );
    };
    (
        $(#[$attr:meta])*
        pub struct $name:ident for $subject:ty;
        $($rest:tt)*
    ) => {
        property_expectation!(@define [$(#[$attr])*] $name [] [] $subject; $($rest)*);
    };
    (
        @define [$($attr:tt)*] $name:ident [$($param:ident)?] [$($bounds:tt)*] $subject:ty;
        kind $kind:ident;
        check |$actual:ident| $check:expr;
        $($diagnostics:tt)+
    ) => {
        $($attr)*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl<$($param,)? R> $crate::Expectation<$subject, R> for $name
        where
            $($bounds)*
        {
            type Success<'a>
                = ()
            where
                $subject: 'a;
            type Rejection<'a>
                = ()
            where
                $subject: 'a;

            fn evaluate<'a>(
                &'a self,
                $actual: &'a $subject,
                _: &$crate::AssertionContext<'_, R>,
            ) -> Result<(), ()> {
                if $check { Ok(()) } else { Err(()) }
            }
        }

        impl<$($param,)? R> $crate::ExpectationDiagnostics<$subject, R> for $name
        where
            $($bounds)*
            R: $crate::ValueRenderer<$subject>,
        {
            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::$kind;

            fn explain<Target>(
                &self,
                rejected: Option<(&$subject, ())>,
                failure: $crate::failure::FailureBuilder<Target>,
                context: &$crate::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder<Target> {
                property_expectation!(@explain rejected, failure, context; $($diagnostics)+)
            }
        }
    };
    (@explain $rejected:ident, $failure:ident, $context:ident;
        relations $relation:literal, $negated:literal;
    ) => {{
        let render = $context.render();
        match $rejected {
            None => $failure.relation($relation),
            Some((actual, ())) => $failure.actual(render.value(actual)).relation($negated),
        }
    }};
    (@explain $rejected:ident, $failure:ident, $context:ident;
        expected $expected:expr, $relation:literal;
    ) => {{
        let render = $context.render();
        let failure = match $rejected {
            None => $failure.relation($relation),
            Some((actual, ())) => $failure.actual(render.value(actual)),
        };
        failure.expected(render.value(&$expected))
    }};
}

/// Implements `Clone`, `Copy`, and `Debug` for an expectation that only selects a type `E` through
/// `PhantomData<fn() -> E>`. The selected type needs none of these traits. `Debug` names it.
macro_rules! type_selection_traits {
    ($name:ident) => {
        impl<E> ::core::clone::Clone for $name<E> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<E> ::core::marker::Copy for $name<E> {}
        impl<E> ::core::fmt::Debug for $name<E> {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(
                    formatter,
                    "{}<{}>",
                    stringify!($name),
                    ::core::any::type_name::<E>()
                )
            }
        }
    };
}

/// Implements `Clone` and `Debug` for an expectation storing its operands as `expected: B` beside
/// a phantom operand marker. Only the stored operands `B` need these traits.
macro_rules! expected_operands_traits {
    ($name:ident<$($param:ident),+>, $marker:ident) => {
        impl<$($param),+> ::core::clone::Clone for $name<$($param),+>
        where
            B: ::core::clone::Clone,
        {
            fn clone(&self) -> Self {
                Self {
                    expected: self.expected.clone(),
                    $marker: ::core::marker::PhantomData,
                }
            }
        }
        impl<$($param),+> ::core::fmt::Debug for $name<$($param),+>
        where
            B: ::core::fmt::Debug,
        {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("expected", &self.expected)
                    .finish()
            }
        }
    };
}

/// Projects a subject whose shape a preceding expectation has already confirmed.
///
/// `owned` and `borrowed` extract the checked part of an owned or borrowed subject. They return
/// `None` only if the subject did not pass that check, which would be a bug in the caller.
pub(crate) fn project_checked<'t, T, U>(
    actual: Actual<'t, T>,
    owned: impl FnOnce(T) -> Option<U>,
    borrowed: impl FnOnce(&'t T) -> Option<&'t U>,
) -> Actual<'t, U> {
    let projected = match actual {
        Actual::Owned(value) => owned(value).map(Actual::Owned),
        Actual::Borrowed(value) => borrowed(value).map(Actual::Borrowed),
    };
    projected.unwrap_or_else(|| unreachable!("the expectation already checked the subject"))
}
