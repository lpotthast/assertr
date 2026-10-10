## Core concepts

### Subjects and ownership

An [`AssertThat<T>`](AssertThat) chain holds the subject, the value under test. The subject's type
`T` decides which assertions are available. Whether the chain owns or borrows the value makes no
difference to that.

`assert_that!(value)` borrows. `assert_that!(&value)` is equivalent for sized values, and both
produce `AssertThat<Value>`. Unsized targets such as `str` and `[T]` stay references, so
`assert_that!("text")` produces `AssertThat<&str>`. Closure literals and `async` blocks are owned,
because nothing else could use them. `assert_that_owned!(value)` takes ownership of anything else,
which only the assertions that consume their subject need, such as iterator assertions.

### Child chains

[`AssertThat::derive`] starts a child chain on a field of the subject. The parent stays usable,
and a failure on the child counts as a failure of the parent. Use [`AssertThat::derive_owned`] for
computed values and [`AssertThat::derive_async`] when the projection must be awaited.

The [`AssertThat::satisfies`] family does the same inside a closure and then returns the parent.
This keeps a single chain going:

```
use assertr::prelude::*;

let person = (String::from("Ada"), 36);
assert_that!(person)
    .satisfies(|p| &p.0, |name| {
        name.starts_with("A");
    })
    .satisfies(|p| &p.1, |age| {
        age.is_greater_or_equal_to(18);
    });
```

Children inherit the parent's renderer, rendering budget, detail messages, location setting, and
panic presentation. They start without a subject name or source expression.

### Panic mode and capture mode

Every chain runs in one of two [modes](mode). In panic mode, the default, the first failing
assertion panics with its report. In capture mode, failures are collected as
[`AssertionFailure`](failure::AssertionFailure) values. Enter capture mode with [`AssertThat::capture`], or with `verify(..)`
when the `fluent` feature is enabled.

A failure stores its parts as data: the rendered [`actual`](failure::AssertionFailure::actual) and
[`expected`](failure::AssertionFailure::expected) values, the [`relation`](failure::AssertionFailure::relation)
between them, extra [`facts`](failure::AssertionFailure::facts), nested
[`children`](failure::AssertionFailure::children), and its [`kind`](failure::AssertionFailure::kind). Its `Display`
implementation produces the report. In panic mode, the report becomes the panic message unless you
replace it with [`AssertThat::with_panic_presentation`]. See [`failure`] for details.

### Adding context to failures

Name the subject or add messages that appear in every later failure of the chain:

```
use assertr::prelude::*;

let failures = assert_that!(vec![3, 1, 2])
    .with_subject_name("ids")
    .with_detail_message("ids must be returned in order")
    .capture(|ids| ids.contains_exactly([1, 2, 3]));
assert_that!(failures[0].to_string())
    .contains("Subject: ids")
    .contains("ids must be returned in order");
```

[`AssertThat::with_location`] turns off the file and line in reports, which helps when a test
compares a whole report.

## Comparing values

### Borrowed expected values

Comparisons accept the expected value owned or by reference. Passing a reference lets you reuse a
value that does not implement `Clone`. A matcher such as [`matchers::eq`] can also hold a reference
and be used several times:

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
```

The [`BorrowFor`](crate::borrow_for::BorrowFor) trait decides which borrowed form an expected
value compares as. That is why `"Ada"` works as the expected value for a `String`. Your own
wrapper types can opt in. Implement `Borrow<View>` and `BorrowFor<Actual>` with `View` as the
comparison type, and the subject type must implement `PartialEq<View>`. Reports show the view, so
the wrapper needs neither `Debug` nor `Clone`:

```
use assertr::{borrow_for::BorrowFor, matchers::eq, prelude::*};
use core::borrow::Borrow;

struct ExpectedName(String);

impl Borrow<str> for ExpectedName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl BorrowFor<String> for ExpectedName {
    type View = str;
}

let expected = eq(ExpectedName(String::from("Ada")));
assert_that!(String::from("Ada")).matches(&expected);
```

References as subjects keep their type. See
[`PartialEqAssertions`](assertions::PartialEqAssertions) for comparing through
references and for cross-type comparisons.

### Expected lists

Assertions such as `contains_all` and `contains_exactly` take their expected items as a list that
can be viewed as a slice. Arrays, slices, and vectors all work without copying. Collect an
iterator into a `Vec` first:

```
use assertr::prelude::*;

