## Core model

An [`AssertThat<T>`](AssertThat) holds an owned or borrowed [`Actual<T>`](Actual). Methods are
selected by `T`, independent of ownership. Borrowing entry points normalize sized references to
their pointee. Owned references and unsized targets remain reference-typed subjects.

[`AssertThat::derive`] creates a child assertion for a borrowed field. Project first, then chain
assertions on the child. The parent remains usable for other field checks, and child failures
propagate to the root. Use [`AssertThat::derive_owned`] for computed values and
[`AssertThat::derive_async`] for asynchronous projections. The [`AssertThat::satisfies`] family
asserts on a child in a closure and returns the original chain. Its variants cover borrowed,
owned, and unsized projections.

Panic mode stops at the first failure. Capture mode collects structured [`AssertionFailure`]
values within [`AssertThat::capture`] or the fluent `verify` entry points. A failure carries its
structured rendered [`actual`](AssertionFailure::actual) and
[`expected`](AssertionFailure::expected) values, the [`relation`](AssertionFailure::relation)
between them, further [`facts`](AssertionFailure::facts), nested
[`children`](AssertionFailure::children), and a [`kind`](AssertionFailure::kind) as data. An
[`Adapter`](failure::adapter::Adapter) transforms that data, and adapters compose into typed
chains. Capture mode stores failures without invoking presentation. Panic mode uses the
context's [presentation adapter](AssertThat::with_panic_presentation) to produce the panic text,
defaulting to [`ToHumanReadableText`](failure::adapter::ToHumanReadableText).

## Borrowed equality

Pass a reference to reuse an expected value without cloning it. An equality definition also
retains its operand and can be borrowed by several assertions:

```
use assertr::{matchers::eq, prelude::*};

#[derive(Debug, PartialEq)]
struct Token(u32); // No Clone implementation.

let actual = Token(7);
let expected = Token(7);
assert_that!(actual).is_equal_to(&expected);
let equal = eq(&expected);
assert_that!(actual).matches(&equal);
assert_that!([Token(7)]).contains_matching(&equal);
assert_that!(expected.0).is_equal_to(7);
```

[`BorrowFor`](crate::borrow_for::BorrowFor) selects the expected view for the declared actual
type. A custom wrapper implements `Borrow<View>` and opts in with `BorrowFor<Actual>`. Equality
then requires `Actual: PartialEq<View>`. Diagnostics render the actual and the selected view,
so this wrapper needs neither `Debug` nor `Clone`:

```
use assertr::{borrow_for::BorrowFor, matchers::eq, prelude::*};
use core::borrow::Borrow;

struct ExpectedName(String);
impl Borrow<str> for ExpectedName {
    fn borrow(&self) -> &str { &self.0 }
}
impl BorrowFor<String> for ExpectedName {
    type View = str;
}

let expected = eq(ExpectedName(String::from("Ada")));
assert_that!(String::from("Ada")).matches(&expected);
let failures = assert_that!(String::from("Grace")).capture(|it| it.matches(&expected));
assert_that!(ToHumanReadableText.render(&failures[0]))
    .contains("Grace").contains("Ada");
```

The scalar constructor stores the operand. Evaluation selects its view once, and rejection retains
that reference for explanation. Missing-subject descriptions select the view without comparing.
Reference-valued subjects keep their declared type. See
[`PartialEqAssertions`](assertions::core::partial_eq::PartialEqAssertions) for explicit pointee
matching and cross-type comparison limits.

## Bulk expected data

Bulk value, key, and entry assertions accept finite, slice-backed expected lists through
`AsRef`. Arrays, slices, vectors, and compatible wrappers reuse their existing storage.
Generators require explicit preparation:

```
use assertr::prelude::*;
let expected = (1..=3).collect::<Vec<_>>();
assert_that!([1, 2, 3, 4]).contains_all(&expected);
assert_that!([1, 2, 3]).contains_all((1..=3).collect::<Vec<_>>());
```

Borrowed lists of custom operands use the stored element type's `BorrowFor` implementation.
They need no extra implementation for references to that wrapper. Bulk expected data must be
repeatable: repeated slice access returns the same logical list, and repeated operand borrowing
describes the same comparison value throughout evaluation and explanation. Access counts and
interleaving with comparisons are unspecified. Constructors store inputs without accessing
views, and library-controlled access occurs after assertion tracking.

Prepare stateful data before the assertion, or use a custom expectation retaining its observation.
Explanation may access expected data again through budgeted rendering, but never repeats
comparisons, searches, lookups, callbacks, or iterator consumption. Rejections retain the failed
observations instead of complete expected-view buffers. Scalar borrowing and matcher, callback,
guard, and identity contracts remain unchanged.

## Async limitations

