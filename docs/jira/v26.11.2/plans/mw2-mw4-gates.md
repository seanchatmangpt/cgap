# Plans — MW2/MW3/MW4 + G5/G6 gate mechanization (W2/W4 execution specs)

Provenance: Plan wave lanes MW2/MW3/MW4/G5G6, on-disk-grounded. Re-read at use time.

## MW2 — std-strip, cgap-only (stardf OUT-OF-SCOPE per operator 2026-11-01)

Confirmed on disk: no stardf crates in the cgap workspace. Already no_std: cgap-arrow,
cgap-blake3, cgap, cgap-git, cgap-parquet, wasm-footprint. cgap-beam is host membrane glue
(std by construction; same ruling as symbol_gate.py host membranes — out of scope).

Kernel std leakage (non-test, witnessed):
- cgap-bont/src/shacl_compiler.rs (54 hits): std::collections → alloc::collections;
  std::fmt → core::fmt; std::error::Error → core::error::Error; std::mem → core::mem.
  oabi.rs std hits are cfg(test) — leave.
- cgap-bn3/src/unify.rs:98: HashMap/HashSet → alloc BTreeMap/BTreeSet (ordered iteration
  preserved). Other hits core-equivalents or tests.
- cgap-verification/{ingest,seal,run,rules}.rs: std::vec → alloc::vec; BTreeMap → alloc;
  std::fmt → core::fmt.
- cgap-crypto/src/chatman.rs (3): impl std::error::Error → core::error::Error (bn3
  builtins.rs precedent).
- cgap-bosym/src/slab.rs: core paths (std::mem/fmt → core; HashSet → BTreeSet) to no_std
  body; fibo_graph_export + std::fs/env fixtures under #[cfg(test)].
- cgap-storage, cgap-gate: test-only — wrap, no body edits.

Per-crate: add #![no_std] + extern crate alloc in lib.rs; #[cfg(test)] extern crate std;
extend the bn3 builtins.rs:1277-1285 must-kill assertion (`!body.contains("std::")`) to
shacl_compiler + unify.

std-linkage gate (new scripts/std_linkage_gate.py, wired `make std-gate` next to
symbol-gate, Makefile:~129):
- PRIMARY gating: cargo build --target wasm32-unknown-unknown -p <crate> for the 6 crates
  (std linkage fails to compile there); any failure → exit nonzero.
- SECONDARY gating: llvm-nm over the artifacts, forbid _ZN4std / std:: name-mangled symbols
  (symbol_gate.py empirically validates llvm-nm on this tree; fallback host rlib).
- ANTI-VACUITY gating: copy a kernel crate to /tmp scratch, inject
  `extern crate std;` + one HashMap use, rebuild — injected build MUST fail; if the probe
  passes the mutation the gate is vacuous → exit 1.
Targets: probe wasm32-unknown-unknown; bare-metal alternate riscv32imc-unknown-none-elf
(installed); host/test aarch64-apple-darwin (tests keep std via cfg(test)). Receipt per
crate: probe command + exits.

Order: crypto+verification (small) → bont+bn3 (hot paths) → bosym split → gate script +
Makefile + anti-vacuity → receipts. Before editing bont/bn3/verification: ggen sync dry-run
to confirm no touched module is generated output; if it is, edit the template.

## MW3 — registry sweep

On-disk findings: 22 members; publish=false: bench, cgap-beam, cgap-court, cgap-verification,
cli (5) — but cgap-gpu has NO explicit publish line (implicit true — silent default, exactly
the cell class the sweep must make loud); integration + wasm-footprint carry pinned
version = "26.11.1" NOT version.workspace (bypass the workspace-version gate);
release_check.py:21 stale pin; stale DataFusion comment Cargo.toml:33.

Prereqs (blocking): workspace version → 26.11.2; release_check pin → 26.11.2; delete stale
DataFusion comment; convert integration/wasm-footprint to version.workspace = true; explicit
publish = true on cgap-gpu.

scripts/registry_sweep.py (stdlib-only): single source of truth
`cargo metadata --no-deps --format-version 1`; never a hand-listed 22. Per member three
cells, each ∈ {PASS, REGISTERED-DIVERGENCE, MISSING}:
- standing: publishable → crates.io API check (urllib, UA header, one request/crate, no
  retry loop), 200 + max_version == workspace version; non-publish → standing IS the court
  pointer.
- court pointer: manifest annotation or ledger divergent-pair table; publish=false with no
  pointer = MISSING.
- version cell: version.workspace = true or explicit divergence w/ pointer.
Gate: exit 0 iff all cells ∈ {PASS, REGISTERED-DIVERGENCE} AND REGISTERED-DIVERGENCE count
== court-pointer count (no orphan divergences); exit 1 on any MISSING. Final line
`REGISTRY-SWEEP: PASS | FAIL <n missing>`.

