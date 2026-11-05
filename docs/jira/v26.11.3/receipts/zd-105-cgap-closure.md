# ZD-105 — cgap v26.11.3 closure seal receipt
Standing: PARTIAL_ALIVE

Date: 2026-11-04. Lane: ZD-105 (seal). Repo: `/Users/sac/cgap`
main @ `b6d685177d6d62b95708b75627fc1b617265a82d` (+ this commit),
pushed after commit.

## Diff (owned files only)

- `docs/jira/v26.11.3/closure/cgap-closure.md` (new; `closure/` dir
  created by this lane)
- `docs/jira/v26.11.3/receipts/zd-105-cgap-closure.md` (new)

## Deliverables rowed

All standing re-read from receipt bodies on disk this session, never
memory.

- AoQ kernel cgap-aoq ALIVE (AF-37 p99 torture ×4, RX-2 conformance
  exit 0 ×4 + lib courts 14/0, AF-1 corridor join 3/0)
- Crypto kernels: AB-2 FIPS-202 ALIVE, AB-3 ChaCha ALIVE, AB-4 AEAD
  PARTIAL_ALIVE, AB-5 ML-KEM-768 ALIVE (liboqs interop); dod-006
  closed quiet 3× GREEN, 261.26 MB/s min (zd-8)
- Wave-2 membrane LANDED (zd-63 flip over zd-6/zd-13 + arcus
  zd-29/zd-42); rung 8 E2E zd-56; rung 7 gate0 closed by commit
  `f30580ff` after the zd-79 BLOCKED witness
- DoD 12/12 (zd-36; zd-55 disclosed `--skip-ea-admission` ruling)
- Dry-run ladder 20/20 accounted (zd-43 7-green + 13 registry-dep;
  zd-97 retry +2, 0 unclassified)
- XIP cfg-stub pattern + xip_t1_riscv_elf_size_court (zd-56)
- Corridor real-KEM (AB-5 + AF-1)
- Post-wave rustfmt convergence at HEAD b6d68517

## Gates run this session (seal's own actuation, real exits)

| command | exit |
|---|---|
| `cargo check --workspace` | 0 |
| `make gate0` | 0 |
| `git rev-parse HEAD origin/main` (post-fetch) | equal (b6d68517) |

`cargo check` surfaced 13 pre-existing cgap-crypto lib warnings —
disclosed, pre-existing, no errors.

## Standing

ALIVE for the seal's contracted scope: both files land, every claim
resolves on disk, both quick gates re-run green at the sealed subject,
commit + push executed with pathspec-scoped commits (`git commit -F`,
retry ×3 bounded). BLOCKED/DEFERRED carries (crates.io Stage 3, tags,
cgap-mcp standalone workspace) are recorded in the closure, not claimed
closed.

## Falsifiers

- Any rowed standing word disagrees with its receipt body on disk.
- `cargo check --workspace` or `make gate0` re-run at b6d68517 exits
  non-zero.
- Receipt count (78 pre-seal / 80 post-seal) disagrees with a fresh
  `ls | wc -l`.
- Push fails or origin/main ≠ the sealed SHA.
