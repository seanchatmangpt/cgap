//! Minimal MCP-shaped request driver for cgap-git (lane GIT-14).
//!
//! ## Divergence from MCP (prominent, deliberate)
//!
//! MCP on the wire is JSON-RPC 2.0. JSON-RPC requires a JSON parser,
//! which requires heap-backed recursive-descent parsing and `std`-grade
//! error reporting. `cgap-git` is a zero-external-dependency,
//! `no_std`-capable crate: no JSON parser exists in-tree, and smuggling
//! one in would violate the crate's law. This module therefore
//! implements a **fixed-schema text envelope** that is MCP-*shaped*
//! (typed Resources and Tools, request/response, latency report) but is
//! **NOT JSON-RPC** and must not be described as such anywhere.
//!
//! Named future work (not half-built here): a `std`-feature-gated
//! serde-free JSON-RPC front-end that maps onto the same typed surface.
//!
//! ## Wire grammar (fixed schema, newline-delimited frames)
//!
//! ```text
//! READ <path>\n           -> resource read (Resource::DerivedState)
//! TOOL <name> <path>\n    -> tool invocation (Tool::Push proposal)
//! ```
//!
//! * `<path>` and `<name>` are non-empty, contain no NUL/control bytes
//!   other than the terminating `\n`, and are at most
//!   [`MAX_FIELD`] bytes each; the whole frame is at most
//!   [`MAX_FRAME`] bytes (fixed buffers, zero heap in the parse path).
//! * Any other first token, a missing field, a NUL, an over-long
//!   field, or an over-long frame is a typed
//!   [`ProtocolError`], never a panic. All functions are total.
//!
//! Zero-daemon law: there is no socket, no process, no server. Every
//! operation is a pure in-process function call; latency is measured
//! with `Instant` and *reported* as a typed value, not gated as an SLA.

#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;
#[cfg(feature = "std")]
use alloc::vec::Vec;
use core::fmt;

/// Canonical frame cap: 256 bytes, fixed buffer, no growth.
pub const MAX_FRAME: usize = 256;

/// Canonical per-field cap (path or tool name).
pub const MAX_FIELD: usize = 128;

/// Typed Resources surface (MCP-shaped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource<'a> {
    /// Derived-state read: the bytes stored at `path` (a real file read
    /// performed by the driver, not a fetch).
    DerivedState(&'a str),
}

/// Typed Tools surface (MCP-shaped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tool<'a> {
    /// Snapshot push: `Push { snapshot_path }` enqueues a push proposal.
    Push { snapshot_path: &'a str },
}

/// Typed protocol errors. Every parse failure is one of these; parsing
/// is total — malformed input can never panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// Frame exceeded [`MAX_FRAME`].
    FrameTooLong(usize),
    /// A field exceeded [`MAX_FIELD`].
    FieldTooLong(usize),
    /// Empty or missing field where one is required.
    EmptyField,
    /// NUL or other forbidden control byte inside a field.
    ForbiddenByte(u8),
    /// First token was neither `READ` nor `TOOL`.
    UnknownVerb,
    /// Verb known but the wrong number of fields followed.
    MalformedArity,
    /// Frame did not end with `\n`.
    MissingTerminator,
    /// Unknown tool name after a valid `TOOL` verb.
    UnknownTool,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::FrameTooLong(n) => {
                write!(f, "frame too long: {n} > {MAX_FRAME}")
            }
            ProtocolError::FieldTooLong(n) => {
                write!(f, "field too long: {n} > {MAX_FIELD}")
            }
            ProtocolError::EmptyField => write!(f, "required field empty"),
            ProtocolError::ForbiddenByte(b) => write!(f, "forbidden byte 0x{b:02X}"),
            ProtocolError::UnknownVerb => write!(f, "unknown verb"),
            ProtocolError::MalformedArity => write!(f, "wrong field count for verb"),
            ProtocolError::MissingTerminator => write!(f, "frame missing LF terminator"),
            ProtocolError::UnknownTool => write!(f, "unknown tool"),
        }
    }
}

/// A parsed request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request<'a> {
    Read(Resource<'a>),
    Tool(Tool<'a>),
}

/// A queued push proposal produced by `Tool::Push`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushProposal {
    pub snapshot_path: String,
}

/// Typed latency report (measured with `Instant` where `std` is
/// available; reported, not gated as an SLA).
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyReport {
    /// Round-trip nanoseconds for the whole dispatch call.
    pub dispatch_ns: u64,
    /// Kind marker for the report consumer.
    pub op: &'static str,
}

/// Outcome of one dispatch over a [`Driver`].
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchOutcome {
    /// `Some(bytes)` for a successful READ (the resource bytes).
    pub read: Option<Vec<u8>>,
    /// `Some(())` for a successful TOOL push enqueue.
    pub pushed: Option<()>,
    pub latency: LatencyReport,
}

