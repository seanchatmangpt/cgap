# Lane GIT-14: MCP-shaped request driver (cgap-git)

Date: 2026-11-02 · Branch: main · Contract: additive-only, no lib.rs edit, files:
`crates/cgap-git/src/mcp_driver.rs`,
`crates/cgap-git/tests/mcp_driver_court.rs`, this receipt.

## Honest scoping (the lane is judged on this)

MCP on the wire is JSON-RPC 2.0. JSON-RPC needs a JSON parser = heap +
std-grade error paths; cgap-git is a zero-external-dep, no_std-capable
published kernel with no JSON parser in-tree. Half-building JSON-RPC in
no_std would violate the crate's law. This lane therefore ships a
**fixed-schema text envelope** — MCP-*shaped*, NOT JSON-RPC, documented
as a divergence at `crates/cgap-git/src/mcp_driver.rs:1-35` (module
docs, wire grammar, caps). Named future work (recorded, not half-built):
a `std`-feature-gated, serde-free JSON-RPC front-end mapping onto the
same typed surface.

## What landed

- `parse_frame` — total parser, `Ok(Request)` or typed
  [`ProtocolError`] (8 variants, Display impl). Grammar: `READ <path>\n`
  / `Tool::Push` via `TOOL Push <path>\n`; frame cap 256 B
  (`MAX_FRAME`), field cap 128 B (`MAX_FIELD`), NUL/CR/LF forbidden
  bytes, LF-terminator required.
- `Driver` (std-gated): in-process, zero-daemon — plain struct + real
  `std::fs::read` resource reads rooted at `root`; typed `LatencyReport`
  via `Instant` — reported, not gated as SLA.
- Typed surfaces: `Resource::DerivedState`, `Tool::Push`,
  `PushProposal` queue, `DispatchOutcome { read, pushed, latency }`.

## Courts (6/6 green, anti-vacuity twins)

- malformed-frame court: `"TOOL\0garbage"` and 8 more malformed frames
  under `catch_unwind` — typed error, no panic (mutation = un-parseable
  frame; vacuous parser would accept it).
- distinct-flip court: 8 mutations of a valid frame each flip to a
  DIFFERENT typed error; caps court proves `FieldTooLong` ≠
  `FrameTooLong` distinctly, plus at-cap boundary parses.
- valid READ returns real temp-file bytes; valid TOOL enqueues exactly
  one `PushProposal` (final queue state asserted); latency court asserts
  measured `dispatch_ns > 0`.

## Commands + exits (this session, exact subject)

- `CARGO_TARGET_DIR=/tmp/git14-target cargo test -p cgap-git --test mcp_driver_court`
  → 6 passed; 0 failed, exit 0.
- `CARGO_TARGET_DIR=/tmp/git14-target build --no-default-features` — lib
  no_std build clean (mcp_driver is test-consumed via `#[path]`; lib.rs
  untouched, published surface unchanged this lane).
- `cargo test -p cgap-git --lib` → 61 passed; 0 failed, exit 0.
- Full `cargo test -p cgap-git` compiled aborts in OTHER lanes'
  in-flight test files (`object_store_court.rs`,
  `ingress_ring_court.rs`) — census law: compile-abort ≠ failure count;
  not this lane's surface.

## Standing

ALIVE for the module + court (commands + exit 0 above, this session,
exact subject). UNVERIFIED: no_std build of the MODULE itself (it is
std-gated where it touches fs/Instant; the parse half is no_std-pure and
compiles clean in the lib build which forbids std in hot paths — the
parse half's standalone no_std build is UNVERIFIED, never rounded up).
Perf numbers are observations (`dispatch_ns > 0` measured; sub-ms
observed but REPORTED, not gated as SLA).
