# 09-overnight-mutation-set

Consolidated anti-vacuity mutation register for the v26.11.2 overnight loop.
Sources (cited, not redefined): `docs/jira/v26.11.2/01-release-gates.md`
(FLIP blocks, G1-G8) and `docs/jira/v26.11.2/02-pareto-levers.md`
(per-lever mutations, L1-L5). No invented mutations - every row here is
named in one of those two documents.

## Overnight-loop mechanics (applies to every row)

1. The loop applies each mutation to a `/tmp` sandbox copy of the subject
   tree/file - never the shared tree.
2. The owning gate/court must FLIP (exit nonzero or the named typed error).
   An unflipped mutation is `VACUOUS-GATE`: the loop exits nonzero.
3. Restore is sha256-exact AND the restored file's mtime is bumped (or
   `cargo clean -p <crate>` issued) before any rebuild - byte-exact restore
   alone leaves cargo's fingerprint stale and the mutated binary survives
   (`docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184`).
4. Skips are loud: `exit 3` + `[SKIP] <reason>`; a skip is never a pass.
5. Exit-code contract: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.

## Register: G-gates (01-release-gates.md FLIP blocks)

| ID | Subject (file:line) | Mutation | Expected kill signal | Owning gate/court | Flip status |
|---|---|---|---|---|---|
| G1-M1 | one publishable crate Cargo.toml (copy) | Strip `description` | release_check on the copy FAILs (exit 1) | G1-PUBLISH / `scripts/release_check.py` | [UNFLIPPED-t26.11.2] |
| G2-M1 | bench kernel hot loop, per level L1-L5 (copy) | Remove `#[inline]` / inject bounds check / add `core::hint::black_box` around hot loop | Level perf threshold flips to FAIL | G2-PARETO / criterion + `scripts/divan2receipt.py` | [UNFLIPPED-t26.11.2] |
| G3-M1 | mat closure kernel, crates/bench (copy) | Skip one derivation rule or de-duplicate one join arm | Changed derived-fact count detected, or throughput threshold flips | G3-MAT-BENCH / mat_closure_bench | [UNFLIPPED-t26.11.2] |
| G4-M1 | DRED increment path, crates/bench (copy) | Skip the entropy recompute | Entropy-variance number changes; <0.15 ns variance gate flips | G4-DRED / dred_bench | [UNFLIPPED-t26.11.2] |
| G5-M1 | no_std subject, riscv32imc (copy) | Add a `Vec::new()` push (alloc symbol) or a `panic!` site | `scripts/symbol_gate.py` FAILs on the copy | G5-SILICON / symbol_gate.py | [UNFLIPPED-t26.11.2] |
| G6-M1 | a workspace Cargo.toml (copy) | Append `datafusion = "42"` | Gate on the copy FAILs | G6-NO-DF / grep -ri datafusion gate | [UNFLIPPED-t26.11.2] |
| G6-M2 | real temp-copy restore | Self-check corrupts a restore | Gate refuses the restore mismatch (fail-closed) | G6-NO-DF self-check | [UNFLIPPED-t26.11.2] |
| G7-M1 | patent claims.txt (copy) | Merge one claim with a prior-art mechanism (near-duplicate) | Verdict drops to 3/4, gate FAILs | G7-PATENT / verify_ost_design_around.py | [UNFLIPPED-t26.11.2] |
| G8-M1 | mapping table (copy) | Remove one of the 11 mappings | Census exits nonzero AND golden digest breaks | G8-STARDF-F1 / stardf_f1_census + stardf_golden + kd_flip KD-12/KD-14 | [UNFLIPPED-t26.11.2] |

## Register: L-levers (02-pareto-levers.md per-lever mutations)