Await asynchronous operations before applying synchronous expectations, or use an async
projection such as [`AssertThat::derive_async`]. With `std`, async function assertions consume
an owned closure in panic mode. The caller supplies the runtime and awaits the returned future:

```
# #[cfg(feature = "std")]
# {
use assertr::prelude::*;

# tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
assert_that_owned!(async || 7_u32)
    .does_not_panic_async().await
    .is_equal_to(7);
# });
# }
```

Async function assertions capture the caller when called, then track and invoke the closure
on the first poll. Cancellation does not restore consumed inputs or user state. `panics_async`
catches invocation, polling, and output-drop panics. `does_not_panic_async` returns the output,
so its later drop is outside the caught boundary. Reqwest body extraction has a different
boundary: it checks ownership and tracks before returning the future, then reads when polled.

[`Expectation::evaluate`] and [`ExpectationDiagnostics::explain`] are synchronous hooks.
[`AssertThat::capture`] likewise expects a synchronous callback returning the chain:

```compile_fail,E0308
use assertr::prelude::*;

let failures = assert_that!(7).capture(|it| async move {
    it.is_equal_to(7)
});
```

Chains are neither `Send` nor `Sync`. A future keeping a chain across an await cannot be sent
between threads, even if its subject and renderer are thread-safe:

```compile_fail,E0277
use assertr::prelude::*;

fn requires_send(_: impl core::future::Future<Output = ()> + Send) {}

requires_send(async {
    let chain = assert_that_owned!(7);
    core::future::ready(()).await;
    chain.is_equal_to(7);
});
```

When a task API requires `Send`, await the input first, then construct and complete the chain
without carrying it across another await. A runtime that supports local futures can instead
keep the chain in the same task across suspension.

## Custom assertions

