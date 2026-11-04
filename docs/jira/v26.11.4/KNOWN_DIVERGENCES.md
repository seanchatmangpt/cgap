# KNOWN DIVERGENCES — v26.11.4

Probe-receipt register of SDK/wire divergences. A divergence closes only via
its close condition; a court that documents a row is not a fix.

## KD-1 — version-refusal status-word collision (OE-4)

- **Surface**: `crates/cgap-ffi/src/aoq_session.rs` `status_of` and
  `crates/cgap-ffi/src/bin/aoq_sidecar.rs` `session_status` (twin mapping sites,
  same pinned collisions); wire-visible via the sidecar's status word.
- **Divergence**: `SessionError::Wire(AoqError::UnsupportedVersion)` projects to
  `DIGEST_MISMATCH 0x4347A005`, the same word as a genuine digest/authenticity
  fault. A consumer cannot distinguish "tampered" from "wrong version" on the
  wire.
- **Why not fixed here**: the frozen status band 0x4347A001..A008 is fully
  allocated (8/8 words, `AOQ_STATUS_COUNT = 8`, asserted by pack gates;
  `crates/cgap-ffi/src/aoq/status_maps.rs` is a generated ggen projection —
  DO NOT EDIT). No version-appropriate word exists in the band, and remapping
  `UnsupportedVersion` onto EPOCH_MISMATCH/KEY_UNKNOWN would project a false
  class. Refusal fires at the correct parse stage (codec order
  magic→version→epoch→digest, witnessed by the L2/L3 flip-ladder legs); only
  the emitted word collides.
- **Evidence**: OE-4 receipt `docs/jira/v26.11.4/receipts/lane4-flip-ladder-court.md`
  (real release sidecar, L2 leg: version skew 3→2 with honest re-tag →
  DIGEST_MISMATCH; L3 ordering leg proves version fires before digest).
- **Close condition**: band extension at the next wire-version bump — a new
  `aoq:StatusConstant` (e.g. `UNSUPPORTED_VERSION 0x4347A009`) in
  `packs/aoq-pack/ontology.ttl`, regenerated `status_maps.rs`, then remap the
  `UnsupportedVersion` arm in BOTH mapping sites and re-pin the L2/L3 court
  legs to the new word.
- **Registered**: 2026-11-04 (FIX-or-REGISTER lane, option b).

## KD-2 — benchmark ns/frame claims do not bound contention (claims row 28)

- **Surface**: benchmark claims table (`docs/jira/v26.11.4/receipts/bm6-benchmark-claims-table.md`) ns/frame rows.
- **Divergence**: the quoted ns/frame figures are quiet-host measurements; under combined memory+CPU load the 65,536-const stage reaches **124.06 ns/frame (+826% vs quiet fastest)** — a quiet-host number is not a worst-case bound and consumers must not read it as one.
- **Evidence**: `docs/jira/v26.11.2/redteam/red-10-timing-jitter-exploit.md:43-66` (W4 method: two /tmp load generators, repo untouched; W5 table: quiet fastest 10.66, combined bw6+cpu9 slowest 124.06).
- **Close condition**: claims-table rows carry a stated load profile + a ratio fence against a sleep-free baseline (per the no-sleep doctrine), or a contention court is run alongside each published bench; flip CLOSED when the table's rows are re-published with load-profile columns.
- **Registered**: 2026-11-04 (migrated from claims row 28 / red-10).

## KD-3..KD-7 — v26.11.1 campaign ledger §5, migrated verbatim

Migrated from `docs/jira/v26.11.1/waves/v26111-campaign-ledger.md` §5
("Registered divergences"), items 1-5, transcription faithful to the
ledger text; evidence paths as cited in the ledger.

### KD-3 — P28 twin in cgap-core (outside the five-crate publish set)

frame.rs:44 `ATTESTATION_LEN: usize = 80`, const-pinned to the layout at
frame.rs:390; runtime-tied and court-enforced (f3-06:9-13, 18-19).
**Close**: the twin collapses into the generated surface (surface-gate
regeneration removes the hand twin). **Status**: OPEN.

### KD-4 — oabi-lookup UNSUPPORTED(scope)

`crates/cgap-ffi/src/oabi_lookup.rs:1-31` is a typed decision record per
the A19 four-verb surface law (A19:47, 157-166 cited in-file); A10 T5's
blocker stays BLOCKED-on-A19 (oabi_lookup.rs:36-40). **Close**: A19
lands slot resolution and oabi-lookup gains a real implementation.
**Status**: OPEN (BLOCKED-on-A19).

### KD-5 — Quad64 double-def in cgap-arrow

`crates/cgap-arrow/src/layout.rs:36` and `transpose.rs:26` both define
`#[repr(C, align(64))] struct Quad64` with the identical field lattice;
transpose.rs:3-7 self-documents the self-contained-by-design note that
predates the layout lane's landing. **Close**: collapse into one
definition (recorded coordination point; auditor grep: exactly two
struct Quad64 definitions repo-wide). **Status**: OPEN.

### KD-6 — cgap-blake3/tests/bench_tmp.rs ships in the package

f3-07:38-41; auditor ls: 564 B, mtime Oct 31 22:40. Real content,
scratch-flavored name. **Close**: rename to a non-scratch name in a
later wave. **Status**: OPEN (cosmetic).

### KD-7 — `_cgap_p28_invariant_check` header question

Zero occurrences repo-wide (auditor grep over crates/, include/, tests/
at ledger time); include/cgap.h carries the four frozen prototypes only
(cgap.h:164 = cgap_verify_p28). **Close**: GAP-1 resolves the question;
if the symbol is intended to be public, mint the header prototype; if
not, declare it internal with a gate. **Status**: PARTIAL (GAP-1 was in
flight at ledger time; nothing on disk to cite yet).