| ID | Subject (file:line) | Mutation | Expected kill signal | Owning gate/court | Flip status |
|---|---|---|---|---|---|
| L1-M1 | ingress stage fold (cgap dam FrameView chain) | Corrupt a slot byte or stage code | Per-stage checksum assert (vs untimed preflight fold) fires; escalation still refuses the spoliated frame | L1 ingress court (P3-16 chain bench) | [UNFLIPPED-t26.11.2] |
| L2-M1 | committed page window byte, cgap-parquet (copy) | Flip one committed window byte before `verify_seal` | `ChecksumMismatch { region: Body }` -> exit 101 | L2 parquet seal court (P3-17) | [UNFLIPPED-t26.11.2] |
| L3-M1 | BN3 unify comparator, crates/cgap-bn3/src/unify.rs:11,35-39,274 | Binding-alias comparator swap (doe-4 SHIM-DOE4 class) | Differential oracle assert (match count == raw-word O(N) hash join, no unify code in oracle path) flips; M12-7 source_mask popcount assert rides along | L3 BN3 unify court (P3-18 + doe-4) | [UNFLIPPED-t26.11.2] |
| L4-M1a | committed header byte, cgap-ffi sovereign commit | Corrupt one committed header byte (refusal class) | Per-stage `CgapStatus` asserts fire | L4 sovereign commit court (P3-19) | [UNFLIPPED-t26.11.2] |
| L4-M1b | slab write in the commit path | Drop a slab write | Per-stage `CgapStatus` asserts fire | L4 sovereign commit court (P3-19) | [UNFLIPPED-t26.11.2] |
| L4-M2 | P28 digest byte | Flip one digest byte | Exactly `CGAP_ERR_P28_SPOLIATION` (P28 spoliation negative control) | L4/P28 spoliation negative control | [UNFLIPPED-t26.11.2] |
| L5-M1a | upstream receipt | Bit-flip an upstream receipt | validate refuses | L5 seal chain court (P3-19/BENCH-4) | [UNFLIPPED-t26.11.2] |
| L5-M1b | crossing | Demote a crossing | No proof emission - `proven == N/2` assert flips | L5 seal chain court | [UNFLIPPED-t26.11.2] |
| L5-M1c | receipt digest | Flip a receipt digest | verify returns DigestMismatch | L5 seal chain court | [UNFLIPPED-t26.11.2] |

## Flip-status legend and prior witnesses (same mutation CLASS, v26.11.1)

All rows default [UNFLIPPED-t26.11.2]: no mutation in this register has been
executed against the v26.11.2 tree. Prior same-class witnesses (citation only,
not a v26.11.2 flip):

- L2-M1 class (committed byte flip -> exit 101): witnessed P3-17,
  `docs/jira/v26.11.1/waves/bench-chains-parquet.md:73-75` ("COURT FLIPS").
- L3-M1 class (SHIM-DOE4 comparator swap): witnessed
  `docs/jira/v26.11.1/waves/a12-finish-receipt.md:87-99` (FLIP rc 101) and
  `docs/jira/v26.11.1/waves/a12-wave8-audit-receipt.md:63` (M12-7 popcount
  court flips under projector dedupe).
- L5-M1 class (bit-flip / demote / digest-flip triple): named and executed
  in `docs/jira/v26.11.1/waves/bench-chains-seal.md:57-58`.
- L4-M2 class (flipped digest byte -> spoliation): witnessed
  `docs/jira/v26.11.1/waves/gate-4-spoliation.md:15-18` (2560 mutations /
  2516 real deaths; flipped-byte negative control fails) and
  `docs/jira/v26.11.1/waves/a17-wave2-audit-re-audit.md:63-69`.
- G5-M1 class (symbol-gate flip): `docs/jira/v26.11.1/waves/gate-5-silicon.md`
  records ELF size-law flips (R11), same gate family.
- G-gates G1/G3/G4/G6/G7/G8 FLIP blocks: no v26.11.1 same-class witness found
  in waves/ this session; UNVERIFIED, remains [UNFLIPPED-t26.11.2].
