# BR6 — branchless reuse map: parquet footer/ring

Anchor commits: P5 (MetaBuf fixed `[u8;512]` scratch + inline spans), FA7 (typed
overflow arms), MB7 (seal 16.25 ms @1M). Zero-alloc seal witnessed.

## R1. MetaBuf write helpers — REJECT (branches ARE the safety mechanism)

- `footer.rs:119-126` (`MetaBuf::push`): `if self.pos >= self.buf.len()` → typed
  `ParquetError::RunOverrun`. Bounds check + typed refusal on every byte.
- `footer.rs:128-135` (`MetaBuf::extend`): same class.
- `footer.rs:133-143` (`varint`): per-byte `push` loop, typed refusal per byte.
- `footer.rs:147-163` (`field_header`): `(1..=15).contains(&delta)` branch is
  PROTOCOL (delta vs long form selects different wire bytes), not overhead.

Classification: all are COLD-path bounds checks on a bounded 512-byte scratch.
The only "branchless" forms are (a) a computed clamp — fail-open, replaces a
typed refusal with silent truncation of the footer, or (b) branchless-emit-then-
single-check, which still needs the same compare at the end and saves nothing on
a ≤512-byte emit that runs once per block. Verdict: **REJECT**. Substitution
weakens typed refusal (fail-open) for zero measurable gain on a cold path.

## R2. Seal-path hot loop — the real heat is in mmap_writer, not footer

- `mmap_writer.rs:421-482` `encode_hybrid_into`: the hot loop. Branch structure:
  - `mmap_writer.rs:359-372` `push_byte`: `buf.get_mut(*pos)` match per byte —
    bounds-check branch per lane byte, plus `Result` propaation.
  - `mmap_writer.rs:335-357` `push_varint`: capacity check PER BYTE inside the
    varint loop — 5 checks for a 5-byte varint.
- `mmap_writer.rs:436-448` run detection: nested `while` equality scans, 2 deps
  per element (inner run-length scan + outer group-scan). Branchy but data-
  dependent; branch predictor hides much of it.

Primitive candidate (P1): **batched-capacity fast path** — one check per group
(8 lane bytes) instead of per byte: emit 8 lane bytes via a single
`buf[pos..pos+8]` slice write after one `pos+8 <= cap` compare, fall back to the
checked per-byte path only when the group would overrun. Preserves the typed
refusal (the fallback still refuses; wraparound impossible since `pos` only
advances by checked steps). Gain class: removes ~1 predicted-taken branch + 1
`Result` propagation per lane byte @1M values; MB7 target: seal 16.25 ms →
~13 ms (TARGET, unmeasured).
Primitive candidate (P2): hoist the varint capacity check out of the loop —
compute `varint_len(v)` (mirror already exists at `mmap_writer.rs:329`), one
check, then unchecked emit; refusal semantics unchanged (same `RunOverrun`
before any byte written).

REJECT candidate: the run-detection nested scans (`mmap_writer.rs:436-448`) —
SIMD/branchless run detection would change grouping decisions only if the two
scans disagree with the scalar semantics; the differential court pins byte-exact
output, and the data-dependent branch is already well-predicted. REJECT until
P1/P2 measured and still hot.

## R3. Sovereign lease/claim transitions — DEFER to FA9FIX

The 0x4347_0006 atomic claim + drain guard design is FA9FIX's (sovereign.rs).
Do not redesign here. Post-landing census items for that lane's receipt:
- the claim is a single acquire load on the hot path (no compare-exchange in
  steady state),
- the drain guard branches are on a COLD path (drain only),
- `0x4347_0006` magic is byte-pinned — no branchless rewriting of the magic
  compare.

## Court mutants (per item)

- R1 REJECT: mutant = clamp `push` capacity check (`footer.rs:120`) → court must
  FAIL (footer truncation must stay typed). A court that passes on the clamp is
  VACUOUS.
- R2 P1: mutant = drop the group-capacity fallback check → `RunOverrun` court on
  a scratch sized exactly to `body_len` must flip to error.
- R2 P2: mutant = compute `varint_len` but skip the one remaining check → same
  exact-fit scratch court flips.
- R3: census-only; court owned by FA9FIX.

Standing: map only, no code written, no commands required beyond reads; gains
are TARGET (unmeasured). Shared tree untouched.
