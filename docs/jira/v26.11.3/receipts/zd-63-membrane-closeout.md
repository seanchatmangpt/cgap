# Receipt: ZD-63 — Wave-2 membrane closeout (plan status flip + rung witness ledger)

**Lane**: Wave-2 closeout (zd-63)
**Date**: 2026-11-04
**Status**: PARTIAL_ALIVE — this receipt's standing is documentation
correctness (per-lane standings and ladder rung witness states re-read
from each lane receipt at closeout time); it claims no new executions.
No git actions taken (not committed, per contract).
**Subject files** (exact):

- `/Users/sac/cgap/docs/jira/v26.11.3/plans/wave-2-membrane-plan.md` —
  Status line flipped PLAN (UNVERIFIED) → LANDED with rung states; new
  "Wave-2 landing state" section added at top; plan body below it
  untouched.
- This receipt.

## Per-lane standing (re-read from receipts at closeout)

All four receipt paths below were read from disk this closeout.

- **W2-A** — cgap
  `docs/jira/v26.11.3/receipts/zd-6-w2a-session-ffi.md`: ALIVE
  (surface + 6 courts + ×2 ladder rows 3-4).
- **W2-B** — arcus
  `docs/jira/v26.11.3/receipts/zd-29-w2b-twin.md`: ALIVE (twin +
  hand-law boundary + boot wiring; sync ×2 `written: []`; zero cgo).
- **W2-C** — cgap
  `docs/jira/v26.11.3/receipts/zd-13-w2c-sidecar.md`: ALIVE (sidecar
  bin + J1/J2/J3 ×2).
- **W2-D** — arcus
  `docs/jira/v26.11.3/receipts/zd-42-w2d-admission.md`: ALIVE (admit
  chain + bijection court + real-binary law ladder; flight 24/0 ×2).
- **W2-E** — rode with lanes; courts landed per-lane
  (`aoq_session_ffi_court.rs` 6/6, `aoq_sidecar_court.rs` 3/0 ×2,
  `aoq_ipc_test.go` drift + round-trip + status-band courts,
  `aoq_boundary_test.go` incl. TestAoqBoundaryRealSidecar,
  `admit_test.go` 6 courts). Green per lane receipts.
- **ZD-56 E2E** — receipt ABSENT on disk (`ls` + grep over both
  receipt dirs → no match; independently disclosed absent by the
  zd-57 index pass). NOT LANDED.

## Ladder rung witness ledger (plan §4, rungs 0-8)

- **Rung 0** — PARTIAL. Corridor leg: `cargo test -p cgap-aoq --test
  corridor_join_court` 3/0 exit 0 this session (zd-13). `cargo test -p
  cgap-crypto` leg: NOT re-run by any Wave-2 receipt; AF1-KD-1 standing
  from AB-5 (ALIVE, earlier pass), uncited this session → not
  re-claimed.
- **Rung 1** — WITNESSED ×2 (zd-6 rows 3-4, zd-13 full suite; 14
  suites, lib 60/60, session court 6/6).
- **Rung 2** — WITNESSED ×2 (zd-13, 3 passed / 0 failed twice).
- **Rung 3** — WITNESSED (zd-13; corridor unregressed with the bin
  target landed).
- **Rung 4** — WITNESSED (zd-29 `go test ./...` exit 0 first run;
  zd-42 `go test -count=1 ./...` ×2 exit 0; flight 24/0 ×2).
- **Rung 5** — WITNESSED (zd-29: build/vet exit 0;
  `go list -deps | grep -c runtime/cgo` = 0).
- **Rung 6** — PARTIAL. Arcus side witnessed (sync ×2 final `written:
  []`, drift court byte-matches `include/cgap_aoq.h` at test time,
  zd-29). Cgap-side sync determinism not cited by any Wave-2 receipt.
- **Rung 7** — UNWITNESSED. `make gate0` not run by any Wave-2
  receipt.
- **Rung 8** — PARTIAL. Real-binary law ladder + refusal chain
  witnessed (zd-42 courts 6/7: valid real tag → EPOCH_MISMATCH, i.e.
  tag gate passed and the state law fired deeper; flipped byte + stale
  tag → DIGEST_MISMATCH at the tag gate; full admission chain → typed
  in-band 0x4347A006). Happy-path echo witnessed only against an
  in-process peer speaking the sidecar wire law with the real blake3
  tag gate (zd-42 court 5, decrypted == pre-seal bytes) — the ML-KEM-768
  handshake is out of Go-stdlib reach, so the real-binary full
  handshake echo is ZD-56's leg; its receipt is ABSENT on disk →
  rung 8 NOT fully witnessed.

## Falsifier sweep (plan §3)

None fired unaddressed. Spot-check 3 (per contract):

1. **W2-A** — fingerprint-identity-after-refusal / stale-handle /
   panic falsifiers addressed by zd-6 courts 2 (ct-echo flip →
   DIGEST_MISMATCH, fingerprint byte-identical), 3 (refusal battery
   with accepted-step-moves-fingerprint anti-vacuity leg), 4
   (stale/never-existed/double-free refused identically, monotone
   ids), 5 (hostile inputs never panic). Bonus: the W2-B drift-court
   falsifier addressed by zd-29 TestAoqHeaderDriftCourt (parses
   `include/cgap_aoq.h` at test time).
2. **W2-C** — tag-disagreement-never-echo and zero-byte-accept
   falsifiers addressed by zd-13 J2 (stale tag → DIGEST_MISMATCH, no
   echo, EOF follows; fresh sidecar accepts the intact window) and J3
   (zero-byte → clean exit 0; short prefix → typed INCOMPLETE_FRAME).
3. **W2-D** — unknown-op and admit-reimplementation falsifiers
   addressed by zd-42 court 4 (TestAdmitUnknownOpTypedRefusal, typed
   refusal never nil,nil) and the chain delegating to RequireBoundary
   → Boundary.Seal/Unseal (no Go-side admit re-implementation).

Whole-plan falsifiers: no cgo (`grep -c runtime/cgo` = 0, zd-29); no
external dep added to cgap-aoq/cgap-arrow (zd-6 `cargo check -p
cgap-aoq` exit 0; sidecar path-dep only).

## Disclosures

- This receipt runs no commands; its evidence is entirely citation of
  four on-disk lane receipts re-read this session, plus on-disk
  existence checks of the eight lane-owned files (all present,
  mtime Nov 4 01:48–03:42) and absence checks for zd-56/zd-63
  (zd-63 absent before this receipt wrote itself, self-indexed per
  the YD-5 precedent).
- Standing words are as-of-closeout: any run of the unwitnessed rungs
  (7; 8 full handshake echo; rung 0 crypto leg; rung 6 cgap side)
  after this receipt supersedes this ledger.
- Known open items carried forward: rung 7 (`make gate0`), rung 8
  full real-binary handshake echo (ZD-56, absent), stale
  aoq_sidecar janitor note (zd-42 disclosure 6), and the W2-B flake
  class on TestAoqBoundaryRealSidecar (zd-42 disclosure 5).