assert_that!([1, 2, 3, 4]).contains_all([1, 2]);
assert_that!([1, 2, 3, 4]).contains_all((1..=3).collect::<Vec<_>>());
```

Lists of custom wrapper values compare through the wrapper's `BorrowFor` implementation, so they
need nothing extra. The list may be read more than once, for example once to compare and once to
build the report. It must return the same items every time. Prepare data with side effects before
the assertion.

## Async code

Await the value first and assert on the result, or use an async projection such as
[`AssertThat::derive_async`]. With `std`, async closures can be checked for panics. The test
provides the runtime and awaits the returned future:

```
# #[cfg(feature = "std")]
# {
use assertr::prelude::*;

# tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
assert_that!(async || 7_u32)
    .does_not_panic_async().await
    .is_equal_to(7);
assert_that!(async || panic!("boom"))
    .panics_async().await
    .has_type::<&str>()
    .is_equal_to("boom");
# });
# }
```

`panics_async` catches panics while calling the closure, polling its future, and dropping the
output. `does_not_panic_async` returns the output, so a panic while dropping it later is not
caught.

Expectations and [`AssertThat::capture`] callbacks are synchronous. Chains are also neither `Send`
nor `Sync`, so a future that holds a chain across an `.await` cannot move between threads:

```compile_fail,E0277
use assertr::prelude::*;

fn requires_send(_: impl core::future::Future<Output = ()> + Send) {}

requires_send(async {
    let chain = assert_that_owned!(7);
    core::future::ready(()).await;
    chain.is_equal_to(7);
});
```

When a task must be `Send`, await the input first, then build and finish the chain without another
`.await` in between.

### Values that change over time

For state that changes asynchronously, such as a page, a cache, or a background job, assert on an
observation: a closure returning a future of the current value. `eventually` observes it until the
expectation holds and continues with that value. `consistently` requires it to keep holding. Both end
with any matcher or assertion callback:

```
# #[cfg(feature = "std")]
# {
use assertr::{matchers::ge, prelude::*};
use std::{sync::atomic::{AtomicU32, Ordering}, time::Duration};

# tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
let processed = AtomicU32::new(0);
let observe = || async { processed.fetch_add(1, Ordering::SeqCst) + 1 };

assert_that!(observe)
    .eventually()
    .within(Duration::from_millis(500))
    .matches(ge(3))
    .await
    .is_less_than(10);
assert_that!(observe)
    .consistently()
    .satisfies(|count| {
        count.is_greater_than(3);
    })
    .await;
# });
# }
```

A failure shows the expectation's report, how long and how often the value was observed, and the
values seen. [`Patience`](crate::assertions::Patience) configures the timeout, the polling interval,
and the consistency duration. The defaults are fast (1 s, 10 ms, 100 ms). Set your own for a whole
test suite with `Patience::set_global`, and override them for one assertion with `within`,
`polling_every`, `for_at_least`, or `with_patience`. Use `eventually_ok` and `consistently_ok` for
observations returning a `Result`. Eventual assertions need no particular runtime, and their futures
are `Send` when the observation is.

## Custom assertions

Before writing new code, check whether existing tools cover the case:

- For one check on a field, use [`AssertThat::derive`].
- For several fields and nested values, use [`partial!`](mod@matchers#structural-syntax).
- For your own collection or map type, implement the matching capability, such as
  [`HasLength`](assertions::HasLength) or [`Collection`](assertions::Collection). The
  existing assertions then work on it. See the [assertion families](assertions).

When a domain check shows up throughout your tests, give it a name. There are three ways, from
least to most effort.

### Reusable checks from existing matchers

A function returning `impl Expectation` needs no trait implementation. Build it from:

- [`matchers::field`] to apply a matcher to one field. Failures point at that field.
- [`matchers::predicate`] to turn a boolean closure into a check. Name the check with
  [`described_as`](matchers::Predicate::described_as) and the failure with
  [`rejected_as`](matchers::Predicate::rejected_as).
- [`matchers::satisfying`] to use assertion methods, including your own, as a check.
- [`matchers::all_of`], [`matchers::any_of`], and [`partial!`](mod@matchers#structural-syntax) to
  combine checks.

```
use assertr::matchers::{IsNotEmpty, field, ge, predicate};
use assertr::prelude::*;

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

fn is_adult<R: ValueRenderer<u32>>() -> impl Expectation<Person, R> + Clone {
    field("age", |person: &Person| &person.age, ge(18))
}

fn has_name<R: ValueRenderer<String>>() -> impl Expectation<Person, R> + Clone {
    field("name", |person: &Person| &person.name, IsNotEmpty)
}

