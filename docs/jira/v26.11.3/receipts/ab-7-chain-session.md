# AB-7 Receipt — chain.rs + session.rs (AoQ Wave-1 W1-4)

Date: 2026-11-03. Lane: AB-7 (implementation). Repo: /Users/sac/cgap.
Never ran git (lane law).

## Subject (owned files)

- crates/cgap-aoq/src/chain.rs — header-agnostic append-only hash chain
  (coordinator R-H1 ruling): record hashes EXACTLY
  `prev_hash ‖ epoch ‖ index ‖ payload_digest`; no header fields, no flags,
  no variable length; `reserved` 14B zero, unhashed.
- crates/cgap-aoq/src/session.rs — 4-variant machine (HandshakeInit,
  HandshakeWait, Established, Closed), full 20-cell (state,frame) table,
  MLKEM consts 1184/1088/32 const-pinned, parse_frame length-checked
  composing AB-6's codec::decode (offsets never re-declared), two-round
  KDF (`round2 = H(ctx ‖ H(ctx ‖ ss) ‖ ek)`), 3 context consts,
  transcript binds ek_server ‖ ss ‖ epoch, REKEY_BUDGET counted-not-timed,
  no wall clock, KemSeam caller-injected (crate ships no crypto, R-K1).
- tests/chain_courts.rs (7 courts), tests/session_courts.rs (8 courts),
  tests/chain_bench.rs (1k/10k/100k min-of-5).
- Minimal disclosed unblocks (SLA) on lanes' in-flight files:
  - tests/aoq1_stress_court.rs — AB-6's mid-edit `wire_placeholder()`
    replaced with a real codec-sealed buffer (owning lane supersedes).
  - tests/purity_gate.rs — collision-ledger flip after chain.rs
    registration (see below), + AB-8's verify.rs/writer.rs added to the
    REGISTERED scan list as they landed.
- lib.rs registration reconciliations (AB-1 owns; each disclosed inline in
  lib.rs and here): created lib.rs at lane start (absent), registered
  chain.rs after the R-H1 rewrite landed, registered session.rs (AB-1
  restored it first, kept their line).

## Collision disclosure (required)

AB-7 initially drafted chain.rs with header-projecting records (parent
prefix, key_id, seq, payload_len, inline payload) — the same design AB-6
independently disclosed as a collision with the landed ontology. The
coordinator's R-H1 ruling superseded it: chain is now HEADER-AGNOSTIC
(opaque digests only). The first draft never landed green; the on-disk
chain.rs is the R-H1 rewrite, registered after the flip. AB-1's purity
gate court `chain_rs_is_ab7_collision_ledger_and_still_unregistered` was
flipped (renamed
`chain_rs_collision_ledger_is_closed_and_registered`) to name the
transition — the ledger's own designed flip path, disclosed in
purity_gate.rs and here.

## Courts (15 total, all fail-closed, real collaborators, no mocks)

chain_courts.rs: c1 GENESIS all-zeros + empty-verify + token round trip;
c2 append/verify_all/verify_span/token-resume + forged-links refusal;
c3 tamper battery (payload_digest/digest/prev_hash/epoch flips all flip
verify; parent break classifies typed DigestMismatch expected!=got);
c4 checkpoint/resume law (stale/forged head refuses, extend-past-token
lawful); c5 append-only + no-mutation-on-refusal byte-identity over
records; c6 Spoliation opacity (zero-sized, uninformative Display,
identity-equality, obtained only from a real refusal); c7
compose-with-emit_verified law (session frames append via
append_verified; tampered emission refuses typed DigestMismatch and
leaves the chain unmutated — tag covers header ‖ kind ‖ seq ‖ payload so
the chain re-derivation composes exactly).

session_courts.rs: S1 exhaustive 20-cell (state,frame) table, each cell
named lawful/UnexpectedFrame/SessionClosed + fingerprint byte-identity on
every refusal; S2 no-mutation-on-refusal battery (5 refusal classes x 4
states); S3 counted-not-timed budget (exact boundary refusal
RekeyRequired at 4096, rekey resets, no wall clock); S4 parse_frame
length law (all truncations refuse, typed expected/got, bad magic composes
codec refusal); S5 transcript binding (ek/ss/epoch sensitivity +
determinism); S6 two-round KDF (not the single-round form, context-bound,
deterministic); S7 epoch/ordinal wire law (RETIRED refused, typed
EpochMismatch/OrdinalMismatch, replay refuses); S8 10k-frame Established
soak with u32 ordinal-wrap walk (wrap witnessed mid-soak, 2 rekeys on
counted schedule, session Established throughout).

## Gates (real command + exit, this session)

`cargo test -p cgap-aoq` — GREEN TWICE:
- Run 1: exit 0 at 22:51:44 (poll-converged during concurrent AB-5/AB-8
  lane freezes; see convergence note below).
- Run 2: exit 0, 16 targets, 71 tests, 0 failures (receipt file
  /tmp/ab7_gate_D.txt). My targets in that run: chain_courts 7/7,
  session_courts 8/8, chain_bench 1/1, purity_gate green.

Convergence disclosure: the crate-wide gate was transiently red 6 times
this session due to OTHER lanes' in-flight files (cgap-crypto/kem.rs,
cgap-aoq verify.rs/writer.rs, ab8_sweep, corridor_join_court,
cbmqs_runner). Every such red was diagnosed to the owning lane's file;
none touched chain/session surfaces. Two of those in-flight files received
SLA minimal unblocks (listed above); the rest were polled to owner
convergence (bounded 30-60s polls, no sleep-based tests involved).

## Bench (measured this session, method: Instant around leg / links,
min-of-5)

| links | append µs/op | verify µs/op |
|------:|-------------:|-------------:|
| 1k    | 1.400        | 0.810        |
| 10k   | 1.418        | 0.911        |
| 100k  | 3.273        | 1.827        |

(Deterministic test digest seam; the bench test asserts a nonzero floor —
no perf claim beyond the printed min-of-N method.)

## Standing

- chain.rs, session.rs, 15 courts, bench: ALIVE (commands above, exit 0,
  this session, exact subject on disk).
- Full-crate gate stability depends on concurrent lanes (AB-8/AB-5) —
  receipt is as-of 22:51-22:53 local this session.
