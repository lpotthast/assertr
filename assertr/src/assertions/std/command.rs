use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use std::{ffi::OsStr, path::Path, process::Command};

use crate::{
    AssertThat,
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
    mode::Mode,
    renderer::{
        DebugRenderer, Rendered, RenderedBody, RenderingContext, RenderingOrder, ValueRenderer,
    },
};

/// A process command that exposes its [`std::process::Command`], the capability behind the
/// command assertions.
///
/// `std::process::Command` implements it, and so does `tokio::process::Command` with the
/// `tokio-process` feature. Implement it for your own command builder to make
/// [`CommandAssertions`] and [`CommandProjectionAssertions`] available on it:
///
/// ```
/// use std::process::Command;
///
/// use assertr::{assertions::StdCommand, prelude::*};
///
/// /// Builds the invocation of a tool.
/// struct Invocation(Command);
///
/// impl StdCommand for Invocation {
///     fn as_std(&self) -> &Command {
///         &self.0
///     }
/// }
///
/// let mut command = Command::new("git");
/// command.arg("status");
/// assert_that!(Invocation(command))
///     .has_program("git")
///     .has_arg("status");
/// ```
///
/// `StdCommand` is not part of the prelude, so import it from
/// [`assertr::assertions`](crate::assertions). Shared and mutable references to a command and
/// boxed commands are commands too.
pub trait StdCommand {
    /// The standard library command describing the program, its arguments, its environment
    /// changes, and its working directory.
    fn as_std(&self) -> &Command;
}

impl StdCommand for Command {
    fn as_std(&self) -> &Command {
        self
    }
}

#[cfg(feature = "tokio-process")]
impl StdCommand for tokio::process::Command {
    fn as_std(&self) -> &Command {
        tokio::process::Command::as_std(self)
    }
}

impl<C: StdCommand + ?Sized> StdCommand for &C {
    fn as_std(&self) -> &Command {
        C::as_std(self)
    }
}

impl<C: StdCommand + ?Sized> StdCommand for &mut C {
    fn as_std(&self) -> &Command {
        C::as_std(self)
    }
}

impl<C: StdCommand + ?Sized> StdCommand for Box<C> {
    fn as_std(&self) -> &Command {
        C::as_std(self)
    }
}

/// Checks command arguments and retains their observed views on rejection.
#[derive(Debug, Clone)]
pub struct HasArg<E>(E);

impl<E> HasArg<E> {
    /// Expects this argument.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<C, E, R> Expectation<C, R> for HasArg<E>
where
    C: StdCommand + ?Sized,
    E: AsRef<OsStr>,
    R: ValueRenderer<OsStr>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = (Vec<&'a OsStr>, &'a OsStr)
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.0.as_ref();
        let command = actual.as_std();
        if command.get_args().any(|arg| arg == expected) {
            Ok(())
        } else {
            // Only a rejection retains the argument list for its diagnostic.
            Err((command.get_args().collect(), expected))
        }
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure
                .relation("contains")
                .expected(render.value(self.0.as_ref())),
            Some((_, (args, expected))) => failure
                .actual(render_args(render, &args))
                .relation("does not contain")
                .expected(render.value(expected)),
        }
    }
}

/// Rejects commands passing an unexpected argument, retaining the argument list on rejection.
#[derive(Debug, Clone)]
pub struct DoesNotHaveArg<E>(E);

impl<E> DoesNotHaveArg<E> {
    /// Expects no argument equal to `unexpected`.
    #[must_use]
    pub const fn new(unexpected: E) -> Self {
        Self(unexpected)
    }
}

