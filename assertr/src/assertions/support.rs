//! Crate-private building blocks shared by the assertion families.
//!
//! This module is declared first with `#[macro_use]`, so its macros are in scope for every
//! assertion family module declared after it.

use crate::{AssertThat, Mode, actual::Actual, failure::FailureBuilder, renderer::Rendered};

/// Defines a unit expectation for a property of the subject that retains no observation.
///
/// The subject is either a concrete type (`for bool`) or a generic subject (`for<T: Signed> T`,
/// `for<T> Mutex<T>`). `check` receives the borrowed subject. Diagnostics either describe the
/// property with a positive and a negated relation (`relations`), or compare with a computed value
/// as a direct equality (`expected`). Both render the subject through the active renderer. The
/// optional `present` function replaces that rendering: it receives the rendering context and the
/// value and returns the rendered value. `hidden` describes only the relations and needs no
/// renderer.
macro_rules! property_expectation {
    (
        $(#[$attr:meta])*
        pub struct $name:ident for<$($param:ident $(: $bound:path)?),+> $subject:ty;
        $($rest:tt)*
    ) => {
        property_expectation!(
            @define [$(#[$attr])*] $name [$($param)+] [$($param: $($bound)?,)+] $subject; $($rest)*
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
        @define [$($attr:tt)*] $name:ident [$($param:ident)*] [$($bounds:tt)*] $subject:ty;
        kind $kind:ident;
        check |$actual:ident| $check:expr;
        relations $relation:literal, $negated:literal;
        hidden;
    ) => {
        property_expectation!(
            @impl [$($attr)*] $name [$($param)*] [$($bounds)*] $subject; $kind; |$actual| $check;
            |rejected, failure, _context| failure.relation(
                if rejected.is_some() { $negated } else { $relation }
            )
        );
    };
    (
        @define [$($attr:tt)*] $name:ident [$($param:ident)*] [$($bounds:tt)*] $subject:ty;
        kind $kind:ident;
        check |$actual:ident| $check:expr;
        $($diagnostics:tt)+
    ) => {
        property_expectation!(
            @impl [$($attr)*] $name [$($param)*]
                [$($bounds)* R: $crate::ValueRenderer<$subject>,] $subject; $kind; |$actual| $check;
            |rejected, failure, context| property_expectation!(
                @explain rejected, failure, context; $($diagnostics)+
            )
        );
    };
    (
        @impl [$($attr:tt)*] $name:ident [$($param:ident)*] [$($bounds:tt)*] $subject:ty;
        $kind:ident; |$actual:ident| $check:expr;
        |$rejected:ident, $failure:ident, $context:ident| $explain:expr
    ) => {
        $($attr)*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl<$($param,)* R> $crate::Expectation<$subject, R> for $name
        where
            $($bounds)*
        {
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
                $actual: &'a $subject,
                _: &$crate::AssertionContext<'_, R>,
            ) -> Result<(), ()> {
                if $check { Ok(()) } else { Err(()) }
            }

            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::$kind;

            fn explain(
                &self,
                $rejected: Option<(&$subject, ())>,
                $failure: $crate::failure::FailureBuilder,
                $context: &$crate::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder {
                $explain
            }
        }
    };
    (@explain $rejected:ident, $failure:ident, $context:ident;
        relations $relation:literal, $negated:literal;
        $(present $present:expr;)?
    ) => {{
        let render = $context.render();
        $failure.relations(
            $rejected.map(|(actual, ())| {
                property_expectation!(@present [$($present)?] render, actual)
            }),
            $relation,
            $negated,
        )
    }};
    (@explain $rejected:ident, $failure:ident, $context:ident;
        expected $expected:expr, $relation:literal;
        $(present $present:expr;)?
    ) => {{
        let render = $context.render();
        let failure = match $rejected {
            None => $failure.relation($relation),
            Some((actual, ())) => $failure.actual(
                property_expectation!(@present [$($present)?] render, actual)
            ),
        };
        failure.expected(property_expectation!(@present [$($present)?] render, &$expected))
    }};
    (@present [] $render:ident, $value:expr) => {
        $render.value($value)
    };
    (@present [$present:expr] $render:ident, $value:expr) => {
        ($present)($render, $value)
    };
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

/// Explains a variant expectation, rendering the rejected subject's variant when there is one.
pub(crate) fn explain_variant(
    failure: FailureBuilder,
    rejected: Option<impl Into<Rendered>>,
    expected: &'static str,
) -> FailureBuilder {
    failure
        .relations(
            rejected.map(Into::into),
            "is the expected variant",
            "is not the expected variant",
        )
        .expected(expected)
}

impl<T, M: Mode, R: Clone> AssertThat<'_, T, M, R> {
    /// Runs `assertions` on a child over `value`, the successful observation of a preceding
    /// variant check. A failed check has already been raised, so the callback does not run.
    pub(crate) fn satisfy_success<U>(
        &self,
        value: Option<&U>,
        assertions: impl for<'a> FnOnce(AssertThat<'a, U, M, R>),
    ) {
        if let Some(value) = value {
            assertions(self.derive(|_| value));
        }
    }
}
