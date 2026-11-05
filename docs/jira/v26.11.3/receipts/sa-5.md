# SA-5: cgap-pack sync-time validation gates

Repo: /Users/sac/cgap (consumer) · /Users/sac/ggen-marketplace/packs/cgap-pack (pack, lane-owned) · ggen 26.10.11 · 2026-11-04 · No git commands run by this lane (inventory used a read-only `git status` once, directed by the coordinator restart).

## Question

Does cgap-pack — the pack behind `capabilityRef <urn:ggen:pack:cgap-pack>` in arcus's ABB chain — carry fail-closed sync-time validation gates modeled on arcus-pack's 8-gate corpus (ar-1)?

## Restart inventory (dead-lane law)

The lane was killed by a 429 mid-exploration and restarted. Inventory first:
cgap-pack already carried prior-lane state — version 26.10.29→26.11.2,
ontology +594 lines (import-primitives section), untracked gates 010-160,
`[gates]` table, pass/fail witnesses for 010-160, `gate-court.toml`. Missing
per contract: `[validation]` block, layout-closure law, status-closure law,
`# MESSAGE:` header convention on 140/150. I verified the existing corpus
rather than re-authoring it.

## Landed

Pack (all under /Users/sac/ggen-marketplace/packs/cgap-pack/):

1. `pack.toml` — replaced the ad-hoc `[gates]` table with the arcus-shape
   `[validation] gates = [...]` block listing all 19 gates.
2. `ontology.ttl` — appended a C-ABI status-code family section:
   `p:StatusConstant` class + `p:statusVariant`/`p:statusCode` properties +
   7 individuals (OK=0, InvalidFormat=1, FormatMismatch=2, MissingBuffer=3,
   BufferTooShort=4, Misaligned=5, ForeignError's NegativeLength=6), grounded
   in `crates/cgap-arrow/src/foreign.rs` + the generated `status` mod
   (ground truth re-read at use time).
   Pack-ontology sha256 `fb8e82ceb98e89dd194f20bbb1ac65ffc757b248b20ff18be4f1af98238453a7`.
3. `gates/170_layout_closure.rq` — every p:ArrayKind (the ArrayKindSpec
   surface) must carry bufferCount > 0 and a power-of-2 kindAlignment
   (closed 1..65536 enumeration; engine has no bitwise ops); missing facts
   are violations. `# MESSAGE:` header.
4. `gates/180_status_closure.rq` — TX-7/CG-1 status-closure law as SPARQL,
   three branches: (a) no two StatusConstants share a statusCode (CG-1);
   (b) every p:fnErrorMapping value matches the closed ForeignError
   vocabulary regex (TX-7 referential closure); (c) every error variant is
   named by ≥1 ImportFn mapping (coverage). `# MESSAGE:` header.
5. `gates/140_bench_file_exists.rq`, `150_receipt_sha_format.rq` — first
   lines normalized to the `# MESSAGE:` house convention (header text only;
   query bodies untouched).
6. `witnesses/pass|fail/170_layout_closure.ttl`, `.../180_status_closure.ttl`
   — witness graphs in the existing house shape.

Consumer: `/Users/sac/cgap/ggen.lock` regenerated twice by real `ggen sync run`
(disclosed re-lock; never hand-edited). Final
`content_hash = blake3:3a0c2e2dad360093b8c27a95d8c13d87b97f1c2aa88e3c3e89a43c0864973b72`.

## Verification ladder (all real runs)

- **Clean positive path**: `ggen sync run` (cwd /Users/sac/cgap) → exit 0,
  19 gates evaluated (all in the JSON closure), `written []`, 13 files
  `skipped: unchanged: content identical` — **zero generated-artifact drift**,
  so no cgap-arrow re-pin was needed and the arrow suite is regression
  evidence, not repair. Dry-run with lock → exit 0.
- **Shim courts (FM-PACK-013 flip + byte-exact restore, AR-1 pattern)**:
  each shim = violating triple injected into pack ontology, `ggen.lock` held
  aside (FM-PACK-008 preempts gate evaluation otherwise — witnessed), dry-run,
  restore, sha256 equality against baseline
  `fb8e82ce...`:
  - 170: `kind-dense-union` alignment 64→48 → `[FM-PACK-013] pack 'cgap-pack' gate '170_layout_closure.rq' refused`, exit 1. RESTORE_BYTE_EXACT_OK.
  - 180 branch (a): second `p:statusCode 1` on status-ok → refusal, first row `duplicate statusCode 1`, exit 1. RESTORE_OK.
  - 180 branch (b): `fnErrorMapping "InvalidFormat,NotARealVariant"` → refusal, first row `fnErrorMapping token outside ForeignError vocabulary: InvalidFormat,NotARealVariant`, exit 1. RESTORE_OK.
  - 180 branch (c): dropped `NegativeLength` from the array fn → refusal, first row `status constant with no ImportFn mapping fact: NegativeLength`, exit 1. Restore initially failed (bad sed anchor) — repaired on the spot, RESTORE_BYTE_EXACT_OK.
- **Witness non-vacuity courts** (witness graph appended to pack ontology,
  lock aside): 170 pass → exit 0; 170 fail → FM-PACK-013 naming the gate;
  180 pass → exit 0; 180 fail → refusal firing all three branches. (First
  170-pass witness lacked the arrow_layout fact set and tripped the sibling
  gate — fixed by completing the facts; a pass witness that trips a different
  gate is not a pass.) All restores byte-exact.
- **`cargo test -p cgap-arrow`** → exit 0, 12 test binaries, 228 tests,
  0 failures (145+36+17+7+6+5+4+3+3+1+1+0).

## Engine findings (session-introduced knowledge, not defects)

- FM-GRAPH-013: a BIND target projected in SELECT is treated as in-scope
  ("re-binds in-scope variable") — write gates that project only pattern vars.
- No bitwise `&` operator in this SPARQL engine (parses as ENCODE_FOR_URI
  expectation) — pow2 test is a closed IN-enumeration.

## Standing

- Gates 170/180 + `[validation]` block: ALIVE — executed at sync time,
  per-gate non-vacuity witnessed by flipping courts, positive-path PASS with
  exit-0 sync receipt.
- Pre-existing gap (prior-lane state, not session-introduced, left as-is):
  `gate-court.toml` names `runner = python3 qualification/verify.py` but
  `qualification/verify.py` does not exist; the witness dirs have no runner.
  Also, the witness-court mechanism (append-witness graph) is documented here
  in lieu of that runner.
- `cgap/packs/` does not exist; the pack lives at
  /Users/sac/ggen-marketplace/packs/cgap-pack (contract's first guess was wrong).
- No BLOCKED/UNSUPPORTED/REFUSED residues from this lane. No git run: the
  untracked prior-lane state in packs/cgap-pack plus my additions awaits the
  coordinator's git transitions.

result: UNSUPPORTED
