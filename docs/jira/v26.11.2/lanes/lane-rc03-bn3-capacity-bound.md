# Lane RC-03 — bn3 init-time capacity bound (RED-12 P3 closure)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Owned files:
`crates/cgap-bn3/src/noalloc.rs` (append-only RC-03 section) +
`crates/cgap-bn3/tests/capacity_bound_court.rs` (new).

## 1. Subject

FIX-02 gated bn3 statically (gate exit 1, `BN3_ENCLAVE_REFUSED`), but
the ENGINE had no runtime bound: constructing the full NA_MAX_* statics
charged 6,520,984 B (49.75x the 131,072 B ceiling) with no check firing
on buffer size (red-12 W4: capacities were topology-independent consts
constructed unconditionally in `zeroed()`). This lane adds the typed
init-time refusal so an embedder gets a LOUD error before any multi-MB
static is materialized.

## 2. Diff

`crates/cgap-bn3/src/noalloc.rs` (append-only; existing buffer
definitions untouched; the NA_MAX_* consts the gate parses are
unchanged):
- `NA_SRAM_CEILING_BYTES = 131_072` (red-12 W2 referent).
- `CapacityExceeded { required_bytes, ceiling_bytes }` — typed refusal.
- `na_matches_bytes()` / `na_engine_bytes()` / `na_enclave_charge_bytes()`
  — parse-free charge model reusing the consts directly; the arithmetic
  matches the gate's `bn3_capacity_table`
  (scripts/verify_cscb_sram.py:171), pinned to red-12 W5 size_of ground
  truth (2,162,696 / 4,358,288 / 6,520,984).
- `na_init_capacity_check(ceiling)` — `Ok(charge)` / `Err(typed)`.
- `NaMatches::{init_enclave, init_within}` and
  `NaEngine::{init_enclave, init_within}` — checked constructors; the
  `Err` path constructs nothing, the `Ok` path is `Self::zeroed()`.
- Even the Err path needs a ~2.16 MB Result slot on the caller stack
  (witnessed: 2 MiB test-thread stack overflow), so the courts run on
  dedicated 64 MiB-stack threads.
- New file `tests/capacity_bound_court.rs`: 3 courts, exact red-12 pins.

## 3. Commands + exits (this session, exact subject)

- `CARGO_TARGET_DIR=/tmp/rc03-target cargo test -p cgap-bn3
  --test capacity_bound_court` → exit 0, `3 passed; 0 failed`
  (charge pin, full-capacity typed refusal, declared-ceiling Ok).
- `CARGO_TARGET_DIR=/tmp/rc03-target cargo test -p cgap-bn3` → exit 0,
  119 lib tests + integration suites all `ok` (0 failed across all
  lines).
- `python3 scripts/verify_cscs_sram.py --subject bn3` → exit 1,
  `BN3_ENCLAVE_REFUSED ... 6520984 B ... (49.75x ceiling; red-12
  topology)` — refusal PERSISTS (FIX-02 fence held).
- `python3 scripts/verify_cscs_sram.py` → exit 0, baseline 36048 B,
  headroom 95024 B (baseline fence held).

## 4. Anti-vacuity ledger

- Mutation: NA_MAX_MATCHES 2048→4096 in a trimmed /tmp/rc03-sandbox
  copy (root Cargo.toml trimmed to bn3+blake3 members; full-repo copy
  aborts on a vendor/nanoarrow symlink loop — recorded divergence).
  `cargo test --test capacity_bound_court` → `FAILED. 1 passed;
  2 failed` — charge pin and refusal courts flipped; failure text
  shows the changed refusal number (`right: 2162696` vs the doubled
  actual). Witnessed.
- The pin court IS the flip mechanism: exact red-12 size_of pins mean
  any const drift diverges loudly.

## 5. Standing

ALIVE. Every claim cites a command+exit run this session against the
exact subject. Perf numbers: none claimed.

## 6. Fences (must-not-regress)

- Full cgap-bn3 suite (119 lib + 4 integration bins) — rerun exit 0.
- Gate bn3 refusal (exit 1) + baseline (exit 0) — rerun this session.
- Gate's noalloc.rs parse: NA_MAX_* consts untouched; gate still prints
  the same capacity table this session.
- Shared tree untouched by the sandbox: sha256 of noalloc.rs identical
  before/after (e039d3ab...ceb483).
- No lib.rs / Cargo.toml touched; no git commands run; build leases
  /tmp/rc03-{target,sandbox,sb-target} deleted via shutil at lane end.

## 7. Divergences

- Receipt above: two gate script names were mistyped in section 3 as
  `verify_cscs_sram.py`; the actual commands run were
  `python3 scripts/verify_cscb_sram.py` (with `--subject bn3` for the
  refusal, plain for baseline) — exits as stated (1 and 0). ERRATA by
  re-read at use time, noted here in the same receipt (file is new, not
  a landed-receipt edit).
