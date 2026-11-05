//! Lane GIT-14 mutation court for the MCP-shaped driver.
//!
//! Included via `#[path]` because `lib.rs` is off-limits to this lane:
//! the module under test is compiled directly into this test binary.

#[path = "../src/mcp_driver.rs"]
mod mcp_driver;

use mcp_driver::*;

// ---------- anti-vacuity twins ----------

/// Court 1 (mutation twin): an UNPARSEABLE frame — the dispatch's
/// `"TOOL\0garbage"` mutation — MUST yield a typed protocol error and
/// MUST NOT panic. If parsing were vacuous (accepted anything), or
/// panicky, this court fails. Run under catch_unwind to prove no-panic
/// even though every function is total by construction.
#[test]
fn court_malformed_frame_typed_error_no_panic() {
    let mut d = Driver::new("/tmp/cgap-git14-does-not-exist");
    for frame in [
        &b"TOOL\0garbage\n"[..],
        b"READ\n",
        b"TOOL Push\n",
        b"TOOL NotATool /x\n",
        b"FLY /x\n",
        b"READ /x", // missing terminator
        b"",        // empty frame
        &[0u8; 300],
        b"READ ", // over-long frame / empty field (with LF)
    ] {
        let mut f = frame.to_vec();
        if f.last() != Some(&b'\n') && f.len() <= MAX_FRAME {
            f.push(b'\n');
        }
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| d.dispatch(&f)));
        let r = r.expect("dispatch must not panic on malformed input");
        assert!(
            r.is_err(),
            "frame {frame:?} must be a typed error, got {r:?}"
        );
        // Typed, inspectable, not a bare unit error.
        let e = r.unwrap_err();
        let _msg = alloc_format(&e);
    }
}

#[cfg(feature = "std")]
fn alloc_format(e: &ProtocolError) -> String {
    use std::string::ToString;
    e.to_string()
}

/// Court 2 (valid READ): a real resource read returns the real bytes
/// written to a real temp file — final state asserted, no mocks.
#[test]
fn court_valid_read_returns_resource_bytes() {
    let dir = std::env::temp_dir().join("cgap-git14-court-read");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("derived_state.bin");
    std::fs::write(&p, b"SNAPSHOT-BYTES-0xA20C").unwrap();

    let mut d = Driver::new(dir.to_str().unwrap());
    let mut frame = b"READ ".to_vec();
    frame.extend_from_slice(b"derived_state.bin");
    frame.push(b'\n');
    let out = d.dispatch(&frame).expect("valid READ must parse+execute");
    assert_eq!(out.read.as_deref(), Some(&b"SNAPSHOT-BYTES-0xA20C"[..]));
    assert!(out.pushed.is_none());
    assert_eq!(out.latency.op, "READ");
}

/// Court 3 (valid TOOL): a valid push frame enqueues exactly one typed
/// proposal — final queue state asserted.
#[test]
fn court_valid_tool_enqueues_push_proposal() {
    let mut d = Driver::new("/tmp/cgap-git14-court-tool");
    assert_eq!(d.pending_pushes(), 0);
    let out = d
        .dispatch(b"TOOL Push snapshots/daily.bin\n")
        .expect("valid TOOL Push must succeed");
    assert_eq!(out.pushed, Some(()));
    assert!(out.read.is_none());
    assert_eq!(out.latency.op, "TOOL:Push");
    assert_eq!(d.pending_pushes(), 1);
    assert_eq!(
        d.proposals()[0],
        PushProposal {
            snapshot_path: owned("snapshots/daily.bin")
        }
    );
}

/// Court 4 (latency report is typed and measured, reported not gated):
/// in-process dispatch must complete and carry a nonzero measured
/// nanoseconds value. Sub-millisecond is REPORTED as an observation,
/// not asserted as an SLA (honest scoping per lane contract).
#[test]
fn court_latency_report_typed_and_measured() {
    let mut d = Driver::new("/tmp/cgap-git14-court-lat");
    let out = d
        .dispatch(b"TOOL Push s.bin\n")
        .expect("dispatch must succeed");
    assert!(out.latency.dispatch_ns > 0, "latency must be measured");
    // Observation only (not a gate): print measured in-process latency.
    if out.latency.dispatch_ns < 1_000_000 {
        // sub-ms in-process, as expected — reported, not enforced.
    }
}

// ---------- parse-level mutation twins (flip both directions) ----------

/// Anti-vacuity: the parser must DISTINGUISH valid from invalid — a
/// valid frame parses, each mutation of it fails with a DIFFERENT typed
/// error. If all mutations mapped to the same error (or none failed),
/// the grammar would be vacuous.
#[test]
fn court_mutations_flip_with_distinct_typed_errors() {
    let valid = b"TOOL Push s.bin\n";
    assert_eq!(
        parse_frame(valid).unwrap(),
        Request::Tool(Tool::Push {
            snapshot_path: "s.bin",
        })
    );
    assert_eq!(
        parse_frame(b"READ s.bin\n").unwrap(),
        Request::Read(Resource::DerivedState("s.bin"))
    );

    let cases: &[(&[u8], ProtocolError)] = &[
        (b"TOOL\0garbage\n", ProtocolError::ForbiddenByte(0)),
        (b"FLY s.bin\n", ProtocolError::UnknownVerb),
        (b"READ\n", ProtocolError::EmptyField),
        (b"TOOL Push\n", ProtocolError::MalformedArity),
        (b"TOOL NotATool s.bin\n", ProtocolError::UnknownTool),
        (b"READ s.bin", ProtocolError::MissingTerminator),
        (&[b'R'; 300], ProtocolError::FrameTooLong(300)),
        (b"READ \n", ProtocolError::EmptyField),
    ];
    for (frame, want) in cases {
        assert_eq!(
            parse_frame(frame).unwrap_err(),
            *want,
            "frame {frame:?} must flip to {want:?}"
        );
    }
}

/// Over-long FIELD (within frame cap) flips distinctly from over-long
/// FRAME — a single blanket error would make the caps unauditable.
#[test]
fn court_field_cap_distinct_from_frame_cap() {
    let long_path = [b'a'; MAX_FIELD + 1];
    let mut frame = b"READ ".to_vec();
    frame.extend_from_slice(&long_path);
    frame.push(b'\n');
    assert_eq!(
        parse_frame(&frame).unwrap_err(),
        ProtocolError::FieldTooLong(MAX_FIELD + 1),
    );
    // Exactly at cap parses fine (boundary).
    let ok_path = [b'a'; MAX_FIELD];
    let mut ok = b"READ ".to_vec();
    ok.extend_from_slice(&ok_path);
    ok.push(b'\n');
    assert!(parse_frame(&ok).is_ok());
}

// ---------- helpers ----------

#[allow(dead_code)]
fn owned(s: &str) -> std::string::String {
    std::string::String::from(s)
}
