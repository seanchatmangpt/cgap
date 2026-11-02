# V2-STARDF-CLOSURE — stardf gap-closure spec (v26.11.2)

Charter: CHARTER-CGAP-STARDF-v26.11.2. Lane V2-STARDF-CLOSURE (lane-runner).
Lease: this file plus `waves/v2-04-stardf-closure.md` (receipt). No git
writes. Exit-code contract for all gates: 0 pass / 1 gate fail / 2 subject
missing / 3 loud skip.

## Sources read (this lane, 2026-11-01)

- `/Users/sac/stardf/docs/ticket-001-state.md` — S-3 receipt: contract
  digest-bound to live purrdf.h (sha256 6ef9b4a0…ad45, shasum exit 0);
  F4 header-freeze court witnessed; F1 census exit 1 with 11 missing type
  mappings; verifiers only in ggen-legacy; D1 commit split pending.
- `/Users/sac/stardf/docs/WORK-BOARD.md` — S-7 fold: F1 exit 1 re-run
  witnessed (128 AbiFunction, 22 header typedefs vs 11 missing); P5
  license-gate re-point UNAPPLIED (`parents[3]`, gate [SKIP] exit 1);
  build_receipt.py:43 same 2-up bug; critical path R1 -> sparql-eval
  SIGABRT -> process membership -> D1.
- `build-baseline.md` — ABSENT. Not on disk at
  `/Users/sac/stardf/docs/build-baseline.md` (find: 0 hits). WORK-BOARD
  confirms S-2 absent at its 45-min poll deadline. Cited as named-absent,
  never read; nothing in this spec rests on it.
- `/Users/sac/stardf/docs/parity-inventory.md` — S-5 audit: 17 surfaces
  RUNS-GREEN in a /tmp sandbox; upstream-parity (KD-12/KD-14) has zero
  courts; "zero of 10 vs 10 golden digests shared"; node --check exit 1
  on the full-parity workflow; blake3 single-identity gate exit 0.
- `/Users/sac/stardf/arrow/docs/DIVERGENCES.md` KD-12/KD-14 — both
  PARTIALLY CLOSED, no differential court vs upstream exists.
- Live witnesses run this lane (commands in the receipt file): node
  --check exit 1 (Illegal return statement, line 541) on
  `/Users/sac/stardf/workflows/v26.11.1-full-parity.workflow.js`;
  verify_stardf_licenses.py run -> `[SKIP] compat register missing:
  /Users/sac/authority/...` (the P5 parents[3] bug, live); grep confirms
  all 11 missing typedefs exist in
  `/Users/sac/purrdf/crates/rdf-capi/include/purrdf.h`.

## (a) 11 missing C-ABI type mappings -> stardf_contract.ttl

Current witnessed state: `python3 scripts/verify_stardf_ontology.py --ttl
arrow/authority/stardf/stardf_contract.ttl` -> exit 1, `[FAIL] Missing
type mappings:` the 11 typedefs below (witnessed in S-3 receipt and
re-run per WORK-BOARD). The contract has 13 TypeDefinitions
(`stardf_contract.ttl:75-147`), 5264 lines, digest-bound to live
purrdf.h. All 11 typedefs are present in purrdf.h (grep line receipts in
the waves receipt).

v26.11.2 work — add 11 `stardf:TypeDefinition` individuals to
`/Users/sac/stardf/arrow/authority/stardf/stardf_contract.ttl`, following
the existing block shape exactly (rdfs:label, stardf:predecessorSymbol =
purrdf name, stardf:targetSymbol = stardf rename):

| # | predecessorSymbol (purrdf.h) | header line | targetSymbol |
|---|---|---|---|
| 1 | PurrdfTermView | 1137 (`} PurrdfTermView;`) | StardfTermView |
| 2 | PurrdfGraphMatch | 551 (typedef int32_t) | StardfGraphMatch |
| 3 | PurrdfResourceVector | 1242 | StardfResourceVector |
| 4 | PurrdfQueryGovernors | 1204 | StardfQueryGovernors |
| 5 | PurrdfGovernorEvidence | 1299 | StardfGovernorEvidence |
| 6 | PurrdfGovernedEntailmentEvidence | 1339 | StardfGovernedEntailmentEvidence |
| 7 | PurrdfPartialCertificate | 1317 | StardfPartialCertificate |
| 8 | PurrdfGovernorTrip | 713 (typedef int32_t) | StardfGovernorTrip |
| 9 | PurrdfAbiVersion | 1375 | StardfAbiVersion |
| 10 | PurrdfCapabilities | 1410 | StardfCapabilities |
| 11 | PurrdfStr | 1090 | StardfStr |

