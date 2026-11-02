# AU10 — Enterprise Silo Fragmentation (Phase 2)

Five silos, six real boundaries, no formal version contracts on any shared artifact.
All boundaries are intra-repo today (coordination drag scales with boundary count,
docs/audit/models/09_brooks_law_and_coordination_drag.md).

## Silo map (verified paths)
- FIRMWARE: cgap-gate, cgap-blake3, cgap-crypto, cgap-p28, cgap-ffi (no_std confirmed in
  cgap-blake3/Cargo.toml:5, cgap-crypto/Cargo.toml:7)
- STORAGE: cgap-parquet, cgap-arrow, cgap-git (+ cgap-storage: closure.rs, dred.rs, atomic_commit.rs)
- QUERY: cgap-bn3, cgap-storage, cgap-bont, cgap-bosym
- TOOLING: cli, ontology/, scripts/ggen, ggen.toml + ggen.lock, include/cgap_c_data.h
- COMPLIANCE: 35 gate scripts in scripts/ (gate3_cdata.py, gate3_schema.py, receipt_lint.py, ...) + cgap-verification + receipt fleet

## Boundaries: shared artifact / friction class / evidence / contract

### B1 firmware <-> storage: FNV-1a constants + format codes
(1) REAL artifact: FNV-1a is re-implemented, not imported, in 4+ places: crates/cgap-parquet/src/footer.rs:282 `fnv1a64` (basis/prime footer.rs:31);
crates/cgap-arrow/src/ipc.rs:159 `fnv1a32` (different width); crates/cgap-ffi/src/sovereign.rs:864
private `fn fnv1a`; scripts/generate_enterprise_cgap.py (Python copy). Plus
include/cgap_format_codes.h mirrored by gate3_schema.py.
(2) Friction: silent constant/width drift — checksums are cross-silo identities, so a constant
bump on one side breaks every other side's verification with no compile-time signal.
Class: cross-silo identity drift.
(3) EVIDENCE-INTRA: 10+ footer.rs torn-write SLA incidents (footer.rs:836-837 diagnostic path
already separates body_digest vs stored-digest failures) — each incident re-derived the digest
contract by hand instead of consulting a versioned constant contract; incident count scales
with the number of independent constant copies.
(4) ASSUMPTION: semver contract on (basis, prime, width, seeded-derivation) via a
`cgap-digest-contracts` crate, 1.x, breaking = any constant change; no_std + host consumers
import it; Python/FFI mirrors pinned by gate3_cdata.py check.

### B2 storage <-> query: closure/dred algorithms
(1) REAL artifact: cgap-storage/src/closure.rs + dred.rs (README: 1,336-LOC substrate, dred.rs
298) consumed by query-side cgap-bn3 — unify.rs:1849 rebuilds "full-closure column views" as a
naive fixpoint oracle, i.e. bn3 carries a SECOND closure implementation. Closure laws are
parameter-sensitive: unify.rs:213 documents VAR_REGS 128 -> 127 flipping a band-partition law.
(2) Friction: production-vs-oracle algorithmic drift; a correctness property, not just compat.
Class: dual-implementation semantic drift.
(3) EVIDENCE-INTRA: MB3's closure_ladder arena panic found mid-bench
(crates/bench/benches/chains-bn3.rs + crates/cgap-bn3/src/arena.rs) — surfaced in a bench, not
a gate, because no boundary contract forced oracle/production arena agreement pre-bench.
(4) ASSUMPTION: semver on closure/dred semantics (law-vector invariants + arena preconditions),
0.x while laws still flip, 1.0 only when VAR_REGS-class parameters freeze; oracle agreement
checked by a gate, not a bench.

### B3 tooling <-> all: ggen projections + cgap_c_data.h
(1) REAL artifact: ggen.toml + ggen.lock (regenerate CLI + bindings) and include/cgap_c_data.h
(62 lines, C ABI surface consumed by FFI and gate3_cdata.py).
(2) Friction: generated-code staleness — silos hand-edit projected/header content and nothing
fails until a downstream silo regenerates. Class: generated-artifact staleness / one-way breakage.
(3) EVIDENCE-INTRA: AU1's no_std registry divergence — firmware's registry
(crates/cgap-gate/src/lib.rs, penstock.rs) diverged from what tooling projections assumed,
found downstream. Plus E1/SPDX tracking_alloc scan collision
(crates/cgap-core/tests/source_scan_gate.rs:155,409-418: `#[path]`-included tracking_alloc.rs
confuses the scanner's alloc::-vs-no_std classification) — tooling-owned gates mis-scanning
firmware-owned sources is this boundary failing.
(4) ASSUMPTION: semver the header (cgap_c_data.h = ABI contract, 1.x) and ggen.lock (projection
pin, exact-pin; bump = regenerate-all gate); source_scan_gate.rs exemptions become contract
clauses, not ad-hoc path allowlists.

### B4 compliance <-> all: gates own the contracts
(1) REAL artifact: the 35 scripts/ gates + cgap-verification + receipt fleet ARE the de-facto
interface contracts for B1-B3 (gate3_cdata.py pins the header, gate3_schema.py pins format
codes, receipt_lint pins receipts).
(2) Friction: compliance is both contract owner and enforcer with no declared contract version —
each B1-B3 contract above is currently implicit inside gate scripts, so contract change =
gate-script edit and drift is caught post-hoc. Class: enforcer-coupled contract opacity.
(3) EVIDENCE-INTRA: the E1/SPDX collision again — the gate misclassified tracking_alloc.rs
(alloc::-vs-no_std) and the fix was negotiated inline between compliance (gate test) and
firmware (crate source) with no contract artifact to negotiate against; same class as AU1's
no_std registry divergence (gate expectation vs silo reality resolved by incident, not version).
(4) ASSUMPTION: each gate declares which contract+version it enforces (scripts/contracts.toml,
machine-readable), so a silo sees "this gate pins contract X@2" before editing the shared
artifact; receipt fleet version-stamps receipts.

## Scaling summary
6 boundaries; 4 intra-team incident families (footer torn-write x10+, SPDX scan collision,
no_std registry divergence, closure_ladder arena panic) share one root: no version contract on
any boundary artifact. Incident rate tracks independent-copy count (B1: 4+; B2: 2; B3: pin+header).