Publish sequencing (after structural pass): exactly one cargo publish per publishable crate,
dependency order via cargo metadata topological sort of the member subgraph; 429/Retry-After
honored (min(Retry-After, 60s)) then one retry; second 429 → record, continue, retry at pass
end; already-published-at-version → REGISTERED-DIVERGENCE: already-published (one-attempt
law); hard failure → finish current frontier, record, exit 1.

Anti-vacuity (both in /tmp, sha256 fail-closed): Flip A delete a registry row for one
publishable crate → sweep exit 1 MISSING. Flip B strip publish=false from cli in a sandbox
manifest copy → sweep must demand a crates.io row and exit 1. A sweep with no witnessed flip
= [SKIP] loud (exit 3). Wire-in: pre-step of G1-PUBLISH; G1's "19/22" line becomes
sweep-computed, not prose.

## MW4 — like-for-like mat bench (RDFox envelope)

On-disk state: 01-release-gates.md specs G3-MAT-BENCH + G4-DRED but both are [SKIP]-blocked —
no mat/dred [[bench]] targets exist. 02-bench-protocol.md (now 06-bench-protocol.md per the
W1 numbering; reconcile names at W4 start) absent at planning time. Existing substrate:
crates/cgap-storage/src/dred.rs (retract/compact/oracle_surviving) exists; NO forward-chaining
rule engine anywhere in the workspace — the closure kernel must be built.

Workload (RDFox-published-method-comparable): recursive Datalog — transitive closure
(anc(x,z) :- anc(x,y), anc(y,z). anc(x,y) :- par(x,y).) + same-generation, 2 recursive + 1
base rule; synthetic par(x,y) bucketed at 1M/5M/10M base facts, splitmix64 seeded (dred.rs:34
style). Metric class identical to the citation: derived facts / second / core, single thread,
warm cache, semi-naive (delta-driven) eval.

New kernel crates/stardf-datalog (cgap-side NEW crate; name is in-repo history): semi-naive,
stratified semi-join via the L3 bit-slice machinery, u64 fact IDs, zero interning (§5
prohibition), no_std from birth ([feature = "std"] test-only). Derived-fact counter is the
numerator; wall time the denominator.

Bench benches/mat_closure.rs (+ [[bench]] entry; buckets 1M/5M/10M, facts/s per bucket,
single-core) + benches/dred_churn.rs (G4: reuse dred.rs; 90% churn + entropy sweep 0/50/100%;
ratio fence vs sleep-free baseline).

Honest-comparison framing (receipt boilerplate, verbatim rule): "RDFox was NOT executed.
Baseline = published 2-6.1M facts/s envelope, source: <citation>, machine+method quoted;
cgap measured on <cpu/cores/RAM/rustc/commit>, method: semi-naive, single core." Before first
run every number is [TARGET-v26.11.2]. No cross-unit conversion; "X× RDFox" phrasing only
after both sides of the like-for-like class are stated.

Mutations (G7 flip court, /tmp sandbox, sha256 fail-closed): drop one atom from anc's
recursive body → derived-fact count MUST change (differential oracle vs unmutated fact set)
else exit 3 VACUOUS. Second flip: de-duplicate one join arm → same.

## G5/G6 — zero-DataFusion gate + receipt lint

Authoritative receipt: docs/jira/v26.11.1/waves/cgap-datafusion-removal.md.

scripts/zero_datafusion_gate.py: primary = cargo metadata scan (no datafusion package among
members/deps); secondary = cargo tree --workspace --edges all text scan; tertiary = literal
scan of member Cargo.tomls + Cargo.lock. All three must agree; disagreement = typed refusal.
Exit contract: 0 clean / 1 VIOLATION (print file+dep-path per hit) / 2 GATE-REFUSED (tool
missing, unparsable metadata, zero members, no evidence, exceptions) / 3 VACUOUS (self-check
failure). --self-check: tempfile copy in /tmp, inject dummy `datafusion = "1"` dep, run
production predicate over the copy — MUST flag it (exit 1 on copy) or real gate exits 3;
clean-twin arm stays green. Modes: default / --self-check / --json (machine receipt:
members_count, evidence_sources, hits, verdict). Wire: justfile recipe `zero-datafusion`
folded into just doctor; one line in scripts/verify_v26.10.31.sh (CI mirrors verbatim);
append first-green + self-check row to the removal receipt.

scripts/receipt_lint.py: scope = docs/jira/v26.11.*/waves/*.md + finish/audit receipts.
Claim-sentence heuristic (declarative line containing landed|removed|verified|green|passes|
closed|fixed|deleted|stops|pruned) must carry a validated receipt ref (file existence + line
proximity) or [TARGET-v26.11.2] (regex). Unclaimed → UNRECEIPTED-CLAIM <file>:<line>, exit 1.
Excluded paths skip loudly via explicit dir allowlist. Fail-closed: zero docs / unreadable /
exception → exit 2. --self-check plants a claim-less fixture in /tmp and must fail. Wire:
just doctor after G5; verify_v26.10.31.sh. First-run policy: fix existing unreceipted claims
in place (receipts exist) — no allowlist (allowlist is the next drift channel).
