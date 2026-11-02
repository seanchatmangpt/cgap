# LANE CL1 — determinism re-lock, FM-PACK-008 (2026-11-02)

## Defect

`make gate0` exit 2 on `[FM-PACK-008]`: ggen.lock pinned
`blake3:e085b8d1...` but pack on disk hashes `blake3:22ba9f10...`
(pre-existing, W1). Baseline reproduction this session:
`make gate0` → ERROR FM-PACK-008, `make: *** [gate0] Error 1`,
exit **2** (19:02:44Z).

## Fix: intentional re-lock (error's own remediation path)

1. `rm ggen.lock && ggen sync run` → exit **0**; new lock
   sha256 `a1356da108b1dab17cd79c7db2b9659a2143294c701be2b563353a7f4fa95f36`
   (lock pins `blake3:22ba9f10...` — matches pack on disk).
2. Determinism witness: second `ggen sync run` → exit **0**;
   `cmp` of run1 vs run2 ggen.lock → **byte-identical PASS**.
3. `make gate0` (post-fix) → exit **0** (byte-stable replay cmp PASS +
   freestanding C header compile PASS; all 12 projections
   "skipped: unchanged: content identical"; `written: []`).
4. `CARGO_TARGET_DIR=/tmp/cl1-target cargo check -p cgap` → exit **0**.

## Hashes

| | hash |
|---|---|
| ggen.lock before (on-disk, pinned e085b8d1) | sha256 `82cc26e1742f92709c519c0c4d224405cf1a6683cefda8f0fa3b0da3a01e0c11` |
| ggen.lock after (pins 22ba9f10) | sha256 `a1356da108b1dab17cd79c7db2b9659a2143294c701be2b563353a7f4fa95f36` |
| pack content_hash delta | `e085b8d1b9dd880b0d11b20b2a0e18161c18f8c9bcec35b2a46eb24240151a6b` → `22ba9f10057187b369b4b3637a1e2e87a0b652681aedf028de1c36ebd1f07761` |

## ggen.lock git state

Pre-wave: **tracked and modified** (`git ls-files` matches;
`git status --porcelain` = ` M ggen.lock`). Regenerated lock left
uncommitted per lane contract (coordinator owns git transitions).

## Standing

ALIVE — all commands + exits run this session against the exact
subject (repo-root ggen project, ggen 26.10.11).
