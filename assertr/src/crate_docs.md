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

### Define evaluation and diagnostics

[`Expectation::evaluate`] borrows the subject and returns its original successful observation
or rejection. Use `()` when no additional observation is needed. It must not track or raise.
Retain observations such as converted operands, errors, or guards when checking again would
repeat user code or observe different state.

[`ExpectationDiagnostics::explain`] receives a builder and either `Some((actual, rejection))`
or `None`. The latter describes an unmet expectation with no subject, such as a missing element.
Never evaluate again during explanation. Put diagnostic renderer bounds on this trait, keeping
evaluation independent when possible. Set its `KIND` to the appropriate [`FailureKind`].

Supply the [`actual`](failure::FailureBuilder::actual) value, a lowercase
[`relation`](failure::FailureBuilder::relation) sentence without embedded values or a trailing
period, and any [`expected`](failure::FailureBuilder::expected) or
[`unexpected`](failure::FailureBuilder::unexpected) value. Add evidence through
[`fact`](failure::FailureBuilder::fact) or [`facts`](failure::FailureBuilder::facts), constructing
[`Fact::labelled`] values or [`Fact::note`] values as appropriate. Add nested
[`children`](failure::FailureBuilder::children) for further evidence. Return the populated builder.
The executor records or raises the failure according to the mode.

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

These assertion helpers show the same failure-builder pattern for each kind of subject. Their
renderer bounds cover only leaves, so the collection, map, and `Option` need no `Debug` implementation:

```
# extern crate alloc;
use alloc::collections::BTreeMap;
use assertr::{prelude::*, FailureKind};
use assertr::assertions::{collection::Collection, map::Map};

#[track_caller]
fn check_empty_collection<C: Collection, M: Mode, R: ValueRenderer<C::Item>>(
    it: AssertThat<'_, C, M, R>,
) -> AssertThat<'_, C, M, R> {
    it.track_assertion();
    if it.actual().length() != 0 {
        it.failure(FailureKind::Length)
            .actual(it.render().collection(it.actual()))
            .relation("is not empty")
            .raise();
    }
    it
}

#[track_caller]
fn check_empty_map<T: Map, M: Mode, R>(it: AssertThat<'_, T, M, R>) -> AssertThat<'_, T, M, R>
where
    R: ValueRenderer<T::Key> + ValueRenderer<T::Value>,
{
    it.track_assertion();
    if it.actual().length() != 0 {
        it.failure(FailureKind::Length)
            .actual(it.render().map(it.actual()))
            .relation("is not empty")
            .raise();
    }
    it
}

#[track_caller]
fn check_none<T, M: Mode, R: ValueRenderer<T>>(
    it: AssertThat<'_, Option<T>, M, R>,
) -> AssertThat<'_, Option<T>, M, R> {
    it.track_assertion();
    if let Some(value) = it.actual() {
        it.failure(FailureKind::Variant)
            .actual(it.render().variant(it.actual(), "Some", value))
            .relation("is not none")
            .raise();
    }
    it
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
let collection = assert_that!([Token(7)]).with_renderer(TokenRenderer).capture(check_empty_collection);
let map = assert_that!(BTreeMap::from([(1_u32, Token(7))]))
    .with_renderer(TokenRenderer).capture(check_empty_map);
let wrapper = assert_that!(Some(Token(7))).with_renderer(TokenRenderer).capture(check_none);
assert_that!(collection).has_length(1);
assert_that!(map).has_length(1);
assert_that!(wrapper).has_length(1);
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
