# Lane FIX-05 — pack_demux poison refusal + surplus guard (receipt)

Lane: FIX-05 · Date: 2026-11-02 · Repo: /Users/sac/cgap (branch main)
Subject: crates/cgap-git/src/pack_demux.rs (owned), plus new court
crates/cgap-git/tests/pack_demux_poison_court.rs (owned), plus ONE
disclosed minimal unblock (see Divergences #3).
Standing: **ALIVE** (all courts ran green this session; mutations flipped)
Source finding: docs/jira/v26.11.2/redteam/red-09-alloc-sentry-breach.md W6 F1/F2

## W1 Diff (μ)
- F1 (red-09 F1): `Phase::Done => {}` (pack_demux.rs:343 pre-fix)
  silently absorbed post-trailer poison while `bytes_fed` still counted
  it. NOW: each post-trailer byte increments a `trailing_bytes` ledger
  (new private field + public `trailing_poison()` accessor); `feed`
  returns Err(DemuxError::TrailingBytes(cumulative_count)) — sticky
  refusal across subsequent feeds. Typed variant + Display arm added.
- F2 (red-09 F2): `close_record` had no surplus tripwire. NOW: guard
  BEFORE the objects_seen increment refuses
  DemuxError::ObjectSurplus{seen, declared} when
  objects_seen >= declared_objects. Documented honestly: the guard is
  an INVARIANT GUARD — the streaming phase law enters ObjHeader only
  while objects_seen < declared, so the arm is unreachable through the
  public API; its runtime flip is proven by an in-module
  `#[cfg(test)] mod fix05_courts` (Q1 twin + Q2 flip court) that
  forces the surplus state through the private fields (real type, no
  mocks). The observable surplus channel through the public API (a
  surplus object's bytes overflowing trailer capture) is refused via
  the F1 TrailingBytes arm (court P3).
- Court file: tests/pack_demux_poison_court.rs — P1 clean-pack
  baseline (zero poison), P2 F1 court (4 poison bytes ->
  TrailingBytes(4), trailer uncorrupted, finish Ok, objects pinned),
  P3 F2 exploit court (declare 1, supply 2 well-formed objects; the
  26 post-trailer bytes = 6 surplus framing bytes + 20 stale-checksum
  bytes are refused typed; objects_seen pinned at declared).
- mtime-bump restore law honored: every sandbox restore was `touch`ed
  after sha256-exact copy-back.

## W3 Commands + exits (this session)
- `python3 scripts/run_checked.py -- cargo test -p cgap-git --test
  pack_demux_poison_court --test pack_demux_court --test
  posh_alloc_sentry_court` -> exit 0 (11+1ign / 3 / 1 passed).
- `cargo test -p cgap-git --test posh_refusal_matrix_court --test
  posh_wire_court --test soc_full_chip_court --test
  e2e_wire_substrate_court (+ owned three)` -> exit 0
  (4 / 2 / 3 / 5 / 3 / 11+1ign / 1 passed).
- In-module Q courts: `cargo test --lib pack_demux` (sandbox) -> exit 0
  (q1 + q2 passed).
- Mutation m1 (revert Done arm to `{}`): poison court -> exit 101; P2
  and P3 both fail VACUOUS ("post-trailer poison silently absorbed").
  Restore sha256-exact: e21b07567f8b... / touched.
- Mutation m2 (full pre-fix revert, git show HEAD): compile-abort (new
  API referenced by court) — compile-abort is NOT a failure count; the
  runtime flip for the pre-fix state is carried by m1 (P2/P3) + m3.
- Mutation m3 (delete close_record surplus guard): `cargo test --lib
  pack_demux` -> exit 101; q2 fails VACUOUS ("surplus close admitted"),
  q1 twin stays green. Restore sha256-exact: 8989d3978cbc... / touched.
- Full `cargo test -p cgap-git` (3 runs) -> exit 101 each time from
  OTHER lanes' in-flight test files (ref_store_reentrancy_court,
  ref_store_width_arms_court, quota_alloc_purity_court — raw syntax
  errors, mid-write). Compile-abort is not a failure count; census law.
  All OWNED + fence binaries green in isolation (above). Final state at
  receipt time: full-suite compile BLOCKED-BY-OTHER-LANES, re-poll
  pending at receipt tail.

## W5 Anti-vacuity
- P2/P3 name m1 (Done-arm revert); Q2 names m3 (guard deletion); all
  three FLIPPED this session in the /tmp sandbox
  (/tmp/fix05-sbox/cgap-git, lease /tmp/fix05-sbox + target dir left
  for coordinator cleanup; sandbox ref_store/egress_ring/ingress_ring/
  mcp_driver were HEAD-snapshotted to dodge concurrent-lane churn).
- m2 full-revert: compile-abort (recorded, not counted).
- Shared tree never mutated outside owned files (sha256 verified after
  each sandbox restore).

## W6 Divergences (disclosed)
1. F2 guard reachability: the dispatch asked for a close_record
   surplus check; the phase law makes it unreachable via the public
   API. Landed as a typed invariant guard + in-module flip court (Q2)
   forcing the state privately; the public-API surplus exploit
   (declare 1, supply 2) is refused via the F1 TrailingBytes arm
   (court P3, flipped under m1). Disclosed in the module doc comment
   on the ObjectSurplus variant.
2. MINIMAL UNBLOCK (disclose-in-both-directions law): edited
   tests/posh_refusal_matrix_court.rs (NOT in my named ownership) —
   arm 1's `clean_pack()` was `write_pack_v2` output, but the demux
   law is STORED framing while the writer emits zlib bodies, so writer
   output never ends at the demux trailer; pre-fix, the 8 overflow
   bytes were silently absorbed by the `Phase::Done => {}` arm and the
   court passed on misframed bytes. The typed refusal exposed this
   latent defect ("clean pack feeds: TrailingBytes(8)"). Unblock:
   clean_pack now hand-builds a stored-framing pack
   (encode_object_header + sha1, mirroring pack_demux_court's
   builder); arm's flip law unchanged. Owning lane should
   confirm/supersede on landing. (Errata to red-09 W5: "BREACH none"
   verdict stands, but the D3/D6 clean-parse claims for writer output
   should be re-read against this framing mismatch.)
3. Full `cargo test -p cgap-git` compile is currently broken by OTHER
   lanes' in-flight files (ref_store_*, quota_alloc_purity_court
   syntax error) — within their compile-freeze SLA windows; my owned
   + fence binaries pass in isolation. Contract item "cargo test -p
   cgap-git must stay green": cannot honestly claim for the FULL
   suite this session (BLOCKED-BY-OTHER-LANES); every binary my diff
   can affect is green.