fn has_short_name<R: ValueRenderer<Person>>() -> impl Expectation<Person, R> + Clone {
    predicate(|person: &Person| person.name.len() <= 8)
        .described_as("has a short name")
        .rejected_as("has a long name")
}

let ada = Person { name: "Ada".into(), age: 36 };
assert_that!(&ada).matches(is_adult()).matches(has_short_name());
assert_that!([ada]).contains_matching(has_name());

let failures = assert_that!(Person { name: "".into(), age: 16 })
    .with_location(false)
    .capture(|person| person.matches(is_adult()).matches(has_name()));
assert_that!(failures).has_length(2);
assert_that!(failures[0].to_string()).contains("At .age:");
```

[`AssertThat::matches`] runs any expectation as a step in a chain. The same values work
with `.matches(..)`, `contains_matching(..)`, and as fields in `partial!`.

### A chainable method

To write `.is_adult()` directly on a chain, define a trait and implement it for
`AssertThat<'_, YourType, M, R>`:

- Make the impl generic over `M: Mode` so the method works in panic and capture mode.
- Leave `R` unbounded on the impl. Put renderer and `Clone` bounds on each method instead, in both
  the trait and the impl. Otherwise one method's needs would hide the whole trait. A default of
  `R = DebugRenderer` lets callers name the trait without a renderer.
- Take and return `Self`, and mark the method `#[track_caller]` so failures point at the caller.

The body either delegates to existing assertions or applies an expectation. Both handle failure
reporting and capture mode for you:

```
use assertr::prelude::*;

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
        R: ValueRenderer<String>;
}

impl<M: Mode, R> PersonAssertions<R> for AssertThat<'_, Person, M, R> {
    #[track_caller]
    fn is_adult(self) -> Self
    where
        R: Clone + ValueRenderer<u32>,
    {
        // Delegate to an existing assertion on a child chain.
        self.satisfies(|person| &person.age, |age| {
            age.is_greater_or_equal_to(18);
        })
    }

    #[track_caller]
    fn has_name(self) -> Self
    where
        R: ValueRenderer<String>,
    {
        // Apply a reusable expectation.
        self.matches(matchers::field(
            "name",
            |person: &Person| &person.name,
            matchers::IsNotEmpty,
        ))
    }
}

assert_that!(Person { name: "Ada".into(), age: 36 }).is_adult().has_name();

let failures = assert_that!(Person { name: "".into(), age: 16 })
    .capture(|person| person.is_adult().has_name());
assert_that!(failures).has_length(2);
```

A method that only delegates must not call [`AssertThat::track_assertion`] itself. The delegated
assertions already count.

Some methods must do something an expectation cannot, such as calling the subject or awaiting it.
Such a method calls [`AssertThat::track_assertion`] before doing the work, builds a failure with
[`AssertThat::failure`], and reports it with [`AssertThat::raise`]. [`AssertThat::render`] shows a
complete example.

