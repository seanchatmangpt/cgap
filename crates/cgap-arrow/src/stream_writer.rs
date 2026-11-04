//! StreamWriter: raise the writev handoff witness (lane cl-l, W1) into a
//! reusable type (lane e80-06, RAISE).
//!
//! The witness proved the framing bytes survive a real kernel `writev`
//! across a live UDS by driving `ipc::emit` + `UnixStream::write_vectored`
//! by hand in a test. This module raises that exact pattern: emit the
//! IPC1 window (continuation / metadata_size / sealed 64-byte metadata /
//! body / zero pad / EOS — the byte sequence W1 pinned) into a caller
//! scratch buffer, then scatter it to a generic sink via vectored writes,
//! advancing past whatever the sink accepted (partial accepts happen; a
//! zero-byte accept is a typed refusal, not a spin).
//!
//! `no_std` discipline: the sink is the crate-local [`VectoredSink`] trait
//! (kernel-style `write_vectored` with a typed associated error) — no
//! `std::io`, no alloc (the window lives in a caller scratch buffer, the
//! ipc.rs caller-buffer convention). Socket binding stays in the test's
//! std context; the test crate implements [`VectoredSink`] for
//! `UnixStream` there.
//!
//! Sequence law raised with the type: `MetaHeader.seq` is the writer's
//! monotonic message sequence — `emit` stamps the writer's `seq` over any
//! caller-supplied value and advances it after each fully-written message.

use crate::ipc::{emit, meta_span_for, IpcError, MetaHeader};
use core::fmt;

/// Kernel-style vectored write surface for a byte sink.
///
/// Crate-local (`no_std`, zero deps): hosts implement this for their
/// transport (a std test implements it for `UnixStream`'s
/// `write_vectored`; an embedded host maps it to its DMA/UART scatter).
pub trait VectoredSink {
    /// Typed sink error.
    type Error;
    /// Scatter `bufs` to the sink. Returns the number of bytes accepted —
    /// possibly fewer than the total (partial accept); the caller
    /// re-scatters the remainder. Must be `> 0` on success.
    fn write_vectored(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error>;
}

impl<T: VectoredSink + ?Sized> VectoredSink for &mut T {
    type Error = T::Error;
    fn write_vectored(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error> {
        (**self).write_vectored(bufs)
    }
}

/// Why a `StreamWriter` emit refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamError<E> {
    /// IPC1 framing refused by [`crate::ipc::emit`] (alignment, body_len,
    /// scratch truncation).
    Framing(IpcError),
    /// The sink refused the handoff.
    Sink(E),
    /// The sink accepted zero bytes: the advance loop would spin forever,
    /// so the type refuses instead of looping (the witness's
    /// `assert!(n > 0)` raised into the error surface).
    ZeroProgress,
}

impl<E: fmt::Display> fmt::Display for StreamError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamError::Framing(e) => write!(f, "framing: {e}"),
            StreamError::Sink(e) => write!(f, "sink: {e}"),
            StreamError::ZeroProgress => f.write_str("sink accepted zero bytes (would spin)"),
        }
    }
}

#[cfg(not(target_os = "none"))]
impl<E> core::error::Error for StreamError<E> where E: fmt::Debug + fmt::Display {}

/// Writev handoff of IPC1 windows to a generic [`VectoredSink`].
///
/// Owns the monotonic sequence stamp and the partial-accept advance loop
/// the witness wrote by hand.
#[derive(Debug)]
pub struct StreamWriter<S> {
    sink: S,
    seq: u32,
}

impl<S> StreamWriter<S> {
    /// New writer starting at sequence 0.
    pub fn new(sink: S) -> Self {
        Self { sink, seq: 0 }
    }

    /// New writer starting at `seq` (e.g. to resume a stream mid-flight).
    pub fn with_seq(sink: S, seq: u32) -> Self {
        Self { sink, seq }
    }

    /// The sequence the next emitted message will carry.
    pub fn seq(&self) -> u32 {
        self.seq
    }

    /// Recover the sink.
    pub fn into_inner(self) -> S {
        self.sink
    }
}

impl<S: VectoredSink> StreamWriter<S> {
    /// Emit one encapsulated IPC1 message (`meta` + `body`) into `scratch`
    /// via [`crate::ipc::emit`], then hand the window off to the sink with
    /// vectored writes over the four wire segments (prefix / metadata /
    /// body / zero-pad + EOS), advancing past partial accepts until the
    /// whole window is accepted. Returns the bytes handed to the sink
    /// ([`crate::ipc::window_len`]).
    ///
    /// `meta.seq` is overridden with the writer's sequence, which advances
    /// by one after the message is fully written.
    pub fn emit(
        &mut self,
        scratch: &mut [u8],
        mut meta: MetaHeader,
        body: &[u8],
    ) -> Result<usize, StreamError<S::Error>> {
        meta.seq = self.seq;
        let n = emit(scratch, &meta, body).map_err(StreamError::Framing)?;
        let alignment = meta.alignment as usize;
        let meta_span = meta_span_for(alignment);
        let body_len = body.len();
        // Segment boundaries in the window (ipc.rs layout): prefix,
        // metadata, body, body padding + EOS.
        let starts = [
            0,
            PREFIX_END,
            PREFIX_END + meta_span,
            PREFIX_END + meta_span + body_len,
        ];
        let ends = [
            PREFIX_END,
            PREFIX_END + meta_span,
            PREFIX_END + meta_span + body_len,
            n,
        ];
        let mut written = 0usize;
        while written < n {
            let mut segs: [&[u8]; 4] = [&[], &[], &[], &[]];
            let mut k = 0usize;
            for seg in 0..4 {
                let lo = written.max(starts[seg]);
                let hi = ends[seg];
                if hi > lo {
                    segs[k] = &scratch[lo..hi];
                    k += 1;
                }
            }
            let accepted = self
                .sink
                .write_vectored(&segs[..k])
                .map_err(StreamError::Sink)?;
            if accepted == 0 {
                return Err(StreamError::ZeroProgress);
            }
            written += accepted;
        }
        self.seq = self.seq.wrapping_add(1);
        Ok(n)
    }
}

/// Continuation + metadata_size words (ipc.rs `PREFIX_SIZE`).
const PREFIX_END: usize = crate::ipc::PREFIX_SIZE;
