//! Explicit writing of adapter input to a configured I/O target.

use core::cell::RefCell;
use std::io::{self, Write};

use super::super::Adapter;

/// Writes text or bytes to a configured target as a side effect.
///
/// With a [`Write`] target, this implements [`Adapter`] for any input implementing
/// `AsRef<[u8]>`, including [`HumanReadableText`](super::HumanReadableText), strings, and byte
/// buffers. Each call writes the complete input without adding a newline, then flushes the
/// target. I/O errors are returned to the caller. A failed write may leave partial output.
///
/// Use [`new`](Self::new) with an owned or borrowed target, or choose [`stdout`](Self::stdout)
/// or [`stderr`](Self::stderr). With the `tokio` feature, `tokio_stdout` and `tokio_stderr`
/// construct asynchronous targets for `adapt_async`. The synchronous adapter chain does not
/// drive asynchronous I/O.
///
/// Writing is explicit. Capture and panic presentation never select this sink automatically.
/// Its `()` output cannot be installed as a panic presentation.
///
/// ```
/// # #[cfg(feature = "std")] {
/// use assertr::failure::adapter::{Adapter, AdapterExt, ToHumanReadableText, Writer};
/// use assertr::prelude::*;
///
/// let failures = assert_that!(1).capture(|it| it.is_equal_to(2));
/// let mut output = Vec::new();
/// ToHumanReadableText.then(Writer::new(&mut output)).adapt(&failures).unwrap();
/// assert_that!(String::from_utf8(output).unwrap()).contains("Expected: 2");
/// # }
/// ```
#[derive(Debug)]
#[must_use]
pub struct Writer<W> {
    target: RefCell<W>,
}

impl<W> Writer<W> {
    /// Creates a writer around an owned or borrowed target without performing I/O.
    pub const fn new(target: W) -> Self {
        Self {
            target: RefCell::new(target),
        }
    }

    /// Borrows the target exclusively, for example to inspect a memory buffer.
    pub fn get_mut(&mut self) -> &mut W {
        self.target.get_mut()
    }

    /// Consumes the adapter and returns its target without performing additional I/O.
    pub fn into_inner(self) -> W {
        self.target.into_inner()
    }
}

impl Writer<io::Stdout> {
    /// Creates a writer targeting [`io::stdout`].
    pub fn stdout() -> Self {
        Self::new(io::stdout())
    }
}

impl Writer<io::Stderr> {
    /// Creates a writer targeting [`io::stderr`].
    pub fn stderr() -> Self {
        Self::new(io::stderr())
    }
}

impl<Input: AsRef<[u8]> + ?Sized, W: Write> Adapter<Input> for Writer<W> {
    type Output = ();
    type Error = io::Error;

    /// Writes all input bytes and flushes the target on the calling thread.
    ///
    /// # Errors
    ///
    /// Returns write or flush errors unchanged. Reentrant use of this writer returns an I/O
    /// error because its target is already borrowed by the outer call.
    fn adapt(&self, input: &Input) -> io::Result<()> {
        let bytes = input.as_ref();
        let mut target = self
            .target
            .try_borrow_mut()
            .map_err(|error| io::Error::other(error.to_string()))?;
        target.write_all(bytes)?;
        target.flush()
    }
}

#[cfg(feature = "tokio")]
impl Writer<tokio::io::Stdout> {
    /// Creates a writer targeting [`tokio::io::stdout`] for [`adapt_async`](Self::adapt_async).
    pub fn tokio_stdout() -> Self {
        Self::new(tokio::io::stdout())
    }
}

#[cfg(feature = "tokio")]
impl Writer<tokio::io::Stderr> {
    /// Creates a writer targeting [`tokio::io::stderr`] for [`adapt_async`](Self::adapt_async).
    pub fn tokio_stderr() -> Self {
        Self::new(tokio::io::stderr())
    }
}