Assertr's own `*Assertions` traits are public for method discovery only. Do not implement them for
your types. See [API stability](#api-stability).

### Implement an expectation

Implement [`Expectation`](expectation::Expectation) when a check needs its own failure report, or when it produces a value
worth keeping, such as a parsed number, an error, or a lock guard. An implementation has two parts:

- [`Expectation::evaluate`](expectation::Expectation::evaluate) inspects the subject and returns `Ok` with the observed value, or `Err`
  with the reason for rejection. Use `()` when there is nothing to keep.
- [`Expectation::explain`](expectation::Expectation::explain) fills in a [`FailureBuilder`](failure::FailureBuilder). It receives the
  subject and the rejection from `evaluate`, or `None` when there was no subject, for example a
  missing collection element. It must not evaluate again.

Neither part tracks or raises. The chain does that. Set `KIND` when a [`FailureKind`](failure::FailureKind) fits better
than the default [`FailureKind::Predicate`](failure::FailureKind::Predicate).

This expectation keeps the parsed port on success and the parse error on rejection:

```
use assertr::prelude::*;
use assertr::{expectation::AssertionContext, expectation::Expectation, failure::Fact};
use assertr::failure::FailureBuilder;
use core::num::ParseIntError;

struct IsPort;

impl<R> Expectation<String, R> for IsPort
where
    R: ValueRenderer<String> + ValueRenderer<ParseIntError>,
{
    type Success<'a> = u16;
    type Rejection<'a> = ParseIntError;

    fn evaluate(&self, actual: &String, _: &AssertionContext<'_, R>) -> Result<u16, ParseIntError> {
        actual.parse()
    }

    fn explain(
        &self,
        rejected: Option<(&String, ParseIntError)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure.relation("is a port"),
            Some((actual, error)) => failure
                .actual(render.value(actual))
                .relation("is not a port")
                .fact(Fact::labelled("Error", render.value(&error))),
        }
    }
}

let address = String::from("8080");
let port = assert_that!(address).test_assertion(&IsPort);
assert_that!(port).is_equal_to(Some(8080));

let failures = assert_that!(String::from("http"))
    .capture(|it| it.matches(IsPort));
assert_that!(failures[0].to_string()).contains("is not a port");
```

[`AssertThat::test_assertion`] returns the observed value instead of continuing the chain. In
capture mode, it returns `None` after recording a failure.

When filling the builder:

- Set the [`actual`](failure::FailureBuilder::actual) value and a
  [`relation`](failure::FailureBuilder::relation), then an
  [`expected`](failure::FailureBuilder::expected) or
  [`unexpected`](failure::FailureBuilder::unexpected) value if there is one.
  [`relations`](failure::FailureBuilder::relations) sets the subject and the relation in one call.
- Write relations as lowercase phrases without values or a final period, such as
  `"does not end with"`.
- Add further evidence with [`fact`](failure::FailureBuilder::fact) using [`Fact::labelled`](failure::Fact::labelled) or
  [`Fact::note`](failure::Fact::note), and nested failures with [`children`](failure::FailureBuilder::children).
- Render every value through [`AssertionContext::render`](expectation::AssertionContext::render). This applies the user's renderer and
  rendering budget and keeps type information in the failure. Do not format values with `Debug`
  yourself.

Do not write the report text yourself. Assertr builds the report from these fields, so custom
assertions look like built-in ones.

### Structural evidence

The [rendering context](renderer::RenderingContext) renders whole collections, maps, and wrappers
while you provide renderers only for their leaves. Use
[`collection`](renderer::RenderingContext::collection) for a collection in its usual presentation,
[`stable_collection`](renderer::RenderingContext::stable_collection) when positions matter,
[`map`](renderer::RenderingContext::map) for maps, and
[`variant`](renderer::RenderingContext::variant) or
[`struct_field`](renderer::RenderingContext::struct_field) for a value inside a wrapper. The
[rendering guide](renderer#render-values-in-custom-assertions) lists all methods.

Because only leaves need renderers, this map expectation works even though neither the map nor
`Token` implements `Debug`:

```
# extern crate alloc;
use alloc::collections::BTreeMap;
use assertr::{prelude::*, expectation::AssertionContext, expectation::Expectation, failure::FailureKind};
use assertr::assertions::Map;
use assertr::failure::FailureBuilder;

struct IsEmptyMap;

impl<T: Map, R> Expectation<T, R> for IsEmptyMap
where
    R: ValueRenderer<T::Key> + ValueRenderer<T::Value>,
{
    type Success<'a> = () where T: 'a;
    type Rejection<'a> = () where T: 'a;

    const KIND: FailureKind = FailureKind::Length;

    fn evaluate(&self, actual: &T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.length() == 0 { Ok(()) } else { Err(()) }
    }

    fn explain(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            Some((actual, ())) => failure
                .actual(context.render().map(actual))
                .relation("is not empty"),
            None => failure.relation("is empty"),
        }
    }
}

struct Token(u32);

// One renderer supplies both leaf types: `u32` keys and `Token` values.
struct TokenRenderer;

impl ValueRenderer<Token> for TokenRenderer {
    fn fmt(&self, value: &Token, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "token({})", value.0)
    }
}

impl ValueRenderer<u32> for TokenRenderer {
    fn fmt(&self, value: &u32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "key({value})")
    }
}

let failures = assert_that!(BTreeMap::from([(1_u32, Token(7))]))
    .with_renderer(TokenRenderer)
    .capture(|it| it.matches(IsEmptyMap));
assert_that!(failures[0].to_string()).contains("key(1): token(7)");
```

For lists that do not exist as a value, such as the missing items of a comparison, use
[`borrowed_values`](renderer::RenderingContext::borrowed_values) or
[`entry_list`](renderer::RenderingContext::entry_list) and choose a
[`RenderingOrder`](renderer::RenderingOrder).

If you collect evidence yourself, keep at most
[`budget().max_items()`](renderer::RenderingContext::budget) entries and record the rest with
[`FailureBuilder::omitted_children`](failure::FailureBuilder::omitted_children). The budget limits
the report, never the result of the check. Inside an expectation, skip optional evidence when
[`AssertionContext::is_diagnostic`](expectation::AssertionContext::is_diagnostic) is false.
