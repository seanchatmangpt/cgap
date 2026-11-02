# Lane GIT-02 receipt: pkt-line codec

- **Lane**: GIT-02, repo /Users/sac/cgap, branch main
- **Files written** (only these):
  - `crates/cgap-git/src/pkt_line.rs` (new, untracked)
  - `crates/cgap-git/tests/pkt_line_court.rs` (new, untracked)
  - `docs/jira/v26.11.2/lanes/lane-git-02-pkt-line.md` (this file)
- **lib.rs untouched** (per dispatch): module is NOT wired into
  `crates/cgap-git/src/lib.rs`; tests self-contain via
  `#[path = "../src/pkt_line.rs"] mod pkt_line;`.
- **Reuse vs new**: new self-contained `PktError` enum inside pkt_line.rs
  (not reusing `GitError`) so the module is crate-independent and
  `#[path]`-includable without lib.rs wiring; error style (typed enum +
  verbose Display) matches house style of `GitError`.

## Scope delivered

- 4-hex-digit length-prefixed pkt-line encoder/decoder, constant-time
  hex validation (all 4 nibbles decoded + validity-masked before any
  branch).
- Control markers: flush `0000`, delim `0001`, response-end `0002`
  (per git `gitprotocol-v2.txt`: response-end marks the end of a
  stateless-connection response; semantics stated from the real
  protocol docs).
- `ERR <msg>` packets: `encode_err` + `Pkt::err_message()` typed
  extraction.
- Fixed 64-byte stack buffers (`PKT_BUF_LEN = 64`); zero heap in
  encode/decode paths (no Vec/String in the module; payload borrows
  the input slice). no_std-compatible (`#![deny(unsafe_code)]`, only
  `core` items).
- Protocol cap: `MAX_PKT_PAYLOAD = 65516` (git `LARGE_PACKET_MAX`
  65520 total minus the 4-byte header).
- Additive-only public API; nothing existing was edited.

## Courts (all with anti-vacuity flip twins, 10 tests)

1. `court_bad_hex_length` — `000g` => `PktError::BadHexLength`; flip
   twin mutated parser accepts the same bytes. FLIPPED.
2. `court_bad_hex_length_digits` — non-hex digit at each of the 4
   positions refused identically.
3. `court_truncated_payload` — declared length > available => typed
   `Truncated { declared, available }`. Also 3-byte header truncation.
4. `court_oversize_length` — `ffff` (65535) and `fff5` (65525) refused
   as `BadLength`; cap itself `fff0` (65520 total) is legal Data.
5. `court_control_markers` — 0000/0001/0002 decode as kinds;
   `0003` (below the 4-byte minimum) refused as `BadLength(3)`.
6. `court_roundtrip_byte_exact` — `"refs/heads/main\0report-status"`
   (29 bytes) encodes to `0021...` (33 total), byte-exact vs
   hand-computed; round-trip decode recovers the payload; flip twin
   constant-header encoder produces different bytes. FLIPPED.
7. `court_encode_control` — empty payload => `0004`; flush/delim/
   response-end encoders emit exact markers.
8. `court_err_packets` — `ERR access denied` => `0015ERR access
   denied` byte-exact; `err_message()` typed extraction; non-ERR
   payload yields None.
9. `court_payload_too_large` — 61-byte payload => typed
   `PayloadTooLarge { 61, 60 }`; 60 bytes at cap succeeds => `0040`.
10. `court_stream_framing` — flush + `000awant x` + delim stream
    decodes with exact per-pkt consumption; full consumption asserted.

## Commands (this session)

- `CARGO_TARGET_DIR=/tmp/git02-target cargo test -p cgap-git --test
  pkt_line_court` — exit 0, **10 passed; 0 failed**.

## Commands (this session)

- `CARGO_TARGET_DIR=/tmp/git02-target cargo test -p cgap-git --test
  pkt_line_court` — exit 0, 10 passed; 0 failed.
- `CARGO_TARGET_DIR=/tmp/git02-target cargo test -p cgap-git --lib` —
  exit 0, 61 passed; 0 failed.
- Per-target sweep of the 12 compilable test targets (3 other lanes'
  in-flight targets abort compilation — see divergences): all 12
  green, 0 failures total across 45 tests in integration targets +
  61 lib tests. `perf_baseline` / `ladder_timing_stress` are
  1-ignored-by-design (release-only benches).
- `grep -nE "Vec|String|alloc::|std::" crates/cgap-git/src/pkt_line.rs`
  — zero matches (exit 1): encode/decode paths allocate nothing.
- Build-root cleanup: `python3 -c "import shutil;
  shutil.rmtree('/tmp/git02-target', True)"` at lane end.

## Standing

ALIVE for the lane surface (pkt_line_court 10/10 + lib 61/61 run and
passed this session, exact commands above). The three non-compiling
targets are OTHER lanes' untracked in-flight files
(object_store_court / mcp_driver_court / ingress_ring_court +
their src modules) — compile-abort, not failures, not mine.

## Fences run (must-not-regress surfaces)

Full cgap-git per-target sweep, exit 0 on all 12 compilable targets
(see Commands). No shared file edited, so no lib.rs/Cargo.toml fence
is touched.

## Divergences

1. Dispatch named `docs/jira/v26.11.2/waves/<lane>-receipt.md` as the
   receipt location; actual dispatch text named
   `docs/jira/v26.11.2/lanes/lane-git-02-pkt-line.md` — used the
   named path.
2. pkt_line is NOT wired into lib.rs (lib.rs off-limits per
   dispatch), so the module is not yet reachable as `cgap_git::
   pkt_line` from outside the crate; the court reaches it via
   `#[path]`. Wiring is a one-line additive lib.rs change left to the
   coordinator.
3. Two corrupted long-file writes were caught and rewritten via
   python assembly (test file, receipt file) — on-disk bytes
   verified by the passing compile.
