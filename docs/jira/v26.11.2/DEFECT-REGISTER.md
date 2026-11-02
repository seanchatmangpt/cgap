# DEFECT-REGISTER

Unified defect register for the v26.11.2 starDF campaign. Created by lane
G08 (2026-11-02), branch main @ 9a730320. Register-of-truth inputs:
`adversarial/THE-SMOKING-GUN-REPORT.md` plus the 2026-11-02 re-derivations
cited per row. No source edits; two doc files only (this file + the G08
receipt).

Standing vocabulary: CLOSED requires a command+exit witness run THIS
session on the exact subject; OPEN/BLOCKED carry the exact missing step.
Every CLOSED row names the receipt that would re-open it (anti-vacuity law).

## D-G08-1 -- WG2/G2-PARETO rollup rows (L1/L2/L4/L5): OPEN

- HANDOFF.md:128 (2026-11-02, lane F11): "rollup rows still UNVERIFIED".
  No post-dating receipt contradicts it: grep for the gate thresholds
  (`L1 <`, `L5 <`, `L4 <`) across docs/jira/v26.11.2/ this session hits
  only 00-ard-prd.md, 01-release-gates.md:56-88, 02-pareto-levers.md,
  adversarial/, repair-06, MEGA-REPORT and baseline research -- no
  per-level command+exit+flip witness for L1/L2/L4/L5 as a G2 receipt.
- Partial: L3 radix is CLOSED as a single level, not the rollup:
  repairs/repair-06-radix-default-flip.md:58-66 (5.6x @65,536; 78.3x @1M
  median; 23.72 ms fastest vs <75 ms; floor >20x MET) + :76-81 flip court
  (radix:true->false -> court FAILED, cargo exit 1, sha256-exact restore,
  mtime bumped, rerun exit 0). HANDOFF.md:129 says LANDED-MET only for L3.
- waves/w2-13-l4l5-courts.md:1 ("court shells (pre-lever controls)")
  records L4/L5 bench-level courts as "recorded, not faked" -- i.e. NOT
  executed as threshold courts; its cargo-test rows (:16-17) are control
  shells (exit 0), not L4/L5 threshold measurements.
- mega-bench lane-01 / lane-09 carry numbers (10.65 ns/frame;
  0.755-1.001 us/tx) but not the G2 receipt format of
  01-release-gates.md:56-88 (five numbers + per-level flip witness,
  scripts/divan2receipt.py as parser of record).
- Close condition (re-open receipt): a G2-PARETO receipt carrying all
  five level numbers + per-level flip witnesses per
  01-release-gates.md:56-88. L3 re-open trigger: any run where
  radix:nested @1M median <20x, or a flip court that accepts a mutated
  `radix` default.
- Lane G08 ran NO benches; nothing here overclaims a measured rollup.

## D-G08-2 -- FA10 102,187 B maximal no_std consumer: CLOSED (re-derived)

- Re-derived THIS session by the /tmp caller-crate method (CLAUDE.md
  footprint law), probe crate `/tmp/fa10-max-consumer` (FA10's own crate,
  same profile: opt-level=z, lto=fat, codegen-units=1, panic=abort,
  riscv32imc-unknown-none-elf):
  `cd /tmp/fa10-max-consumer && CARGO_TARGET_DIR=/tmp/g08-target cargo
  +nightly build --release --target riscv32imc-unknown-none-elf` -> exit 0
  (this session, 12:55 local).
- `llvm-size -A /tmp/g08-target/riscv32imc-unknown-none-elf/release/
  fa10-max-consumer` -> .text 102000 + .rodata 389 = **102,389 B**
  text+rodata family sum (exit 0). Verdict: under the 131,072 B envelope
  with ~22% headroom, MATCHING the FA10 witness (102,187 B) within +202 B
  nightly-toolchain drift -- same drift class already witnessed on the
  gate's live numerator (gate 606 B vs FA10's 484 B,
  countermeasures/lane-fa10r-verdict-rename.md:37-40).