impl<C, E, R> Expectation<C, R> for DoesNotHaveArg<E>
where
    C: StdCommand + ?Sized,
    E: AsRef<OsStr>,
    R: ValueRenderer<OsStr>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = (Vec<&'a OsStr>, &'a OsStr)
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let unexpected = self.0.as_ref();
        let command = actual.as_std();
        if command.get_args().any(|arg| arg == unexpected) {
            Err((command.get_args().collect(), unexpected))
        } else {
            Ok(())
        }
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure
                .relation("does not contain")
                .unexpected(render.value(self.0.as_ref())),
            Some((_, (args, unexpected))) => failure
                .actual(render_args(render, &args))
                .relation("contains")
                .unexpected(render.value(unexpected)),
        }
    }
}

/// Compares the program a command runs, retaining the actual program on rejection.
#[derive(Debug, Clone)]
pub struct HasProgram<E>(E);

impl<E> HasProgram<E> {
    /// Expects this program.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<C, E, R> Expectation<C, R> for HasProgram<E>
where
    C: StdCommand + ?Sized,
    E: AsRef<OsStr>,
    R: ValueRenderer<OsStr>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = &'a OsStr
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let program = actual.as_std().get_program();
        if program == self.0.as_ref() {
            Ok(())
        } else {
            Err(program)
        }
    }

    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure
            .relations(
                rejected.map(|(_, program)| render.value(program)),
                "has the expected program",
                "is not the expected program",
            )
            .expected(render.value(self.0.as_ref()))
    }
}

/// Checks that a command explicitly sets an environment variable to a value, retaining the
/// command's environment changes on rejection.
#[derive(Debug, Clone)]
pub struct HasEnv<K, V> {
    key: K,
    value: V,
}

impl<K, V> HasEnv<K, V> {
    /// Expects the variable `key` to be set to `value`.
    #[must_use]
    pub const fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<C, K, V, R> Expectation<C, R> for HasEnv<K, V>
where
    C: StdCommand + ?Sized,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
    R: ValueRenderer<OsStr>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = Vec<(&'a OsStr, Option<&'a OsStr>)>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let (key, value) = (self.key.as_ref(), self.value.as_ref());
        let command = actual.as_std();
        let mut envs = command.get_envs();
        if envs.any(|(name, set)| name == key && set == Some(value)) {
            Ok(())
        } else {
            Err(command.get_envs().collect())
        }
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let expected = Rendered::from((
            render.value(self.key.as_ref()),
            render.value(self.value.as_ref()),
        ));
        failure
            .relations(
                rejected.map(|(_, envs)| render_envs(render, &envs)),
                "sets the environment variable",
                "does not set the environment variable",
            )
            .expected(expected)
    }
}

/// Renders arguments in their order, as a list of `OsStr` values.
fn render_args<R: ValueRenderer<OsStr>>(
    render: RenderingContext<'_, R>,
    args: &Vec<&OsStr>,
) -> Rendered {
    render.borrowed_values::<OsStr, _>(args, RenderingOrder::PreserveIteration)
}

/// Renders environment changes in their order, as `(key, value)` tuples. A removed variable
/// shows `<removed>` in place of its value.
fn render_envs<R: ValueRenderer<OsStr>>(
    render: RenderingContext<'_, R>,
    envs: &[(&OsStr, Option<&OsStr>)],
) -> Rendered {
    render.structured_list(
        envs.iter().map(|&(key, value)| {
            let value = value.map_or_else(
                || Rendered::untyped(RenderedBody::Placeholder("<removed>")),
                |value| render.value(value),
            );
            Rendered::from((render.value(key), value))
        }),
        envs.len(),
        RenderingOrder::PreserveIteration,
    )
}

