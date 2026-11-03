# pub-03-publish-sequence

PUB-03 post-freeze-lift publish sequence (2026-11-02, read-only
planner). Method: fresh read of every publish=true manifest
(dep graph from [dependencies] only; dev-deps excluded).
Grounded in waves/publish-halt.md + w14-lift-register.md +
audit/pub-08-version-unification.md.

## 0. Version law

Every step publishes the 26.11.3 line per PUB-08 §4: one atomic
workspace bump to 26.11.3 + pin rewrite + cargo update -w lock
regen MUST land before Step 1.

## 1. Laws (w14-lift-register)

ONE ATTEMPT PER CRATE per window; 1-HOUR BACK-OFF ON SLIDE
(sequence stops, next attempt >= 1h from slide timestamp);
tier gate: step actuates only after prior step's crates.io
witness poll (UA header) exits 0. Falsifier: any step lacking
a named pre-court receipt is REFUSED.

## 2. NEW DEFECTS found building this sequence (publish blockers)

- G-1: cgap-gate has a RUNTIME path dep on cgap-branchless
  (publish=false) — cargo publish refuses runtime path deps on
  unpublished crates. Gate's first publish is gated on branchless
  gaining publish=true or the edge becoming a registry dep.
- U-1: cgap umbrella carries cgap-branchless {path, version
  26.11.2} — same refusal class. Both belong in the PUB-08
  atomic commit edit list.
- I-1: cgap-integration pins cgap-audit (path+version dev-dep);
  cgap-audit is NOT on crates.io (404 witnessed 2026-11-01) —
  audit 26.11.3 must land before integration, or the pin becomes
  registry-style.

## 3. The sequence

### 3.1 Lead — cgap-gate 26.11.3
Gated on G-1 resolution. Pre-courts: a14-finish-receipt.md +
pub-w3-witness.md (gate 26.11.1->26.11.2 LANDED, exactly-once).

### 3.2 Republish-owed set (leaf-first)
1. cgap-blake3 26.11.3 (D1 NEON; pre-court p1-blake3-optlevel-ub
   -fix.md, 101 flips) — zero-dep leaf.
2. cgap-arrow 26.11.3 (D2 stale generated_version; pre-court
   p4-terminus:40) — zero-dep leaf.
3. cgap-parquet 26.11.3 (RingSink; prereq arrow 26.11.3;
   pre-court a3-ring-sink.md + w5-ring-rewitness.md).
4. cgap-bn3 26.11.3 (no_std flip in tree, tarball predates it;
   prereq blake3 26.11.3; pre-court w6-bn3-nostd-borrowed-driver).
5. cgap-audit 26.11.3 (first publish; prereq arrow+parquet;
   pre-court publish-halt READY + a15-finish-receipt).
6. cgap-integration 26.11.3 (FIRST publish; prereqs arrow/blake3/
   parquet/git/audit live per I-1; pre-court publish-halt READY
   matrix + PUB-W5 baseline).
Falsifier for the set: crates.io max_version == 26.11.3 and
num_versions exactly +1 per crate (UA-header witness poll).

### 3.3 Default-publishable members (leaf-first, 15 steps)

bosym -> git -> bont (blake3) -> crypto (blake3) -> core
(blake3+gate) -> p28 (blake3+crypto) -> storage (blake3+git) ->
gpu (arrow) -> audit (arrow+parquet) -> integration (audit, I-1)
-> ffi (parquet D8 + arrow/blake3/core/gate/bn3/p28) -> cgap
umbrella (all; U-1 resolved) -> wasm-footprint (arrow+parquet
registry pins; D9 host-link gate) -> fibo (blake3+bont/core) ->
cloud (blake3+bont).

Each step's pre-court cited in the receipt: PUB-W2/W3/W5
witnesses + a04/a05/a06/a15/a17/a19 finish receipts +
publish-halt READY matrix rows.

wasm-footprint stays LAST: defect D9 (host-link failure in
tarball verify) must close first, else it exits the window
FAILED under the one-attempt law.

## 4. Sequence-level falsifier

16 steps, 16 named receipts — none REFUSED. Post-wave: every
publish=true crate at 26.11.3, num_versions +1 exactly.
Out of scope: csct 0.1.0, publish=false set (bench, court, beam,
cli, verification, branchless unless G-1/U-1 resolve).

## See Also

audit/pub-08-version-unification.md · waves/publish-halt.md ·
waves/w14-lift-register.md