/// In-process driver. No daemon: a plain struct holding the push queue
/// and the resource-root used for path reads.
#[cfg(feature = "std")]
pub struct Driver {
    root: String,
    queue: Vec<PushProposal>,
}

#[cfg(feature = "std")]
impl Driver {
    /// Driver rooted at `root`; READ/TOOL paths resolve under it.
    pub fn new(root: &str) -> Self {
        Driver {
            root: String::from(root),
            queue: Vec::new(),
        }
    }

    /// Queue depth of pending push proposals.
    pub fn pending_pushes(&self) -> usize {
        self.queue.len()
    }

    /// Inspect the queued proposals.
    pub fn proposals(&self) -> &[PushProposal] {
        &self.queue
    }

    /// Parse a frame, execute it against real state, and report latency.
    /// Total: every input, including garbage, yields `Ok` or a typed
    /// [`ProtocolError`] — never a panic.
    pub fn dispatch(&mut self, frame: &[u8]) -> Result<DispatchOutcome, ProtocolError> {
        let t0 = std::time::Instant::now();
        let req = parse_frame(frame)?;
        let mut out = DispatchOutcome {
            read: None,
            pushed: None,
            latency: LatencyReport {
                dispatch_ns: 0,
                op: "",
            },
        };
        match req {
            Request::Read(Resource::DerivedState(path)) => {
                let full = join(&self.root, path);
                let bytes = std::fs::read(&full).map_err(|_| ProtocolError::EmptyField)?;
                out.read = Some(bytes);
                out.latency.op = "READ";
            }
            Request::Tool(Tool::Push { snapshot_path }) => {
                self.queue.push(PushProposal {
                    snapshot_path: String::from(snapshot_path),
                });
                out.pushed = Some(());
                out.latency.op = "TOOL:Push";
            }
        }
        out.latency.dispatch_ns = t0.elapsed().as_nanos() as u64;
        Ok(out)
    }
}

/// Join root and path with a single separator.
#[cfg(feature = "std")]
fn join(root: &str, path: &str) -> String {
    let mut s = String::from(root);
    if !root.ends_with('/') && !path.starts_with('/') {
        s.push('/');
    }
    s.push_str(path);
    s
}

/// Total frame parser: `Ok(Request)` or typed [`ProtocolError`].
///
/// Grammar (see module docs): `READ <path>\n` / `TOOL <name> <path>\n`
/// with [`MAX_FRAME`]/[`MAX_FIELD`] caps and no forbidden bytes.
pub fn parse_frame(frame: &[u8]) -> Result<Request<'_>, ProtocolError> {
    if frame.len() > MAX_FRAME {
        return Err(ProtocolError::FrameTooLong(frame.len()));
    }
    if frame.last() != Some(&b'\n') {
        return Err(ProtocolError::MissingTerminator);
    }
    if frame.contains(&0u8) {
        return Err(ProtocolError::ForbiddenByte(0));
    }
    let line = &frame[..frame.len() - 1];
    let sp1 = line.iter().position(|&b| b == b' ');
    let (verb, rest) = match sp1 {
        Some(i) => (&line[..i], &line[i + 1..]),
        None => (line, &b""[..]),
    };
    match verb {
        b"READ" => {
            let path = one_field(rest)?;
            Ok(Request::Read(Resource::DerivedState(path)))
        }
        b"TOOL" => {
            let (name, tail) = split_field(rest)?;
            if name != "Push" {
                return Err(ProtocolError::UnknownTool);
            }
            let path = one_field(tail)?;
            Ok(Request::Tool(Tool::Push {
                snapshot_path: path,
            }))
        }
        _ => Err(ProtocolError::UnknownVerb),
    }
}

/// Require exactly one non-empty, byte-clean, in-cap field.
fn one_field(field: &[u8]) -> Result<&str, ProtocolError> {
    check_field(field)?;
    // `check_field` rejects NUL/CR/LF; UTF-8 validity is enforced here
    // and surfaces as a typed error, never a panic.
    core::str::from_utf8(field).map_err(|_| ProtocolError::ForbiddenByte(b'?'))
}

fn split_field(fields: &[u8]) -> Result<(&str, &[u8]), ProtocolError> {
    let i = fields
        .iter()
        .position(|&b| b == b' ')
        .ok_or(ProtocolError::MalformedArity)?;
    let (a, b) = (&fields[..i], &fields[i + 1..]);
    check_field(a)?;
    let a = core::str::from_utf8(a).map_err(|_| ProtocolError::ForbiddenByte(b'?'))?;
    Ok((a, b))
}

fn check_field(field: &[u8]) -> Result<(), ProtocolError> {
    if field.is_empty() {
        return Err(ProtocolError::EmptyField);
    }
    if field.len() > MAX_FIELD {
        return Err(ProtocolError::FieldTooLong(field.len()));
    }
    for &b in field {
        if b == 0 || b == b'\r' || b == b'\n' {
            return Err(ProtocolError::ForbiddenByte(b));
        }
    }
    Ok(())
}