- Corroboration on disk: adversarial/forensic-lane-10.md:81,115;
  hardening/lane-13-enclave-budget-witness.md:58,107 ("CITED (FA10), not
  re-measured" -- now re-measured, this row).
- FA10 remediation status: fix (c) verdict rename + fix (a) coverage
  caveat LANDED (scripts/enclave_budget_gate.py:667,670 grep witness, exit
  0 this session; scenario exits in
  countermeasures/lane-fa10r-verdict-rename.md:28-35). Fix (b) -- a
  BINDING maximal-consumer gate arm -- remains OPEN (lane-fa10r:46-48).
- Re-open trigger: a maximal-consumer build whose text+rodata sum exceeds
  131,072 B, or any no_std image gaining the std-gated scan reader
  (forensic-lane-10.md:87-93) without a gate re-measure.

## D-G08-3 -- FA1 bench-doc framing (chains-ingress.rs:24): OPEN

- crates/bench/benches/chains-ingress.rs:24 still reads:
  "`ingress_chain` -- the full 3-stage chain (the product)." FA1 witness:
  the escalated BLAKE3 leg is structurally unreachable on chain-admitted
  frames (escalate predicate == stage-1 refusal over all 256 flag bytes;
  escalated+dam-admitted = 0 of 65,536 frames; honest escalated cost ~316
  ns/frame = 33x headline) -- adversarial/forensic-lane-01.md:13,36,88-89;
  THE-SMOKING-GUN-REPORT.md section 1 row 1.
- This file is NOT G08's to edit. Recommendation to the owning lane: exact
  edit = replace the line-24 phrase "the full 3-stage chain (the product)"
  with "the 3-stage fast path (escalated leg structurally unreachable on
  chain-admitted frames, 0/65,536; FA1)" -- wording-only, no bench code
  change. Grade per smoking-gun section 2: HONEST-DISCLOSED at kernel,
  OVERCLAIM at framing.
- Close condition: the owning lane's receipt quoting the edited
  chains-ingress.rs:24 plus a re-run of the ingress bench (exit 0).

## D-G08-4 -- FA2 / Cpk n=1 (branchless 300.5M dec/s): OPEN (owned by G09)

- THE-SMOKING-GUN-REPORT.md section 3 item 3: "Single-run Cpk is
  impossible... the honest citation is single-session, this-session re-run
  246.1 M/s (18% noise band)"; forensic-lane-02.md:45-50 (re-run 4.06 ns /
  246.1 M-ops/s vs receipt's 3.33 ns / 300.5 M/s), :186 (verdict row).
- Pointer: G09 owns the 30-trial rig per dispatch. Close condition: a G09
  receipt with a 30-trial distribution (n>=30, stated method + machine)
  and a capability statement computed from it -- not from n=1. Until then
  the lawful citation is "single-session, 246.1-300.5 M/s band".

## D-G08-5 -- RV1 per-lane select_slice semantics: OPEN (law registered)

- Witness: forensic-lane-06.md:73-92 (RV1 section) and
  THE-SMOKING-GUN-REPORT.md section 4 RV1 row ("LANDED but INSUFFICIENT";
  census exits 1 on the current tree, 18/19, host select_slice BRANCHY
  1x cbz).
- Formal law (typed, no code edit by this lane):

  LAW RV1-PERLANE: `select_slice(a, b, cond)` is a per-lane select iff
  for every lane index i, result[i] is derived solely from a[i], b[i] and
  cond[i]. A form returning a whole slice chosen by cond[0] (the current
  repair, select.rs:31-38) is a SEMANTICS RETRACTION: it satisfies
  pointer-identity tests but is not a per-lane select and still emits a
  data-dependent branch (cbz on cond[0]), violating the hot-path contract
  "zero data-dependent control flow" (select.rs:3-10).
  Admissibility of any "branch-free select" claim requires BOTH:
  (1) semantic: differential oracle over the full lane-width domain
      (all cond bit patterns, mixed masks included) against a naive
      per-lane oracle;
  (2) codegen: census cell PASS (zero branch mnemonics) on host AND
      riscv32imc for the exact shipped symbol, via the manual
      rustc-build-and-run census path -- cargo test cannot see the court
      (forensic-lane-06.md:94-101, witnessed: cargo test exit 0 while
      standalone census exits 1).
  A claim meeting only (2) is codegen-true but semantics-false; only (1)
  is a retraction-flavored lie; both are required. Census 19/19 may be
  re-claimed only when the standalone census binary exits 0 on the tree.

- Close condition: a repair receipt with (1)+(2) witnesses and census
  exit 0. Re-open trigger: census exit 1 on any later tree.