#[cfg(feature = "tokio")]
impl<W: tokio::io::AsyncWrite + Unpin> Writer<W> {
    /// Writes all input bytes asynchronously and flushes the target.
    ///
    /// Call this explicitly after rendering or other synchronous adaptation. It requires
    /// exclusive access to the writer and does not create or block on a runtime. Targets such
    /// as Tokio's standard streams require the caller to run inside a Tokio runtime.
    ///
    /// # Errors
    ///
    /// Returns write or flush errors unchanged. Errors or cancellation may leave partial output.
    /// Retrying writes the input from the beginning.
    ///
    /// ```no_run
    /// use assertr::failure::adapter::{Adapter, ToHumanReadableText, Writer};
    /// use assertr::prelude::*;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let failures = assert_that!(1).capture(|it| it.is_equal_to(2));
    /// let text = ToHumanReadableText.adapt(&failures).unwrap();
    /// Writer::tokio_stderr().adapt_async(&text).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn adapt_async<Input: AsRef<[u8]> + ?Sized>(
        &mut self,
        input: &Input,
    ) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;

        let target = self.target.get_mut();
        target.write_all(input.as_ref()).await?;
        target.flush().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::failure::adapter::{AdapterExt, HumanReadableText, ThenError, ToHumanReadableText};
    use crate::failure::{FailureBuilder, FailureKind};
    use crate::prelude::*;

    #[derive(Default)]
    struct Target {
        bytes: Vec<u8>,
        flushes: usize,
        write_error: Option<io::ErrorKind>,
        flush_error: Option<io::ErrorKind>,
    }

    impl Write for Target {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if let Some(error) = self.write_error.take() {
                return Err(error.into());
            }
            // Exercise completion across short writes, including multibyte text.
            let length = bytes.len().min(2);
            self.bytes.extend_from_slice(&bytes[..length]);
            Ok(length)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            match self.flush_error {
                Some(error) => Err(error.into()),
                None => Ok(()),
            }
        }
    }

    mod adapt {
        use super::*;

        #[test]
        fn writes_text_and_bytes_unchanged_and_flushes_each_call() {
            let writer = Writer::new(Target::default());
            writer.adapt(&HumanReadableText::new("é\n")).unwrap();
            writer.adapt("text").unwrap();
            writer.adapt(&String::from("owned")).unwrap();
            writer.adapt(&[0, 255][..]).unwrap();
            writer.adapt("").unwrap();

            let target = writer.into_inner();
            assert_that!(target.bytes).is_equal_to(b"\xc3\xa9\ntextowned\0\xff");
            assert_that!(target.flushes).is_equal_to(5);
        }

        #[test]
        fn composes_with_rendering_into_a_borrowed_target() {
            let failure = FailureBuilder::detached::<i32>(FailureKind::Equality)
                .actual(1)
                .expected(2)
                .build();
            let mut bytes = Vec::new();
            ToHumanReadableText
                .then(Writer::new(&mut bytes))
                .adapt(&failure)
                .unwrap();

            assert_that!(bytes)
                .is_equal_to(ToHumanReadableText.render(&failure).as_str().as_bytes());
        }

        #[test]
        fn retries_interrupted_writes() {
            let writer = Writer::new(Target {
                write_error: Some(io::ErrorKind::Interrupted),
                ..Target::default()
            });
            writer.adapt("complete").unwrap();
            let target = writer.into_inner();
            assert_that!(target.bytes).is_equal_to(b"complete");
            assert_that!(target.flushes).is_equal_to(1);
        }

        #[test]
        fn preserves_write_errors_without_flushing() {
            let writer = Writer::new(Target {
                write_error: Some(io::ErrorKind::BrokenPipe),
                ..Target::default()
            });
            let error = writer.adapt("text").unwrap_err();
            assert_that!(error.kind()).is_equal_to(io::ErrorKind::BrokenPipe);
            let target = writer.into_inner();
            assert_that!(target.bytes).is_empty();
            assert_that!(target.flushes).is_equal_to(0);
        }

        #[test]
        fn preserves_flush_errors_through_composition() {
            let writer = Writer::new(Target {
                flush_error: Some(io::ErrorKind::PermissionDenied),
                ..Target::default()
            });
            let failure = FailureBuilder::detached::<i32>(FailureKind::Other).build();
            let error = ToHumanReadableText
                .then(&writer)
                .adapt(&failure)
                .unwrap_err();
            let ThenError::Next(error) = error;
            assert_that!(error.kind()).is_equal_to(io::ErrorKind::PermissionDenied);
            let target = writer.into_inner();
            assert_that!(target.bytes)
                .is_equal_to(ToHumanReadableText.render(&failure).as_str().as_bytes());
            assert_that!(target.flushes).is_equal_to(1);
        }

        #[test]
        fn reports_write_zero_from_a_full_target() {
            let mut bytes = [0; 2];
            let writer = Writer::new(&mut bytes[..]);
            assert_that!(writer.adapt("longer").unwrap_err().kind())
                .is_equal_to(io::ErrorKind::WriteZero);
            drop(writer);
            assert_that!(bytes).is_equal_to(*b"lo");
        }

        #[test]
        fn an_active_borrow_returns_an_error_without_panicking() {
            let writer = Writer::new(Target::default());
            let active = writer.target.borrow_mut();
            assert_that!(writer.adapt("text").unwrap_err().kind())
                .is_equal_to(io::ErrorKind::Other);
            drop(active);
            writer.adapt("text").unwrap();
            assert_that!(writer.into_inner().bytes).is_equal_to(b"text");
        }
    }

    mod targets {
        use super::*;

        #[test]
        fn constructing_and_recovering_a_target_performs_no_io() {
            let mut writer = Writer::new(Target::default());
            writer.get_mut().bytes.extend_from_slice(b"existing");
            let target = writer.into_inner();
            assert_that!(target.bytes).is_equal_to(b"existing");
            assert_that!(target.flushes).is_equal_to(0);
        }

        #[test]
        fn standard_stream_constructors_select_their_targets() {
            let _: io::Stdout = Writer::stdout().into_inner();
            let _: io::Stderr = Writer::stderr().into_inner();
        }
    }

    #[cfg(feature = "tokio")]
    mod adapt_async {
        use core::{
            pin::Pin,
            task::{Context, Poll},
        };
        use tokio::io::AsyncWrite;

        use super::*;

        #[derive(Default)]
        struct AsyncTarget {
            inner: Target,
            pending: bool,
        }

        impl AsyncWrite for AsyncTarget {
            fn poll_write(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                bytes: &[u8],
            ) -> Poll<io::Result<usize>> {
                self.pending = !self.pending;
                if self.pending {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(self.inner.write(bytes))
            }

            fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
                self.pending = !self.pending;
                if self.pending {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(self.inner.flush())
            }

            fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
                unreachable!("adaptation must not shut down the target")
            }
        }

        #[tokio::test]
        async fn awaits_short_writes_and_flushes_then_reuses_the_target() {
            let mut writer = Writer::new(AsyncTarget::default());
            writer
                .adapt_async(&HumanReadableText::new("é\n"))
                .await
                .unwrap();
            writer.adapt_async(&[0, 255][..]).await.unwrap();
            writer.adapt_async("").await.unwrap();

            let target = writer.into_inner().inner;
            assert_that!(target.bytes).is_equal_to(b"\xc3\xa9\n\0\xff");
            assert_that!(target.flushes).is_equal_to(3);
        }

        #[tokio::test]
        async fn accepts_a_borrowed_async_target_and_produces_a_send_future() {
            fn require_send<F: Future + Send>(future: F) -> F {
                future
            }

            let mut target = AsyncTarget::default();
            require_send(Writer::new(&mut target).adapt_async("text"))
                .await
                .unwrap();
            assert_that!(target.inner.bytes).is_equal_to(b"text");
            assert_that!(target.inner.flushes).is_equal_to(1);
        }

        #[tokio::test]
        async fn preserves_write_errors_without_flushing() {
            let mut writer = Writer::new(AsyncTarget {
                inner: Target {
                    write_error: Some(io::ErrorKind::BrokenPipe),
                    ..Target::default()
                },
                ..AsyncTarget::default()
            });
            assert_that!(writer.adapt_async("text").await.unwrap_err().kind())
                .is_equal_to(io::ErrorKind::BrokenPipe);
            let target = writer.into_inner().inner;
            assert_that!(target.bytes).is_empty();
            assert_that!(target.flushes).is_equal_to(0);
        }

        #[tokio::test]
        async fn preserves_flush_errors_after_writing_the_complete_input() {
            let mut writer = Writer::new(AsyncTarget {
                inner: Target {
                    flush_error: Some(io::ErrorKind::PermissionDenied),
                    ..Target::default()
                },
                ..AsyncTarget::default()
            });
            assert_that!(writer.adapt_async("text").await.unwrap_err().kind())
                .is_equal_to(io::ErrorKind::PermissionDenied);
            let target = writer.into_inner().inner;
            assert_that!(target.bytes).is_equal_to(b"text");
            assert_that!(target.flushes).is_equal_to(1);
        }

        #[test]
        fn standard_stream_constructors_select_tokio_targets_without_a_runtime() {
            let _: tokio::io::Stdout = Writer::tokio_stdout().into_inner();
            let _: tokio::io::Stderr = Writer::tokio_stderr().into_inner();
        }
    }
}
