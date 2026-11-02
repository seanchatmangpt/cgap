# Lane DX-2 — write-integrity PostToolUse hook

Standing: ALIVE (all commands + exits run this session, 2026-11-02)

## Files written

- `.claude/hooks/write-integrity.sh` (NEW, 90 lines, `bash -n` clean)
- `.claude/settings.json` (one PostToolUse `Edit|Write` entry inserted after
  `fmt-edited`; `branchless-court` has no settings entry to anchor after)
- `docs/jira/v26.11.2/lanes/lane-dx-02-write-integrity.md` (this receipt)

## Behavior (as dispatched)

Scope gate: code extensions only (`.rs .py .sh .toml .c/.h/.cc/.cpp/.hpp
.js .ts`), AND (>100 lines OR under `crates/`/`scripts/`); `.md/.txt/.rst/
.adoc`, `docs/`, `ontologies/` excluded unconditionally — docs prose never
scanned. Checks: (a) NUL/control bytes (excl `\n\t`, incl DEL) via python
byte sweep; (b) brace/paren balance DELTA vs `git show HEAD:$REL` (tracked
only; untracked/git-blocked → typed stderr skip, exit 0); (c) stray
tokens `PLACEHOLDER|FIXME-PLACEHOLDER`, U+FFFD runs, latin1-desync
double-encoding (C3 83 C2 / C3 A2 E2 82 AC / C3 82 — "Ã©"/"â€" class).
Clean → exit 0 silent; suspicious → exit 2 + finding + re-verify command.

## Court transcript (this session, fixed hook)

| court | subject | rc | verdict |
|---|---|---|---|
| C_nul | /tmp 120-line .rs + trailing NUL | 2 | FIRES (CONTROL_BYTES) |
| C_ph | /tmp fixture + `// PLACEHOLDER` | 2 | FIRES (STRAY_TOKENS) |
| C_moji | /tmp fixture + `caf\xc3\x83\xc2\xa9` desync | 2 | FIRES (mojibake) |
| C_legit | legit UTF-8 `caf\xc3\xa9` (single-encoded) | 0 | silent (no false positive on real 2-byte UTF-8) |
| C_fixture | clean untracked 120-line .rs | 0 | silent + typed skip (balance: untracked) |
| C_BALANCE_FIRE | /tmp git repo, tracked f.rs + stray `}` vs HEAD | 2 | FIRES (BRACE_PAREN_DELTA -1 vs 0) |
| C_BALANCE_CLEAN | same f.rs restored byte-identical to HEAD | 0 | silent |
| C3_MD | 151-line prose .md with braces/parens | 0 | silent (docs scope-out) |
| C_BIG_TRACKED | crates/cgap-bn3/src/unify.rs (2946 lines) | 0 | silent, 270 ms (< 1 s budget) |

Anti-vacuity: every firing court has a paired negative (legit/clean
variant stays silent); balance court flips fire->clean by byte-restoring
the fixture. Sandbox courts ran on /tmp copies and a /tmp git mirror
(`git -C` prefix swapped via sed) — shared tree never mutated.

## Defects found and fixed during court development

1. `git show HEAD:$FP` with absolute $FP always fails → balance check
   silently no-opped (C_BALANCE_FIRE was rc=0 = vacuous). Fixed: resolve
   repo-relative path via `git ls-files --full-name -- "$FP"`.
2. `rb"[...[\x80-\xbf]]"` char-class regex did not match as written
   (python 3.14 raw-bytes escape interaction) → replaced with plain
   byte-sequence `in` checks (also kills the C3-A9 false-positive class:
   legit single-encoded é is C3 A9, never matched).

## Gate: flip-tests.sh after wiring

`CLAUDE_PROJECT_DIR=/Users/sac/cgap bash .claude/hooks/flip-tests.sh`
→ rc=0, 9/9 PASS (publish-guard fire/quiet + sandbox-mutation,
cwd-guard x3, mtime-restore fire/quiet, rep1-census-gate passed=3).
`python3 json.load(settings.json)` → valid.

## Fences

- `.claude/settings.json` is shared: lane added exactly one PostToolUse
  entry; JSON validity verified post-edit.
- flip-tests.sh is the shared hook gate — run post-wiring, 9/9, rc=0
  (no other-lane transient encountered; no retry needed).
- No git write commands; no lib.rs/Cargo.toml touched.

## Divergences

- Dispatch said insert after "fmt-edited and branchless-court if
  present": branchless-court.sh exists on disk but has NO settings.json
  entry, so insertion is directly after fmt-edited.
- Balance check semantics: delta computed vs `HEAD:path` (as dispatched),
  not vs the immediately-pre-edit working-tree state — concurrent-lane
  edits that themselves change balance will surface as findings on later
  edits of the same file.