Add a method such as `.is_adult()` when a domain check appears throughout your tests. For a
single check on a field, start with [`AssertThat::derive`]. To describe selected fields and
nested values together, use [`partial!`](mod@matchers#structural-syntax). Its field expectations
can use existing assertion methods through [`matchers::satisfying`], including your custom ones.

Existing assertion families also work with custom types that implement their capabilities.
For example, [`HasLength`](assertions::HasLength) provides length assertions and
[`Collection`](assertions::collection::Collection) provides order-free element assertions. See
the [assertion families](assertions) before introducing a separate trait.

### Define a chainable method

Define your own assertion trait and implement it for `AssertThat<'_, YourType, M, R>`. Use
`M: Mode` so the same implementation works in panic and capture mode. Keep `R` unconstrained on
the impl, and put renderer and `Clone` bounds on each method that needs them, in both the trait
and impl. This keeps one method's rendering needs from hiding the entire trait. A default of
`R = DebugRenderer` lets callers name the trait without specifying a renderer.

For a chainable check, take and return `Self`. Mark the method `#[track_caller]` so failures
report its caller's location. The example below shows two implementation styles:

- **Composition:** Delegate to existing assertions through [`AssertThat::satisfies`] and
  friends. Delegated assertions handle tracking, diagnostics, and capture mode. Do not call
  [`AssertThat::track_assertion`] again in a method that only delegates.
- **Leaf assertion:** Implement [`Expectation`] and [`ExpectationDiagnostics`], then delegate
  to [`AssertThat::apply_assertion`]. This tracks once, evaluates, and raises any explained
  rejection. The same definition also works with `.matches(...)` and nested composition.

An execution adapter owns invocation, consumption, or polling that cannot use the borrowed
expectation protocol. It tracks explicitly before its operation and preserves the caller
location. Built-in adapters then use private executor entry points that skip tracking.
Downstream adapters cannot call those private entry points. If an adapter must construct a
failure directly, use [`AssertThat::failure`], the same structured fields, and
[`AssertThat::render`], then raise it. Keep that responsibility outside expectation hooks.

### Define evaluation and diagnostics

[`Expectation::evaluate`] borrows the subject and returns its original successful observation
or rejection. Use `()` when no additional observation is needed. It must not track or raise.
Retain observations such as converted operands, errors, or guards when checking again would
repeat an observation or observe different state. Repeatable [bulk expected data](#bulk-expected-data)
may instead be accessed again during explanation.

[`ExpectationDiagnostics::explain`] receives a builder and either `Some((actual, rejection))`
or `None`. The latter describes an unmet expectation with no subject, such as a missing element.
Never evaluate again during explanation. Put diagnostic renderer bounds on this trait, keeping
evaluation independent when possible. Set its `KIND` to the appropriate [`FailureKind`].
Explanation must not track or raise. Keep observations alive until the evidence needing them
has been rendered, and release temporary guards before returning the builder.

Supply the [`actual`](failure::FailureBuilder::actual) value, a lowercase
[`relation`](failure::FailureBuilder::relation) sentence without embedded values or a trailing
period, and any [`expected`](failure::FailureBuilder::expected) or
[`unexpected`](failure::FailureBuilder::unexpected) value. Add evidence through
[`fact`](failure::FailureBuilder::fact) or [`facts`](failure::FailureBuilder::facts), constructing
[`Fact::labelled`] values or [`Fact::note`] values as appropriate. Add nested
[`children`](failure::FailureBuilder::children) for further evidence. Return the populated builder.
The chain executor raises the completed failure through the active mode. Child contexts instead
build and retain it as evidence for the enclosing assertion.

Render diagnostic values through [`AssertionContext::render`]. Its
[`value`](renderer::RenderingContext::value), [`values`](renderer::RenderingContext::values),
and [structural adapters](#structural-evidence) apply the active renderer and rendering budget.
Pass these adapters to the value setters or `Fact` constructors.
This preserves structured values and type metadata for [failure adapters](failure::adapter) and lets Assertr
produce a consistent report. Errors, lengths, counts, and expected indices are typed evidence too.
Render notes with `.fact(Fact::note(context.render().value(&error)))`. Verbatim notes and primitive
conversions are reserved for structural metadata and caller-authored prose. See the
[rendering guide](renderer) for customization.

### Example

```
use assertr::prelude::*;
use assertr::{AssertionContext, Expectation, ExpectationDiagnostics};
use assertr::failure::{FailureBuilder, FailureKind};

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

// One definition serves both the chain method and composition.
struct HasName;

impl<R> Expectation<Person, R> for HasName {
    type Success<'a> = ();
    type Rejection<'a> = &'a str;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Person,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), &'a str> {
        if actual.name.is_empty() {
            Err(&actual.name)
        } else {
            Ok(())
        }
    }
}

impl<R: ValueRenderer<str>> ExpectationDiagnostics<Person, R> for HasName {
    const KIND: FailureKind = FailureKind::Predicate;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Person, &'a str)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            Some((_, name)) => failure
                .actual(context.render().value(name))
                .relation("has an empty name"),
            None => failure.relation("has a nonempty name"),
        }
    }
}

trait PersonAssertions<R = DebugRenderer> {
    #[track_caller]
    fn is_adult(self) -> Self
    where
        R: Clone + ValueRenderer<u32>;

    #[track_caller]
    fn has_name(self) -> Self
    where
        R: ValueRenderer<str>;
}

impl<M: Mode, R> PersonAssertions<R> for AssertThat<'_, Person, M, R> {
    // Composed: the delegated assertion tracks itself and formats the failure.
    #[track_caller]
    fn is_adult(self) -> Self
    where
        R: Clone + ValueRenderer<u32>,
    {
        self.satisfies(|person| &person.age, |age| {
            age.is_greater_or_equal_to(18);
        })
    }

    // Leaf: the shared executor tracks, evaluates, and raises failures.
    #[track_caller]
    fn has_name(self) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.apply_assertion(HasName)
    }
}

assert_that!(Person { name: "Ada".into(), age: 36 }).is_adult().has_name();
assert_that!([Person { name: "Ada".into(), age: 36 }]).contains_matching(HasName);

let failures = assert_that!(Person { name: "".into(), age: 16 })
    .capture(|person| person.is_adult().has_name());
assert_that!(failures).has_length(2);
```

Assertr's own `*Assertions` traits are public for method discovery only. Implementing them for
other types is not supported. See [API stability](#api-stability).

### Structural evidence

Use [`collection`](renderer::RenderingContext::collection) for a collection's own presentation
and canonical type. [`borrowed_collection`](renderer::RenderingContext::borrowed_collection)
renders an explicit item view, such as `String` items through `ValueRenderer<str>`, while retaining
the collection's outer type. Positional evidence uses
[`stable_collection`](renderer::RenderingContext::stable_collection) or its borrowed counterpart.
These require [`StableOrder`](assertions::collection::StableOrder) and preserve iteration order
even if the ordinary collection presentation sorts diagnostic text.

[`map`](renderer::RenderingContext::map) renders keys and values separately and follows the map's
[`RENDERING_ORDER`](assertions::map::Map::RENDERING_ORDER). Wrappers such as
[`variant`](renderer::RenderingContext::variant) and
[`struct_field`](renderer::RenderingContext::struct_field) retain the owner's type and wrap a
single rendered leaf. [`unavailable_struct_field`](renderer::RenderingContext::unavailable_struct_field)
records a structural placeholder without requiring a field renderer or inventing a field type.

These expectations populate the supplied builder for each kind of subject. Their renderer
bounds cover only leaves, so the collection, map, and `Option` need no `Debug` implementation.
`apply_assertion` owns tracking and raising for each call:

```
# extern crate alloc;
use alloc::collections::BTreeMap;
use assertr::{prelude::*, AssertionContext, Expectation, ExpectationDiagnostics, FailureKind};
use assertr::assertions::{collection::Collection, map::Map};
use assertr::failure::FailureBuilder;

struct EmptyCollection;
impl<C: Collection, R> Expectation<C, R> for EmptyCollection {
    type Success<'a> = () where C: 'a;
    type Rejection<'a> = () where C: 'a;

    fn evaluate(&self, actual: &C, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.length() == 0 { Ok(()) } else { Err(()) }
    }
}
impl<C: Collection, R: ValueRenderer<C::Item>> ExpectationDiagnostics<C, R> for EmptyCollection {
    const KIND: FailureKind = FailureKind::Length;

    fn explain<Target>(
        &self,
        rejected: Option<(&C, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            Some((actual, ())) => failure
                .actual(context.render().collection(actual))
                .relation("is not empty"),
            None => failure.relation("is empty"),
        }
    }
}

struct EmptyMap;
impl<T: Map, R> Expectation<T, R> for EmptyMap {
    type Success<'a> = () where T: 'a;
    type Rejection<'a> = () where T: 'a;

    fn evaluate(&self, actual: &T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.length() == 0 { Ok(()) } else { Err(()) }
    }
}
impl<T: Map, R> ExpectationDiagnostics<T, R> for EmptyMap
where
    R: ValueRenderer<T::Key> + ValueRenderer<T::Value>,
{
    const KIND: FailureKind = FailureKind::Length;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            Some((actual, ())) => failure
                .actual(context.render().map(actual))
                .relation("is not empty"),
            None => failure.relation("is empty"),
        }
    }
}

struct NoValue;
impl<T, R> Expectation<Option<T>, R> for NoValue {
    type Success<'a> = () where T: 'a;
    type Rejection<'a> = &'a T where T: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Option<T>,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), &'a T> {
        match actual { None => Ok(()), Some(value) => Err(value) }
    }
}
impl<T, R: ValueRenderer<T>> ExpectationDiagnostics<Option<T>, R> for NoValue {
    const KIND: FailureKind = FailureKind::Variant;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Option<T>, &'a T)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            Some((actual, value)) => failure
                .actual(context.render().variant(actual, "Some", value))
                .relation("is not none"),
            None => failure.relation("is none"),
        }
    }
}

struct Token(u32);
struct TokenRenderer;
impl ValueRenderer<Token> for TokenRenderer {
    fn fmt(&self, value: &Token, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "token({})", value.0)
    }
}

// Keys and values can have different types. Supply each leaf capability on the same renderer.
impl ValueRenderer<u32> for TokenRenderer {
    fn fmt(&self, value: &u32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "key({value})")
    }
}
let collection = assert_that!([Token(7)]).with_renderer(TokenRenderer)
    .capture(|it| it.apply_assertion(EmptyCollection));
let map = assert_that!(BTreeMap::from([(1_u32, Token(7))]))
    .with_renderer(TokenRenderer).capture(|it| it.apply_assertion(EmptyMap));
let wrapper = assert_that!(Some(Token(7))).with_renderer(TokenRenderer)
    .capture(|it| it.apply_assertion(NoValue));
assert_that!(collection).has_length(1);
assert_that!(map).has_length(1);
assert_that!(wrapper).has_length(1);
assert_that!(ToHumanReadableText.render(&collection[0])).contains("token(7)");
assert_that!(ToHumanReadableText.render(&map[0])).contains("key(1): token(7)");
assert_that!(ToHumanReadableText.render(&wrapper[0])).contains("Some(").contains("token(7)");
```

For synthetic evidence, use [`values`](renderer::RenderingContext::values),
[`borrowed_values`](renderer::RenderingContext::borrowed_values), or
[`entry_list`](renderer::RenderingContext::entry_list). These retain child types without claiming
an outer subject type. Choose [`RenderingOrder`](renderer::RenderingOrder) explicitly with
[`RenderedValues::with_order`](renderer::RenderedValues::with_order) or the `entry_list` argument.
Sorted rendering orders the budgeted leaf text before applying the item limit.

[`RenderingContext::budget`](renderer::RenderingContext::budget) returns a copy of the active
limits. A custom collector can retain at most `budget().max_items()` children and use
[`FailureBuilder::omitted_children`](failure::FailureBuilder::omitted_children) for the rest.
Evaluate the complete assertion result independently of those limits, including when the item
limit is zero. Inside an expectation, also respect [`AssertionContext::is_diagnostic`] when
collecting optional evidence. Prefer its child-evaluation helpers when composing expectations.
