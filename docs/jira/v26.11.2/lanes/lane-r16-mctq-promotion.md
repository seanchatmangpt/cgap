# Lane R-16 — M-CTQ promotion pass (tollgate wave)

Standing: **PARTIAL (gap registered, no claims-file edit)** — the XC-04
consolidation-bench receipt does NOT exist on disk.

## W1 Subject

`docs/jira/v26.11.2/11-measured-claims.md` (owned, read this session —
exclusivity law at header lines 1-9 confirmed: "No number may appear in
the paper that is absent from this table").

## W2 Search performed

- `ls lanes/ | grep -i -e mctq -e consolid -e xc-04` → zero hits
  (exact commands in terminus; exit 0 on the ls, grep exit 1 = no match).
- `grep -ril -e mctq -e xc-04 -e consolidation` over all of
  `docs/jira/v26.11.2/` → 13 files; none is a bench receipt with the
  three station numbers (seal throughput / FFI cost / mmap read).
- Nearest non-receipt hit checked: `lanes/lane-ipc-04-shm-transport.md`
  carries an mmap-arm µs/op table (line 57: 4.52-4.54 µs, 0 allocs) but
  it is a Go shm-transport court, NOT the M-CTQ consolidation bench;
  no seal-throughput or FFI-cost station exists in it.
- `crates/bench/benches/mctq_consolidation.rs` EXISTS on disk
  (16,684 B, mtime 2026-11-02 18:09) but no receipt records a run of
  it — the bench has not been executed and receipted by XC-04.

## W3 Decision

Dispatch's else-branch applies: NO consolidation rows were added to
`11-measured-claims.md`. A claims-file row without a run violates the
exclusivity law; a placeholder row is exactly the
claimed-unreceipted pattern that §3 of the claims file refuses.
`02-voc-ctq-matrix.md` M-CTQ-04/05/06 stay UNVERIFIED — R-07 owns that
file; this lane did not edit it (no cross-file promotion note written).

## W4 Commands / exits

| cmd | exit |
|---|---|
| `ls lanes/ \| grep -i -e mctq -e consolid -e xc-04` | 0 (empty: only bench file hit) |
| `grep -ril -e mctq -e xc-04 -e consolidation docs/jira/v26.11.2/` | 0 (13 hits, none qualifying) |
| `grep -rl -i -e "seal throughput" -e "mmap read" lanes/ waves/` | 0 (1 hit, disqualified above) |
| `grep -n ... lane-ipc-04-shm-transport.md` | 0 (disqualifying read) |
| `cat 11-measured-claims.md` | 0 (29 rows + skips re-read at use time) |

## W5 Fences (must-not-regress)

- Exclusivity law of `11-measured-claims.md` — NOT touched: zero bytes
  written to the owned file this session (`git status` on it shows no
  diff beyond any pre-session state; this lane made no edit).
- `02-voc-ctq-matrix.md` ownership fence (R-07) — not edited.
- Frozen byte-pins / seeds — not approached.

## W6 Standing detail

Every station number (seal throughput, FFI cost, mmap read) is
UNVERIFIED for consolidation purposes. UNVERIFIED does not round up to
MEASURED. [SKIP] XC-04 receipt: absent on disk at consolidation time
(2026-11-02, this session). The bench source exists but a bench run is
not a receipt.

## W7 Replay

Re-run the three greps in W4 against `docs/jira/v26.11.2/lanes/` and
`waves/`; any future XC-04 receipt landing supersedes this gap row and
unblocks a W4-D3-style addendum to `11-measured-claims.md` plus the
M-CTQ-04/05/06 promotion via R-07.

## Ledger row

{"lane":"R-16","ts":1793671924,"fleet":"tollgate","premise_hash":"gap-registered-no-xc04-receipt"}

## ERRATA (2026-11-02, post-landing; append-only per ERRATA law)

ERRATA to W3 above ("NO consolidation rows were added") and W6 (all
stations UNVERIFIED): both statements were TRUE at this receipt's
first terminus — the XC-04 receipt did not exist on disk at search
time, and the refusal to write unreceipted rows was correct.

They are now SUPERSEDED by the resumed promotion pass: the XC-04
receipt landed (`lanes/lane-xc04-mctq-consolidation.md`, exit 0 x2,
three stations + positive controls) and the consolidation rows
30/31/31a/32 landed in `11-measured-claims.md` §4 (attributed to the
R-12-titled edit in the resumed run, citing this lane's owned file).

**State of record**: `docs/jira/v26.11.2/11-measured-claims.md` §4 +
§4.1 (rows 30/31/31a/32, class consolidation-addendum, N=7
Min/Median/Max per 03-msa-gauge §2; the three instrument divergences
D1/D2/D3 carried verbatim). ADV-07 flagged W3/W6 as stale; this
errata is the correction — W3/W6 history is preserved unmodified.
Cross-instrument legs (cgo admit, Go-harness mmap read, 1M-window
seal) remain UNVERIFIED per §4.1 — the W6 non-round-up law still
binds those legs.
