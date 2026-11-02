# Forensic Lane FA6 — branchless census hostile audit

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Sandbox: /tmp/fa6-branchless (crate copy), /tmp/fa6-target (build lease).
Shared tree: untouched (read-only); only this report file written.

## Verdict summary

The branchless census court is REAL but INCOMPLETE and, as of this session,
FAILING on the current tree. Three of five adversarial suspicions are
CONFIRMED against it; two are REFUTED with receipts.

| # | suspicion | verdict |
|---|---|---|
| S1 | cherry-picked symbol list | CONFIRMED — 11/16 pub fns censused; 5 holes |
| S2 | link-dead-code evasion | REFUTED as evasion; CONFIRMED as load-bearing (fail-closed) |
| S3 | zero production consumers | CONFIRMED — the crate is dead library code |
| S4 | 19/19 branchless truth | REFUTED on current tree — 18/19, exit 1, select_slice still branchy |
| S5 | inline-site verdict gaps | CONFIRMED — court scans symbols, not callers |

## S1 — Cherry-picked symbol list: CONFIRMED

Pub surface (grep `pub fn`/`pub const fn` across src/, this session):
16 public functions.

Censused (branchless_census.rs:42-68): 11 fns, 19 fn-target cells
(11 host + 8 riscv). Audit holes — pub fns with ZERO census cells:

| pub fn | location | host flow-ctl count (this session) |
|---|---|---|
| eytzinger_search | eytzinger.rs:71 | 5 (branchy) |
| to_eytzinger_layout | eytzinger.rs:99 | 9 + alloc machinery |
| checked_uniform_select | select.rs:44 | 24 + 11+11 in inlined `all` closures |
| swar_lt | swar.rs:36 | not measured (not in any object) |
| swar_splat | swar.rs:72 | stripped even with link-dead-code=yes (never called, no standalone emission at this opt) |

## S2 — link-dead-code evasion: court fails closed

Rebuilt the exact census wrapper (all 5 modules, same flags minus
`-C link-dead-code=yes`) → 0/11 censused symbols present in the object
(llvm-objdump, 0 mentions each). All cells would read MISSING → exit 1.
So the flag is load-bearing, and the court FAILS CLOSED without it; it is
not used to hide dead code. Matches the w6-b6 receipt's own witness
(19/19 MISSING).

## S3 — production consumption: ZERO (crate is dead library code)

- `grep -rn "cgap_branchless" crates --include=*.rs` outside the crate:
  exactly one hit — `crates/bench/benches/challenger_arrow.rs:58` is a
  benchmark FUNCTION NAME (`arrow_filter_cgap_branchless`), which calls
  `filter_quads_eq` from cgap-arrow's own `predicates`/`branchless`
  module — not the cgap-branchless crate.
- cgap-gate: dep declared (Cargo.toml:16, lane B7), but gate src has
  zero references — an unused, dead dependency.
- cgap-p28: no dep, zero sites (w6-b8 documents this honestly, verdict
  PARTIAL / zero eligible swap sites).
- cgap-arrow/bn3/storage: zero references.
- The w6-b7/w6-b8 receipts are honest about the divergence (PARTIAL,
  zero swaps) — the fraud is not in the receipts, it is in the scope:
  the "centralized branchless ALU" claim covers library code consumed
  by nobody.

## S4 — Current-tree truth: census exits 1 NOW; RV1 repair is a retraction, not a fix

Commands (this session, sandbox copy, shared tree untouched):

- `rustup run nightly rustc --edition 2021 tests/branchless_census.rs -o /tmp/fa6-target/b6-census-bin` → exit 0 (build)
- `/tmp/fa6-target/b6-census-bin` → **exit 1**, 19 cells: 18 PASS,
  1 CENSUS-FAIL: `host select.rs::select_slice -> BRANCHY 1x ["cbz"]`
- Anti-vacuity arm healthy: `naive_if_cmov BRANCHY 3x [tbz, b, b]` (court CAN flip)
- `CARGO_TARGET_DIR=/tmp/fa6-target cargo +nightly test -p cgap-branchless`
  (sandbox copy) → exit 0, 27 passed, 0 failed — differential fuzz vs
  naive oracles is green on the repaired bodies (50k pairs/kernel in
  swar.rs; arithmetic.rs:85 differential court).

### The RV1 repair itself

