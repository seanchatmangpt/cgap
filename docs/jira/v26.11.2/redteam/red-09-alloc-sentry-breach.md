# RED-09 — Zero-Alloc Sentry Poisoning (red-team receipt)

Lane: RED-09 · Date: 2026-11-02 · Repo: /Users/sac/cgap (read-only lane)
Standing: **ALIVE** (battery ran this session, exit 0; no breach confirmed)

## W1 Subject
The zero-alloc sentry surface: `CountingAlloc` (window-gated alloc/byte
ledger over `System`) at
`crates/cgap-git/tests/posh_alloc_sentry_court.rs:49-84`, mirrored by
`soc_full_chip_court.rs` and consumed by
`crates/cgap-parquet/tests/page_zero_alloc.rs` and
`crates/cgap-blake3/tests/zero_alloc.rs`. FFI note: `cgap-ffi` host
builds keep std by design (`crates/cgap-ffi/src/lib.rs`), so there is NO
CountingAlloc across the FFI boundary — that leg is UNSUPPORTED-by-design,
not attacked (see W6).

## W2 Mission + Method
Adversarial (malformed / out-of-order / lie-in-header) inputs driven
through the REAL cgap-git parse paths under an open sentry window, via a
/tmp harness crate (`/tmp/red09`, path-dep on cgap-git, build root
`/tmp/red09-target`, zero writes into the shared tree).

## W3 Commands + exits

- `python3 scripts/run_checked.py --out /tmp/red09_baseline.log -- cargo
  test -p cgap-git --test posh_alloc_sentry_court -- --nocapture` →
  **exit 0**; 1 passed; positive control allocs=1 bytes=512; sentry
  window allocs=0 bytes=0 over 46,442 relayed bytes; honesty window
  MEASURED bn3 fixpoint allocs=2,662 / 650,294,994 bytes (heavy heap
  path, explicitly scoped OUT of the zero claim — court lines 299-334).
- `/tmp/red09-target/release/red09` (via run_checked, log
  `/tmp/red09_run.log`) → **exit 0**; 15 attack cases + positive control
  + poison-tail probe, all green.

## W4 Attack battery (all allocs=0 bytes=0 under open sentry window)

Ring (`crates/cgap-git/src/ingress_ring.rs`, `#![forbid(unsafe_code)]`):
- R1 zero-length frame push + typed `Empty` pop — 0/0.
- R2 40,000-B frame > 32,768-B ring — typed `FrameTooLarge` refusal, 0/0.
- R3 fill to exactly full capacity (64x512B) then one more push — typed
  `Overflow`, state untouched, recovery pop Ok, 0/0. (Red-team note:
  full-capacity use is real — 64 frames fit exactly; the refusal fires
  on the 65th.)
- R4 200 rounds interleaved push/pop, odd frame sizes — byte-count
  fidelity, 0/0.

Demux (`crates/cgap-git/src/pack_demux.rs:274` `feed`):
- D1 bad magic `PACX` — typed refusal, 0/0.
- D2 version 3 — typed `BadVersion`, 0/0.
- D3 pathological 1-byte-at-a-time segmentation of a valid pack →
  reaches `Done`, 0/0.
- D4 declared-count lie (declares 5, supplies 2 + trailer) — typed
  `Truncated` at finish, never a fake clean finish, 0/0.
- D5 overlong varint (12x `0x80`) — typed `BadVarint` via the shift
  guard (pack_demux.rs:362-365), 0/0.
- D6 reserved type-5 header — typed refusal, 0/0.
- D7 OFS_DELTA `0xFF` flood (>10 continuation bytes) — typed
  `BadDeltaOffset`, 0/0.
- D8 declared size 1 TiB, truncated body — skip counter only, typed
  `Truncated`, 0/0.

pkt-line (`crates/cgap-git/src/pkt_line.rs:126` `decode_pkt`):
- P1 flush; P2 `0xFFFF` header; P3 non-hex `zzzz`; P4 length lie
  (declared 100 vs 8 available) — typed `Truncated{100,8}`; all 0/0.

## W5 Verdict
Steady-state zero-alloc is PROVEN (under this battery's edge cases) for
the ring / demux / pkt-line ingress paths: every malformed, out-of-order,
lying-header input was refused typed or parsed clean with allocs=0
bytes=0 inside an open CountingAlloc window, and the sentry's positive
control flips (allocs=1/512B). **BREACH-CONFIRMED: none.**

Scope caveats (honest, not breaches):
1. The bn3 `unify_to_fixpoint` path allocates heavily (2,662 allocs /
   650,294,994 bytes measured this session) and is EXPLICITLY scoped out
   of the sentry's zero claim by the court itself
   (posh_alloc_sentry_court.rs:3-10, 299-334). The zero-alloc claim is a
   claim about ring+seal, not the fixpoint.
2. The FFI boundary has no CountingAlloc sentry; cgap-ffi host builds
   keep std by design (`.claude/rules/kernels-no-std.md` inventory), so
   cross-FFI zero-alloc is UNSUPPORTED (unaudited), not proven.

## W6 Findings (non-alloc, ground-backed)

F1 (protocol, low): post-trailer poison tail is silently absorbed.
`Demux::feed` hits `Phase::Done => {}` (pack_demux.rs:316, phase enum at
:141) and returns `Ok(())` while still counting the poison bytes in
`bytes_fed` (increment at pack_demux.rs:280 precedes the phase match).
Witnessed probe: feed 4 garbage bytes after a clean Done →
`feed=Ok(()), done=true, bytes_fed 34->38`. No typed refusal for a
non-empty tail after the trailer; a fix would be a `TrailingBytes`
refusal or a documented byte-exactness invariant.
F2 (info): `Demux::finish` is the only count-lie tripwire and it fires
only on truncation, not on surplus objects — a pack declaring N and
supplying >N well-formed objects + trailer would need a
`objects_seen > declared_objects` check (none exists; close_record at
pack_demux.rs:407 never compares against `declared_objects`). Not
exploited for alloc (fixed RECORD_CAP table), logged as a header-lie
tolerance gap.

## W7 Fences + restore law
- Shared tree untouched: lane was read-only over crates/, include/,
  scripts/; only file written is this receipt. Verify: `git status`
  clean except docs receipt (coordinator owns git).
- Fence named: the sentry's own anti-vacuity (positive control first,
  court lines 259-271) — my harness replicated the control and it
  flipped; a sentry that cannot flip would be VACUOUS.
- No Process.sleep; all outcomes asserted on final state.
- Build root lease `/tmp/red09-target` left standing for coordinator
  cleanup; harness source `/tmp/red09/` (evidence, not an asset).

## JSON terminus
```json
{"lane":"RED-09",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/redteam/red-09-alloc-sentry-breach.md"],
 "commands":[
  "cargo test -p cgap-git --test posh_alloc_sentry_court -- --nocapture -> exit 0",
  "/tmp/red09-target/release/red09 -> exit 0 (15 attack cases + control + probe)"],
 "courts_run":["posh_alloc_sentry_driver (baseline, exit 0)",
  "RED-09 adversarial battery (16 cases, exit 0)"],
 "standing":"ALIVE",
 "divergences":[
  "No CountingAlloc exists across the FFI boundary (cgap-ffi keeps std by design on host); cross-FFI zero-alloc leg UNSUPPORTED-by-design, attack narrowed to cgap-git sentry surfaces",
  "Two non-alloc findings logged: post-Done poison-tail silent absorption (pack_demux.rs Phase::Done => {}) and no objects_seen>declared_objects surplus check"]}
```
