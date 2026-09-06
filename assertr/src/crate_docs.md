## Core model

An [`AssertThat<T>`](AssertThat) holds an owned or borrowed [`Actual<T>`](Actual). Methods are
selected by `T`, independent of ownership. Borrowing entry points normalize sized references to
their pointee. Owned references and unsized targets remain reference-typed subjects.

[`AssertThat::derive`] creates a child assertion for part of a subject. Its failures propagate
to the root. The [`AssertThat::satisfies`] family asserts on a child and returns the original
chain. Its variants cover borrowed, owned, and unsized projections.

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
single check on a field, start with [`AssertThat::satisfies`]. To describe selected fields and
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
- **Leaf assertion:** Call [`AssertThat::track_assertion`] first. When the condition fails,
  build structured evidence and raise it as described below.

### Build a leaf failure

Start with [`AssertThat::failure`] and the [`FailureKind`] of the assertion's family. Supply
the [`actual`](failure::FailureBuilder::actual) value, a lowercase
[`relation`](failure::FailureBuilder::relation) sentence without embedded values or a trailing
period, and any [`expected`](failure::FailureBuilder::expected) or
[`unexpected`](failure::FailureBuilder::unexpected) value. Add labeled
[`fact`](failure::FailureBuilder::fact)s, [`note`](failure::FailureBuilder::note)s, or nested
[`children`](failure::FailureBuilder::children) for further evidence. Call
[`raise`](failure::FailureBuilder::raise) to record the failure or panic according to the mode.

Render diagnostic values through [`AssertThat::render`]. Its
[`value`](renderer::RenderingContext::value), [`values`](renderer::RenderingContext::values),
and [`borrowed_values`](renderer::RenderingContext::borrowed_values) adapters apply the active
renderer and rendering budget. Pass these adapters directly to the builder. This preserves
structured values and type metadata for [failure adapters](failure::adapter) and lets Assertr
produce a consistent report. See the [rendering guide](renderer) for customization.

### Example

```
use assertr::prelude::*;
use assertr::failure::FailureKind;

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

trait PersonAssertions<R = DebugRenderer> {
    #[track_caller]
    fn is_adult(self) -> Self
    where
        R: Clone + ValueRenderer<u32>;

    #[track_caller]
    fn has_name(self) -> Self
    where
        R: ValueRenderer<Person> + ValueRenderer<String>;
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

    // Leaf: track first, then raise a failure built from rendered values.
    #[track_caller]
    fn has_name(self) -> Self
    where
        R: ValueRenderer<Person> + ValueRenderer<String>,
    {
        self.track_assertion();
        if self.actual().name.is_empty() {
            self.failure(FailureKind::Predicate)
                .actual(self.render().value(self.actual()))
                .relation("has no name")
                .fact("Name", self.render().value(&self.actual().name))
                .raise();
        }
        self
    }
}

assert_that!(Person { name: "Ada".into(), age: 36 }).is_adult().has_name();

let failures = assert_that!(Person { name: "".into(), age: 16 })
    .capture(|person| person.is_adult().has_name());
assert_eq!(failures.len(), 2);
```

Assertr's own `*Assertions` traits are public for method discovery only. Implementing them for
other types is not supported. See [API stability](#api-stability).