4. Fences run: pack_demux_court, posh_wire_court,
   posh_refusal_matrix_court, soc_full_chip_court,
   e2e_wire_substrate_court, posh_alloc_sentry_court (the zero-alloc
   sentry stays green — the demux path gained one u64 field, zero
   allocator calls).

## JSON terminus
```json
{"lane":"FIX-05",
 "files_written":[
  "/Users/sac/cgap/crates/cgap-git/src/pack_demux.rs",
  "/Users/sac/cgap/crates/cgap-git/tests/pack_demux_poison_court.rs",
  "/Users/sac/cgap/crates/cgap-git/tests/posh_refusal_matrix_court.rs (disclosed minimal unblock)",
  "/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-fix05-pack-demux-poison.md"],
 "commands":[
  "cargo test -p cgap-git --test pack_demux_poison_court --test pack_demux_court --test posh_alloc_sentry_court -> exit 0",
  "cargo test -p cgap-git --test posh_refusal_matrix_court --test posh_wire_court --test soc_full_chip_court --test e2e_wire_substrate_court + owned three -> exit 0",
  "cargo test (sandbox) --lib pack_demux -> exit 0 (q1+q2)",
  "mutation m1 (Done-arm revert) -> exit 101 FLIPPED, sha256-exact restore",
  "mutation m3 (surplus-guard deletion) -> exit 101 FLIPPED (q2), q1 green, sha256-exact restore",
  "mutation m2 (pre-fix full revert) -> compile-abort 101 (not a failure count)",
  "cargo test -p cgap-git (full) -> exit 101, BLOCKED-BY-OTHER-LANES in-flight test files"],
 "courts_run":[
  "p1_clean_pack_zero_poison (baseline)",
  "p2_post_trailer_poison_is_typed_refusal (F1, m1-flipped)",
  "p3_surplus_object_supply_is_refused (F2 exploit, m1-flipped)",
  "q1_normal_close_succeeds_below_declared (twin)",
  "q2_surplus_close_refuses_typed (F2 guard, m3-flipped)",
  "pack_demux_court 11+1ign, posh_alloc_sentry_court 1, posh_refusal_matrix_court 4, posh_wire_court 2, soc_full_chip_court 3, e2e_wire_substrate_court 5"],
 "standing":"ALIVE",
 "divergences":[
  "F2 guard unreachable via public API (phase law) — typed invariant guard + private-state flip court Q2; public surplus exploit refused via TrailingBytes (P3)",
  "posh_refusal_matrix_court arm1 clean_pack was zlib-framed writer output absorbed silently by the old Done arm — latent defect exposed; stored-framing unblock disclosed in both directions",
  "full cargo test -p cgap-git BLOCKED by other lanes' in-flight court files this session; owned+fence binaries green in isolation"]}
```
