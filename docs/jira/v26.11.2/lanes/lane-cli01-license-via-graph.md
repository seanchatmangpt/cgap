# Lane CLI-01 — cli license-file via the lawful graph path

Date: 2026-11-02. Repo: /Users/sac/cgap (main). No git write commands run.

## W1 Subject

`crates/cli/manifest_patch.py` (additive patch step 1.5) and the
regenerated `crates/cli/Cargo.toml` (lawful ggen regen, not a hand edit).

## W2 Context

DOD-01FIX's direct edit to `crates/cli/Cargo.toml` was REFUSED by the
guard-generated.sh PreToolUse hook: cli's Cargo.toml is a ggen-generated
surface (`/Users/sac/cgap/.claude/rules/cli-generated.md`). The lawful
channel is the post-sync consumer seam `manifest_patch.py`.

## W3 Diff

`manifest_patch.py` gained step 1.5 (inserted between step 1 and step 2,
script's own idempotent style: guarded on `"license-file" not in s`,
anchored on `publish = false`, asserted present):

```python
s = s.replace(
    "publish = false",
    'publish = false\nlicense-file = "../../LICENSE.md"',
)
```

## DIVERGENCE (dispatch delta, recorded)

Dispatch said `../LICENSE.md` ("crates/cli → ../LICENSE.md"). That is
WRONG for this depth: `crates/cli/..` = `crates/`, and
`crates/LICENSE.md` is absent — the identical defect DOD-01 F3c already
fixed in crates/csct (`crates/csct/Cargo.toml:7`). House style for
crates/* is `../../LICENSE.md` (cgap-core:10, cgap-p28:11, cgap-ffi:8).
Landed `license-file = "../../LICENSE.md"` (resolves to
/Users/sac/cgap/LICENSE.md, verified present).

## W4 Commands (this session, exits)

- `cd crates/cli && rm Cargo.toml && ggen sync run && python3
  manifest_patch.py` — sync rc=0, patch rc=0 ("patched"); flip witness:
  `grep -c license-file Cargo.toml` = 1 (line 10).
- Byte-identical replay (gate0 discipline): regen run twice,
  `cmp /tmp/cli01-pre2.toml Cargo.toml` → REPLAY BYTE-IDENTICAL.
  `git diff --stat` on the other ggen-written files
  (generated_cli.rs, zero_code_cli.rs, command-reference.md, ggen.lock)
  = empty, i.e. byte-identical to HEAD.
- Fence `cargo build` (crates/cli) — exit 0 (Finished dev profile,
  16.10s).
- Fence `cargo metadata --no-deps` (root) via run_checked, rc=0:
  23 workspace members, cli NOT a member — unchanged.
- Per-crate census (fence 5): fleet-ops/ocel2/csct each carry
  `license-file = "../../LICENSE.md"` landed by DOD-01FIX, and each has
  NO `.clap-noun-verb` dir → NOT ggen-generated, direct edits were
  lawful there. Only crates/cli is ggen-generated.

## W5 Fences / anti-vacuity

Must-not-regress: make gate0 byte-identical replay (covered above, cmp
passed); guard-generated.sh refusal path (this lane is the counter-
example that proves the hook: the DOD-01FIX direct edit was REFUSED,
this receipt shows the lawful channel producing the same result).
Anti-vacuity mutation implicit in the flip witness: the patch is
guarded on `"license-file" not in s`; running it twice prints
"patched" then would print "already patched" on an unchanged manifest
(replay cmp proves the second pass is a fixed point).

## W6 Standing

ALIVE. Every claim above re-read from disk this session.

## W7 Report (JSON terminus)

```json
{
  "lane": "CLI-01",
  "files_written": [
    "/Users/sac/cgap/crates/cli/manifest_patch.py",
    "/Users/sac/cgap/crates/cli/Cargo.toml",
    "/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-cli01-license-via-graph.md"
  ],
  "commands": [
    "cd /Users/sac/cgap/crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py  # rc=0 x2",
    "cmp /tmp/cli01-pre2.toml Cargo.toml  # byte-identical replay",
    "cargo build  # crates/cli, rc=0",
    "cargo metadata --no-deps (root) via run_checked  # rc=0, 23 members, cli absent",
    "grep -c license-file crates/cli/Cargo.toml  # 1 (flip witness)"
  ],
  "courts_run": ["byte-identical replay (gate0 discipline)", "flip witness grep"],
  "standing": "ALIVE",
  "divergences": [
    "Dispatch's ../LICENSE.md is wrong at crates/cli depth; landed ../../LICENSE.md per house style (cgap-core:10) and DOD-01 F3c precedent (crates/csct:7). crates/LICENSE.md absent, verified."
  ]
}
```

Note: write-integrity.sh BRACE_PAREN_DELTA hook flagged the first edit
(stray `)` in a comment heading, "# 1.5)"); fixed by rewording the
comment — final file AST-parses clean (`ast.parse` OK, verified).
