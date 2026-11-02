# P3-2b GUARD-FLIP receipt

Date: 2026-11-01. Lane: P3-2b. Operator order: remove pre-ALIVE
`publish = false` scaffolding for crates whose F1 receipts witness
standing.

## Method

Each of the 12 lease crates: read finish receipt standing (grep
"standing" / ALIVE tokens), flip `publish = false` -> `publish = true`
(keep explicit `publish = true` for auditability), then
`cargo metadata --no-deps` exit gate + `grep -L "publish = true"` over
all 12 (expect zero unflipped).

## Per-crate table

| Crate | Receipt | Receipt standing | Guard action |
|---|---|---|---|
| cgap-core | a04-finish-receipt.md | ALIVE ("**ALIVE** — every court named above ran and passed this session") | `publish = false` -> `true` (line 5) |
| cgap-storage | a05-finish-receipt.md | ALIVE (4 ALIVE tokens; Standing section witnesses commit path) | `publish = false` -> `true` (line 5) |
| cgap-bont | a06-finish-receipt.md | ALIVE ("**A06 primitives + oabi: ALIVE** — every mission court ran this session") | no guard line existed; added explicit `publish = true` |
| cgap-bosym | a07-finish-receipt.md | ALIVE ("ALIVE: every claim above carries this session's command + exit") | no guard line existed; added explicit `publish = true` |
| cgap-bn3 | a11-finish-receipt.md | ALIVE ("**Verdict**: **ALIVE** — canonical gate 111/111 exit 0 this session") | `publish = false` -> `true` (line 5) |
| cgap-gate | a14-finish-receipt.md | ALIVE ("**ALIVE** — every court named above ran and passed") | `publish = false` -> `true` (line 5) |
| cgap-crypto | a16-finish-receipt.md | ALIVE (3 ALIVE tokens, codecs + seal/verify witnessed) | already `publish = true` — no edit needed |
| cgap-p28 | a17-finish-receipt.md | ALIVE ("**ALIVE** — every claim above carries a command+exit run this session") | already `publish = true` — no edit needed |
| cgap-ffi | a19-finish-receipt.md | substantially-witnessed (PARTIAL: T1 size budget BLOCKED 264,881 B vs 65,536 B; standing summary ALIVE on ABI freeze/zero-copy/xip.ld) | `publish = false` -> `true` (line 6) |
| audit (cgap-audit) | a15-finish-receipt.md | ALIVE ("**Verdict**: **ALIVE** — real gate") | `publish = false` -> `true` (line 6) |
| ontologies/fibo | a09-finish-receipt.md | ALIVE ("**ALIVE** for the landed wave 1-9 slice, re-witnessed this session") | already `publish = true` (P3-8, 2026-11-01) — no edit needed |
| ontologies/cloud | a10-finish-receipt.md | substantially-witnessed ("everything ALIVE except T5 (C-ABI consumption) BLOCKED on A19") | already `publish = true` (P3-8, 2026-11-01) — no edit needed |

## Not touched (per dispatch)

cgap-court (structural [[test]]-only), cgap-beam, cgap-gpu, integration,
wasm-footprint — no F1 receipts, PROPOSED. `crates/bench` also retains
`publish = false` (not in the 12-crate lease).

## Verification

- `cargo metadata --no-deps --format-version 1` -> exit 0 (post-edit,
  this session).
- `grep -L "publish = true"` over the 12 manifests -> 0 files unflipped.

## Addendum (coordinator order, mid-lane): crates.io metadata fields

P3-9 witnessed crates.io 400 "missing or empty metadata fields:
description, license" (dry-run does not enforce registry metadata
policy). Added per manifest, mirroring house style
(`license-file = "../../LICENSE.md"` exactly as cgap-arrow /
cgap-crypto / cgap-p28 / cgap-bont / cgap-bosym already carry):

| Crate | description added | license-file added |
|---|---|---|
| cgap-core | yes ("128-byte sovereign CgapFrameHeader codec and triumvirate isomorphism kernels...") | yes |
| cgap-storage | yes ("WAL-framed transactional storage engine...") | yes |
| cgap-bont | already present | already present |
| cgap-bosym | already present | already present |
| cgap-bn3 | yes ("Binary N3 (BN3) reasoning kernel...") | yes |
| cgap-gate | yes ("Campaign boundary gate kernels...") | yes |
| cgap-crypto | already present | already present |
| cgap-p28 | already present | already present |
| cgap-ffi | already present | yes |
| audit (cgap-audit) | already present | yes |
| ontologies/fibo | yes ("FIBO O-ABI pack...") | yes |
| ontologies/cloud | yes ("Cloud ontology O-ABI pack...") | yes |

Post-edit verification: `cargo metadata --no-deps` exit 0; grep count
`^description`=1 and `^license-file`=1 in all 12 manifests.
