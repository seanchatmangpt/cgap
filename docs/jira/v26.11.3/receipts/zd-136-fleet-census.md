# ZD-136: Fleet Census (ZP-8 Fill-In)
Standing: ALIVE

**As-of**: 2026-11-04T16:43:02Z (live git/gh reads; no state changes)
**Purpose**: ZP-8 fill-in for the wave-closing receipt.

## Per-repo census

| repo | HEAD | branch | origin parity | receipts (v26.11.3) | tree | tags *26.11.3* |
|---|---|---|---|---|---|---|
| ggen | `14c6612bfecde172c854536f2f687f3bbd71e03f` | v26-11-3-wave | IN-SYNC (ls-remote) | 149 | clean (0) | none |
| cgap | `09e8c7c7eaffb22a2d1fc91c469d7015109fb51c` | main | IN-SYNC (ls-remote) | 84 (+this file, untracked) | dirty (3)¹ | none |
| arcus | `e82b4ad7ca0c17da30dc24e4385bcbfc222becdd` | main | LOCAL-ONLY (no origin remote) | 32 | clean (0) | none |
| lsp-max | `e41e957596c4aa85cb5e72e3cd4fc4157f408f7c` | master | IN-SYNC (ls-remote) | 17 | dirty (1)² | none |
| claude-code-config-lsp | `af67d0e1e5e9d366b4cba9b003c45f3b15b3c9c6` | main | IN-SYNC (ls-remote) | 71 | clean (0) | none |
| ggen-marketplace | `ee4c589be6aa7ce0cd40e105a34db53605f60e9c` | main | IN-SYNC (ls- feeds CARRY-19) | 13 | clean (0) | none |

¹ cgap dirty set: `M docs/jira/v26.11.3/README.md`, untracked `receipts/zd-113-ladder-refresh.md`, `receipts/zd-116-index-final.md` (+ this file after write).
² lsp-max dirty file: `examples/wasm4pm-lsp/tests/receipts/cog010-scan.json`.

## Cross-cutting

- **Total receipts across fleet**: 149 + 84 + 32 + 17 + 71 + 13 = **366**
  (365 pre-census on-disk; +1 = this ZD-136 receipt, currently untracked in cgap).
- **Total closed CARRY items** (`~/ggen/docs/jira/v26.11.4/SEED.md` §A closed set,
  re-verified 2026-11-04): **8** — CARRY-01, 02, 04, 05, 07, 08, 09, 12.
  (CARRY-13/14 UNCHANGED parked; 03/06/10/11 remain open; 15/19/17/16/18 are the
  wave-closing sequence, not closed.)
- **PR #824**: state=**OPEN**, headRefOid=`14c6612bfecde172c854536f2f687f3bbd71e03f`
  (matches ggen local HEAD → CARRY-15 green-or-waived rollup subject is current).

## Commands (replay)

```
for r in ggen cgap arcus lsp-max claude-code-config-lsp ggen-marketplace:
  git -C ~/<r> rev-parse HEAD
  git -C ~/<r> status --porcelain | wc -l
  ls ~/<r>/docs/jira/v26.11.3/receipts | wc -l
  git -C ~/<r> tag -l '*26.11.3*'
  git ls-remote <origin-url> refs/heads/<branch>   # arcus: no origin → LOCAL-ONLY
gh pr view 824 --json state,headRefOid
```

## Standing

ALIVE (all figures read live at the as-of timestamp; zero git state mutations; sole
write = this receipt file).
