# Lane DX-1: Doctor Agent-Lane Checks

Standing: ALIVE (all commands run this session, exits witnessed below).

## Files written

- `scripts/doctor_agent_lanes.py` (new, 168 lines, ~10 checks of 4 families)
- `justfile` — appended `doctor-agents:` recipe only; existing 10-check `doctor` recipe untouched (lines 21-35 unchanged, verified by construction: append-only diff).
- `docs/jira/v26.11.2/lanes/lane-dx-01-doctor-agent.md` (this receipt)

CLAUDE.md untouched (MV-8 owns it).

## Commands + exits (this session)

- `python3 scripts/doctor_agent_lanes.py` — exit 1 (informational WARN, per `just doctor` model: 0 GREEN / 1 WARN / 2 RED).
- `just doctor-agents` — exit 1 (recipe exits script's rc; WARN is informational).
- Fence run: `just doctor` — exit 2, PRE-EXISTING failure: `[MISSING] cgap-branchless: no ledger row` from `scripts/registry_sweep.py`, which carries another lane's in-flight edit (zero overlap with DX-1's files; DX-1 did not touch registry_sweep.py, the cli doctor bin, or any existing doctor check).

## The 4 checks

| check | verdict this run | detail |
|---|---|---|
| stale-server | WARN | binary stale by 11 min (src 13:55 > bin 13:44); remediation `cargo build --release -p branchless-lsp` |
| orphan-leases | WARN | 16 lease dir(s), 13967.5 MB, pointer to `scripts/lease_janitor.py` (DX-3 lands it) |
| workspace-count | PASS | cargo metadata members=23 matches CLAUDE.md claim "23-crate root" |
| dirty-tree | WARN | 259 paths dirty, census by top-level dir (crates:114, docs:92, scripts:23, ...) — expected mid-wave |

Exit model matches the existing doctor: 0 GREEN / 1 any-WARN (informational, exit 1) / 2 any-RED.

## Court (non-vacuity): each check poisoned in /tmp sandbox, each flip witnessed

Sandbox `/tmp/dx1-court`, driven via the script's env overrides
(`DX1_LSPMAX_ROOT`, `DX1_TMP_ROOT`, `DX1_CLAUDE_MD`, `fresh import after env set`),
shared tree never mutated by a court. Sandbox removed after (shutil.rmtree exit 0).

1. **stale-server** flip: sandbox src mtime 09:00, fresh binary copy -> PASS;
   `os.utime(bin, (1,1))` (SHIM-COURT age the binary) -> **WARN** ("binary stale by 29893980 min");
   removed binary -> WARN. PASS→WARN flip witnessed.
2. **orphan-leases** flip: empty sandbox tmp root -> PASS; fake lease dir `dx0-court-target`
   (4096-byte file) -> **WARN** ("1 lease dir(s), 0.0 MB: dx0-court-target"). Flip witnessed.
3. CLAUDE.md sandbox copy "23-crate" -> PASS (real cargo metadata, 23 members);
   "99-crate" -> **WARN**; "no claim" -> RED (structural). Flip witnessed.
   (First court attempt ran cargo metadata with cwd=sandbox non-workspace -> RED invalid-JSON —
   court environment artifact; rerun kept cargo on the real workspace, poisoned only the doc.)
   (Also witnessed: `workspace-count` first read members=0 — parser used the wrong JSON key;
   fixed to `workspace_members` and rerun: PASS. Defect found and fixed by the court itself.)
   The sandbox repo for court 4 required a verified-clean baseline (`git status --porcelain`
   empty before the check returned PASS), otherwise the court is vacuous in the other direction.
4. **dirty-tree** flip: fresh sandbox git repo, verified-clean baseline -> PASS ("clean tree");
   dirty f.txt + untracked g.txt -> **WARN** (2 paths). Flip witnessed. First attempt
   (env set after import, stale module constant) is disclosed as a court artifact, not a flip.

## Divergences from dispatch

1. Dispatch placed the branchless-lsp source at cgap `crates/branchless-lsp/src`; it does not
   exist there — the real source root is `/Users/sac/lsp-max/crates/branchless-lsp/src`.
   The check stats that path (and `/Users/sac/lsp-max/target/{release,debug}/branchless-lsp`).
2. Dispatch said lease-grammar pattern `lspa*/dx*`; implemented as `lspa*` and `dx*-target`.
3. Dispatch said "currently 23" for the CLAUDE.md count; grep confirms CLAUDE.md line 3
   "23-crate root" and cargo metadata `workspace_members` = 23 — PASS, not WARN, on the real tree.
4. `cargo metadata` exposes the member list under `workspace_members` (plain `members` is absent),
   so the count reads that key; a first-draft parser using `members` read 0 and was fixed.