/// Assertions for process commands, such as [`std::process::Command`].
///
/// They only inspect the command. Nothing is spawned.
///
/// ```
/// use std::process::Command;
///
/// use assertr::prelude::*;
///
/// let mut command = Command::new("cargo");
/// command.arg("build").env("RUSTFLAGS", "-Dwarnings");
/// assert_that!(command)
///     .has_program("cargo")
///     .has_arg("build")
///     .does_not_have_arg("--release")
///     .has_env("RUSTFLAGS", "-Dwarnings");
/// ```
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait CommandAssertions<R = DebugRenderer> {
    /// Asserts that the command contains `expected` in its argument list.
    fn has_arg(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>;

    /// Asserts that the command does not contain `unexpected` in its argument list.
    fn does_not_have_arg(self, unexpected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>;

    /// Asserts that the command runs the program `expected`, compared exactly as given to
    /// [`Command::new`], without resolving it through `PATH`.
    fn has_program(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>;

    /// Asserts that the command explicitly sets the environment variable `key` to `value`.
    ///
    /// Only changes made through methods such as [`Command::env`] count. A variable the command
    /// would merely inherit from the parent process does not.
    fn has_env(self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>;
}

impl<C: StdCommand, M: Mode, R> CommandAssertions<R> for AssertThat<'_, C, M, R> {
    #[track_caller]
    fn has_arg(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>,
    {
        self.matches(HasArg::new(expected))
    }

    #[track_caller]
    fn does_not_have_arg(self, unexpected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>,
    {
        self.matches(DoesNotHaveArg::new(unexpected))
    }

    #[track_caller]
    fn has_program(self, expected: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>,
    {
        self.matches(HasProgram::new(expected))
    }

    #[track_caller]
    fn has_env(self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self
    where
        R: ValueRenderer<OsStr>,
    {
        self.matches(HasEnv::new(key, value))
    }
}

/// Projections of a process command onto its program, arguments, environment changes, and
/// working directory.
///
/// Each method borrows the assertion chain and returns a chain over the selected part, like
/// [`AssertThat::derive_owned`], so the general assertion families apply to it. Failures
/// propagate to the original chain. Projecting does not count as an assertion and works in every
/// mode.
///
/// Checks such as [`CommandAssertions::has_arg`] consume the chain and return it, so bind
/// the chain they return to project from it afterwards:
///
/// ```
/// use std::{ffi::OsStr, path::Path, process::Command};
///
/// use assertr::prelude::*;
///
/// let mut command = Command::new("qemu");
/// command
///     .args(["-accel", "kvm"])
///     .env("LANG", "C")
///     .env_remove("DISPLAY")
///     .current_dir("/tmp");
/// let it = assert_that!(command).has_arg("kvm").does_not_have_arg("-snapshot");
/// it.program().is_equal_to("qemu");
/// it.args().contains_contiguous(["-accel", "kvm"]);
/// it.envs()
///     .contains_entry(OsStr::new("LANG"), Some(OsStr::new("C")))
///     .contains_entry(OsStr::new("DISPLAY"), None);
/// it.current_dir().is_equal_to(Some(Path::new("/tmp")));
/// ```
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait CommandProjectionAssertions<'t, M: Mode, R = DebugRenderer> {
    /// Returns an assertion over the program, as given to [`Command::new`].
    fn program(&'t self) -> AssertThat<'t, &'t OsStr, M, R>
    where
        R: Clone;

    /// Returns an assertion over the arguments in order, without the program.
    fn args(&'t self) -> AssertThat<'t, Vec<&'t OsStr>, M, R>
    where
        R: Clone;

    /// Returns an assertion over the explicit environment changes by variable name. A variable
    /// set with [`Command::env`] maps to `Some(value)`. A variable removed with
    /// [`Command::env_remove`] maps to `None`. Inherited variables are not included.
    fn envs(&'t self) -> AssertThat<'t, BTreeMap<&'t OsStr, Option<&'t OsStr>>, M, R>
    where
        R: Clone;

    /// Returns an assertion over the working directory set with [`Command::current_dir`], if
    /// any.
    fn current_dir(&'t self) -> AssertThat<'t, Option<&'t Path>, M, R>
    where
        R: Clone;
}

impl<'t, C: StdCommand, M: Mode, R> CommandProjectionAssertions<'t, M, R>
    for AssertThat<'t, C, M, R>
{
    fn program(&'t self) -> AssertThat<'t, &'t OsStr, M, R>
    where
        R: Clone,
    {
        self.derive_owned_as("program()", |command| command.as_std().get_program())
    }

    fn args(&'t self) -> AssertThat<'t, Vec<&'t OsStr>, M, R>
    where
        R: Clone,
    {
        self.derive_owned_as("args()", |command| command.as_std().get_args().collect())
    }

    fn envs(&'t self) -> AssertThat<'t, BTreeMap<&'t OsStr, Option<&'t OsStr>>, M, R>
    where
        R: Clone,
    {
        self.derive_owned_as("envs()", |command| command.as_std().get_envs().collect())
    }

    fn current_dir(&'t self) -> AssertThat<'t, Option<&'t Path>, M, R>
    where
        R: Clone,
    {
        self.derive_owned_as("current_dir()", |command| {
            command.as_std().get_current_dir()
        })
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    fn cargo_build() -> Command {
        let mut command = Command::new("cargo");
        command
            .arg("build")
            .arg("--locked")
            .env("RUSTFLAGS", "-Dwarnings")
            .env_remove("CARGO_TARGET_DIR");
        command
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::cargo_build;
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            cargo_build()
                .must()
                .have_arg("build")
                .not_have_arg("--release")
                .have_program("cargo")
                .have_env("RUSTFLAGS", "-Dwarnings");
        }
    }

    mod renderer_contract {
        use std::process::Command;

        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Command, Panic, NoRenderer>
                    => CommandAssertions<NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Command, Capture, NoRenderer>
                    => CommandProjectionAssertions<'static, Capture, NoRenderer>
            );
        }

        #[test]
        fn failures_render_arguments_with_the_active_renderer() {
            let mut command = Command::new("program");
            command.arg("--actual");

            let failures = assert_that!(command)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.has_arg("--expected"));

            assert_that!(failures[0].to_string()).contains(SENTINEL);
        }
    }

    mod command_adapters {
        use std::process::Command;

        use super::cargo_build;
        use crate::{assertions::StdCommand, prelude::*};

        struct Invocation(Command);

        impl StdCommand for Invocation {
            fn as_std(&self) -> &Command {
                &self.0
            }
        }

        #[test]
        fn custom_references_and_boxed_commands_get_every_assertion() {
            assert_that!(Invocation(cargo_build())).has_program("cargo");
            assert_that_owned!(&cargo_build()).has_arg("build");
            assert_that_owned!(&mut cargo_build()).has_arg("--locked");
            assert_that!(Box::new(cargo_build())).has_env("RUSTFLAGS", "-Dwarnings");
        }

        #[cfg(feature = "tokio-process")]
        #[test]
        fn tokio_commands_get_every_assertion() {
            let mut command = tokio::process::Command::new("cargo");
            command.arg("build");
            let it = assert_that!(command).has_program("cargo").has_arg("build");
            it.args().has_length(1);
        }
    }

    mod has_arg {
        use std::process::Command;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar");
            assert_caller_location!(assert_that!(cmd), has_arg("help"));
        }

        #[test]
        fn succeeds_when_arg_present() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar").arg("--baz");

            assert_that!(cmd).has_arg("--bar").has_arg("--baz");
        }

        #[test]
        fn panics_when_arg_is_not_present() {
            let mut cmd = Command::new("foo");
            cmd.arg("--bar");

            assert_that!(|| {
                assert_that!(cmd).with_location(false).has_arg("help");
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `cmd`

                Actual: [
                    "--bar",
                ]

                does not contain

                Expected: "help"
                -------- assertr --------
            "#});
        }
    }

    mod does_not_have_arg {
        use indoc::formatdoc;

        use super::cargo_build;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(cargo_build()), does_not_have_arg("build"));
        }

        #[test]
        fn succeeds_when_arg_absent() {
            assert_that!(cargo_build())
                .does_not_have_arg("--release")
                .does_not_have_arg("cargo");
        }

        #[test]
        fn panics_when_arg_is_present() {
            assert_that!(|| {
                assert_that!(cargo_build())
                    .with_location(false)
                    .does_not_have_arg("--locked");
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `cargo_build()`

                Actual: [
                    "build",
                    "--locked",
                ]

                contains

                Unexpected: "--locked"
                -------- assertr --------
            "#});
        }
    }

    mod has_program {
        use indoc::formatdoc;

        use super::cargo_build;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(cargo_build()), has_program("rustc"));
        }

        #[test]
        fn compares_the_program_as_given() {
            assert_that!(cargo_build()).has_program("cargo");
            let failures = assert_that!(cargo_build()).capture(|it| it.has_program("carg"));
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn panics_when_program_differs() {
            assert_that!(|| {
                assert_that!(cargo_build())
                    .with_location(false)
                    .has_program("rustc");
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `cargo_build()`

                Actual: "cargo"

                is not the expected program

                Expected: "rustc"
                -------- assertr --------
            "#});
        }
    }

    mod has_env {
        use indoc::formatdoc;

        use super::cargo_build;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(cargo_build()), has_env("RUSTFLAGS", ""));
        }

        #[test]
        fn succeeds_when_the_variable_is_set_to_the_value() {
            assert_that!(cargo_build()).has_env("RUSTFLAGS", "-Dwarnings");
        }

        #[test]
        fn rejects_other_values_removed_and_inherited_variables() {
            let failures = assert_that!(cargo_build()).capture(|it| {
                it.has_env("RUSTFLAGS", "-Awarnings")
                    .has_env("CARGO_TARGET_DIR", "target")
                    .has_env("PATH", "/bin")
            });
            assert_that!(failures).has_length(3);
        }

        #[test]
        fn panics_with_every_environment_change() {
            assert_that!(|| {
                assert_that!(cargo_build())
                    .with_location(false)
                    .has_env("RUSTFLAGS", "-Awarnings");
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `cargo_build()`

                Actual: [
                    (
                        "CARGO_TARGET_DIR",
                        <removed>,
                    ),
                    (
                        "RUSTFLAGS",
                        "-Dwarnings",
                    ),
                ]

                does not set the environment variable

                Expected: (
                    "RUSTFLAGS",
                    "-Awarnings",
                )
                -------- assertr --------
            "#});
        }
    }

    mod program {
        use std::ffi::OsStr;

        use super::cargo_build;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(cargo_build()).program(),
                is_equal_to(OsStr::new("rustc"))
            );
        }

        #[test]
        fn continues_with_the_program() {
            assert_that!(cargo_build())
                .program()
                .is_equal_to(OsStr::new("cargo"))
                .is_equal_to("cargo");
        }
    }

    mod args {
        use std::ffi::OsStr;

        use super::cargo_build;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(cargo_build()).args(), has_length(3));
        }

        #[test]
        fn continues_with_the_arguments_in_order() {
            assert_that!(cargo_build())
                .args()
                .contains_exactly([OsStr::new("build"), OsStr::new("--locked")])
                .contains_exactly(["build", "--locked"]);
        }
    }

    mod envs {
        use std::ffi::OsStr;

        use super::cargo_build;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(cargo_build()).envs(), has_length(3));
        }

        #[test]
        fn continues_with_set_and_removed_variables() {
            assert_that!(cargo_build())
                .envs()
                .contains_exactly_entries([
                    (OsStr::new("CARGO_TARGET_DIR"), None),
                    (OsStr::new("RUSTFLAGS"), Some(OsStr::new("-Dwarnings"))),
                ]);
        }
    }

    mod current_dir {
        use std::{path::Path, process::Command};

        use super::cargo_build;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(cargo_build()).current_dir(), is_some());
        }

        #[test]
        fn continues_with_the_working_directory() {
            assert_that!(cargo_build()).current_dir().is_none();
            let mut command = Command::new("ls");
            command.current_dir("/tmp");
            assert_that!(command)
                .current_dir()
                .is_equal_to(Some(Path::new("/tmp")));
        }
    }
}