`select_slice` (select.rs:31-38) was "repaired" by changing SEMANTICS:
it no longer selects per-lane; it returns one whole slice based on
`cond.first()`. The `if pick_a { a } else { b }` still emits one
DATA-DEPENDENT branch (cbz on cond[0]) — the hot-path contract
("zero data-dependent control flow", select.rs:3-10) is violated by the
repaired form itself, and the census still fails on it. So:

- the 19/19 claim is FALSE on the current tree (18/19, exit 1);
- the repair traded a mixed-mask bug for a semantics retraction
  (per-lane select -> whole-slice pick) and did not reach branch-free;
- the repaired semantics are at least correct and tested (pointer-identity
  tests, mixed-mask panic court, fuzz corpus with 0xA20C_0FFE_E5A7_E000);
- riscv saturated_subs_p28: DIS confirmed branch-free this session
  (15 instrs: sltu/sub/not/and/or/srli/srai/sw/ret, zero branch mnemonics).
  That cell is real.

### cargo test does not run the court

`CARGO_TARGET_DIR=/tmp/fa6-target cargo +nightly test -p cgap-branchless`
compiles branchless_census.rs as a HARNESS binary and runs it as
"running 0 tests ... ok" — the court's fn main never executes under
cargo test. The census only runs via the manual rustc-build-and-run path
documented in the file header. Any `cargo test` green for this crate is
NOT a census pass (witnessed this session: cargo test exit 0 while the
standalone census run exits 1).

## S5 — inline-site verdicts: court scans symbols, not callers — CONFIRMED

`eytzinger_next`/`eytzinger_restore` are `#[inline(always)]`
(eytzinger.rs:42,54) and `#[inline]` fns across select/arithmetic/swar.
The census compiles modules ALONE (no real callers) with
`--inline-threshold=0`, then judges the STANDALONE symbols. At real
call sites the inlined bodies are judged only if the caller is censused
— and no caller is. Witness: `padded_at` (eytzinger.rs:125, #[inline])
appears as its own symbol with 2 branches on host; branches inside
inlined copies of the censused fns at uncensused call sites
(eytzinger_search: 5, to_eytzinger_layout: 9 flow instrs) are invisible
to the court. The claim "the hot fns are branchless" is proven only for
isolated-symbol codegen, not for the engine's actual call graph.

Also latent (not flipped this run): the riscv32 `is_flow` classifier
matches an exact mnemonic list (beq/bne/blt/bge/bltu/bgeu/jal/jalr) and
would MISS RVC compressed branches (c.beqz/c.bnez/c.j/c.jal) and
b*-alias forms (bltz/bgez/blez/bgtz). This run's riscv objects contained
0 RVC branches, so no cell was corrupted by the hole — but the doc
comment's "plus any `b*` conditional branch" claim is not what the code
does (branchless_census.rs:18-19 vs 219-228).

## Commands + exits (this session)

| command | exit |
|---|---|
| census binary build (sandbox) | 0 |
| `/tmp/fa6-target/b6-census-bin` (current tree) | **1** — 18/19, select_slice BRANCHY cbz |
| census obj rebuild WITHOUT link-dead-code | 0; 0/11 symbols present (all would be MISSING) |
| `cargo +nightly test -p cgap-branchless` (sandbox, /tmp/fa6-target) | 0 (27 passed) |
| riscv disasm saturated_subs_p28 | verified branch-free (visual, llvm-objdump) |
| rvc branch count in riscv object | 0 |

## Report (JSON terminus)

```json
{
  "lane": "FA6",
  "files_written": [
    "docs/jira/v26.11.2/adversarial/forensic-lane-06.md"
  ],
  "commands": [
    "census build+run exit 1 (18/19, select_slice BRANCHY)",
    "no-link-dead-code rebuild: 0/11 symbols",
    "cargo test -p cgap-branchless exit 0 (27 passed)",
    "riscv + host disasm audits"
  ],
  "courts_run": [
    "branchless_census (current tree, exit 1)",
    "no-flag MISSING sweep",
    "anti-vacuity arm (naive_if_cmov BRANCHY, healthy)"
  ],
  "standing": "ALIVE",
  "divergences": [
    "19/19 claim false on current tree: 18/19, census exits 1",
    "5 of 16 pub fns have zero census cells",
    "zero production consumers of cgap-branchless (gate dep is dead)",
    "cargo test never executes the census court (0 tests, vacuous green)",
    "riscv32 classifier has a latent RVC-branch hole"
  ]
}
```