Exit criterion: the F1 census verifier exits 0 against the amended
contract (22+ type mappings, zero missing), AND the anti-vacuity arm
`verify_stardf_ontology.py --mutate drop-type` exits nonzero on a /tmp
sandbox copy of the TTL (sha256 fail-closed restore). A green census with
no witnessed flip is VACUOUS.

Path note (divergence, from ticket-001-state.md): the contract lives at
`arrow/authority/stardf/…`, not `authority/…`; the ticket's ladder paths
name the legacy layout. Ladder commands in this campaign use the arrow-
relative path. The stale root-level `authority/stardf/` duplicate that
WORK-BOARD flags as delete-pending is out of this lane's lease.

## (b) License gate: scripts/verify_stardf_licenses.py

Divergence first: the dispatch said "witnessed never-written in either
tree." That is no longer true of stardf — the script EXISTS at
`/Users/sac/stardf/arrow/scripts/verify_stardf_licenses.py` (read this
lane) and already carries the gated-class law (MIT / Apache-2.0 /
MulanPSL-2.0 token containment, P28-SIL consumer without a compatibility
receipt = FAIL). What is witnessed broken is its ROOT RESOLUTION and the
cgap side:

- `REPO = Path(__file__).resolve().parents[3]` (lines 19-22 region) —
  resolves two levels above the arrow workspace, so the gate runs against
  `/Users/sac/authority/...` and dies `[SKIP] compat register missing`,
  exit 1 (witnessed this lane). WORK-BOARD item 1: two one-liners
  (`parents[3]` -> `parents[2]`; same class of bug at
  `build_receipt.py:43`).
- cgap has no equivalent gate under `/Users/sac/cgap/scripts/` (ls this
  lane: no verify_stardf_licenses.py — expected; the gate is stardf-
  scoped).

v26.11.2 work:
1. Apply the P5 two-liner re-point in the stardf tree (lane lease to be
   claimed by the executing lane; this file only specs it).
2. Add the SPDX header invariant arm the dispatch names: every gated
   crate manifest carries a valid `license`/`license-file` SPDX
   expression, and every source file's Apache-2.0/MIT header notice is
   present; a crate whose headers fail the invariant FAILS the gate
   (exit 1), a missing register is a loud `[SKIP]` (exit 3 per the
   campaign exit-code contract, superseding the current bare exit 1).
3. Positive run: `python3 scripts/verify_stardf_licenses.py --root arrow`
   -> exit 0 with the in-tree
   `authority/stardf/license-compat.json` register (P1 landed it;
   WORK-BOARD confirms 6 entries, ls exit 0).

Exit criterion: gate exit 0 on the real tree, PLUS a non-vacuity flip —
shim a P28-SIL-gated license into a /tmp sandbox manifest (sha256
fail-closed restore) and the gate must exit 1. No witnessed flip = FAIL.

## (c) KD-12 / KD-14 differential parity courts vs pinned purrdf fixtures

Current witnessed state (parity-inventory.md, S-5): parity WITH UPSTREAM
purrdf is CLAIMED-not-PROVEN. KD-12 (digest identity vs purrdf-hash): no
differential test exists — grep finds none. KD-14 (columnar
digest-ordering parity): the upstream `golden_bytes.rs` pins
(/Users/sac/purrdf/crates/columnar/tests/golden_bytes.rs) and the
stardf-columnar PINNED set share ZERO digests — 10 vs 10, disjoint.
`golden_bytes` proves self-determinism, not upstream parity.

v26.11.2 work:
1. KD-12 court: `tests/differential_purrdf_hash.rs` in stardf-columnar —
   over a pinned corpus of fixture inputs, assert
   `stardf_blake3_digest(x) == purrdf_hash_digest(x)` per the KD-12
   reconciliation law (the divergence registers that stardf binds blake3
   while purrdf binds purrdf-hash; the court must either prove equality
   on the pinned corpus or convert KD-12 to a REGISTERED divergence with
   a named transform law — never a silent skip).
2. KD-14 court: `tests/differential_digest_ordering.rs` — replay the
   upstream purrdf columnar golden corpus through the stardf columnar
   writer and assert the digest ORDERING relation (same inputs -> same
   order) even where the byte encoding deliberately diverges; a corpus
   where orderings differ FAILS the court.
3. Fixtures: pin to the live upstream tree at a recorded rev (purrdf
   0da05783 per VENDOR-PROVENANCE) — copy the 10 golden vectors into a
   `tests/fixtures/purrdf-parity/` dir with a MANIFEST.sha256; courts
   read fixtures, never the live tree at test time.

Exit criterion: both courts run (command+exit in the receipt), each with
a named anti-vacuity mutation that flips (e.g. mutate one fixture byte in
a /tmp sandbox -> court FAILS), and DIVERGENCES.md KD-12/KD-14 rows move
to CLOSED (court receipt cited) or stay OPEN with the registered
transform law. Zero of 10 shared digests must drop to a proven relation
or a registered divergence — the current CLAIMED-not-PROVEN posture is
the thing this lane kills.

## (d) workflows/v26.11.1-full-parity.workflow.js line 541

Current witnessed state: `node --check
/Users/sac/stardf/workflows/v26.11.1-full-parity.workflow.js` -> exit 1,
`SyntaxError: Illegal return statement` at line 541 (`return {`) — run
this lane, node v26.10.0. (DIVERGENCES note: the file is at the stardf
REPO ROOT `workflows/`, not `arrow/workflows/`; parity-inventory.md's
bare path is ambiguous.) The full-parity campaign workflow has therefore
never been parseable as committed.

v26.11.2 work: wrap the top-level return in a function boundary (the
standard repair: enclose the campaign body in `function main() { … }` +
`main();` or convert the return to a top-level statement). Constraint:
the repair changes ONLY the syntax envelope — no semantic edits to the
campaign steps — and is proven by `node --check` exit 0 PLUS a
byte-sweep of the step list before/after (diff shows envelope-only
delta).

Exit criterion: `node --check` exit 0 witnessed in the receipt; a
mutated re-introduction of a bare top-level `return` in a /tmp copy
re-triggers exit 1 (non-vacuity of the check itself).

## (e) std -> no_std gate split: stardf-hash / stardf-datalog

Charter gate G2 (00-ard-prd.md §4): "stardf-hash and stardf-datalog
compile #![no_std] (std-linkage gate exits nonzero on std)."

Current witnessed state: dispatch cites 392 errors under an riscv
no_std probe at `stardf-hash/src/dispatch.rs:292`. This lane located the
file (`/Users/sac/stardf/arrow/crates/stardf-hash/src/dispatch.rs`) and
read the region (Vec/collect-based dispatch table construction around
lines 285-300) but did NOT re-run the riscv probe (no riscv target
guaranteed this session) — the 392-error count is DISPATCH-CLAIMED,
status UNVERIFIED until the executing lane re-runs:

```
cargo build -p stardf-hash --target riscv32imc-unknown-none-elf
cargo build -p stardf-datalog --target riscv32imc-unknown-none-elf
```

v26.11.2 work:
1. Re-witness the failure mode (counts are targets, not facts, until
   re-run): record per-crate error counts + first-error file:line.
2. stardf-hash/src/dispatch.rs: replace the Vec-collect dispatch-table
   construction (lines ~285-300) with an allocation-free static slice
   iteration (matching the zero-allocation arena invariant in
   ticket-001-state.md's contract bounds); add `#![no_std]` +
   `extern crate alloc` where alloc is genuinely required, `#[cfg]`
   split for the std-only surface.
3. stardf-datalog: same split; enumerate its std dependencies and gate
   them behind a `std` feature with a no_std default.
4. Gate: `scripts/no_std_gate.sh` (or equivalent) that builds both
   crates under the riscv target and exits 0 on success / 1 on any
   std-linkage error / 2 subject-missing / 3 loud skip when the target
   is not installed.

Exit criterion: both crates build under the riscv target (exit 0
witnessed), the gate's non-vacuity arm re-introduces a std dep (e.g.
`use std::` line) in a /tmp sandbox and the gate exits 1 — a gate that
passes with a std dep present is VACUOUS. stardf-sparql-eval lib SIGABRT
(WORK-BOARD row) is a sibling RED, NOT in this lane's lease — named so
it is not silently absorbed.

## Standing (of this spec)

SPEC-ALIVE (authored this session from on-disk receipts + live witnesses;
all work items UNVERIFIED until the executing lanes cite command+exit
runs). Divergences recorded: build-baseline.md absent; license gate
exists-but-broken (not "never-written"); workflow.js at repo-root
workflows/ not arrow/workflows/; 392 riscv error count UNVERIFIED;
contract path arrow-relative vs ticket's legacy layout.
