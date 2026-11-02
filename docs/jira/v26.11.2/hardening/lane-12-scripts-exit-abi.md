# Lane H12 — gate scripts exit-ABI normalization (EX11)

Date: 2026-11-02. Owners touched: `scripts/boundary_gate.py`,
`scripts/zero_string_gate.py` only. Shared tree untouched otherwise; no git
writes; all mutation scenarios ran in `/tmp` sandboxes.

## Finding (EX11 restated)

Both scripts implemented exits {0, 1} only vs the documented 0/1/2/3 gate
contract (repo CLAUDE.md): 0 pass / 1 fail / 2 subject-missing / 3 loud skip.
Subject-missing and loud-skip paths collapsed into exit 1.

## Exit ABI (landed)

| Exit | Class | boundary_gate.py | zero_string_gate.py |
|---|---|---|---|
| 0 | PASS | unchanged (witnessed --deps/--strings/campaign green) | unchanged (witnessed scan/self-check/register green) |
| 1 | REFUSE | unchanged: vacuous gate, red baseline, restore mismatch, cross-gate red, bad input (`--targets` escape/duplicate, bad `--only-op`, bad triage key) | unchanged: hits red, vacuous, tool-failed, emitter-fence/unresolved, extractor mismatch, internal error |
| 2 | SUBJECT_MISSING | `covered_crate_dirs()` missing covered crate dirs; `parse_targets()` TARGET MISSING (NEW) | GateRefused classes {vocab-missing, vocab-empty, vocab-unparsable, workspace-manifest, member-manifest-missing, crate-no-artifacts, no-artifacts-scanned, execution-crate-no-artifacts, artifact-vanished, tool-missing} mapped to 2 in main() (NEW) |
| 3 | SKIP_LOUD | `--strings` over covered crates with zero scannable .rs/.toml files -> `[SKIP]` + exit 3 (NEW; fires before the canonical-scan spawn) | cmd_scan scanned=0 with skipped_artifacts>0 (all discovered artifacts loud-[SKIP]ed, e.g. wasm) -> exit 3; cmd_dep_direction checked=0 skipped>0 -> exit 3 (NEW). Nothing discovered at all stays the crate-no-artifacts refusal -> exit 2. |

Boundary: the checked-in `[SKIP]` row classes (rmeta members,
artifact-vanished single skips) are unchanged — they remain intra-run skips
counted in `skipped_artifacts`; exit 3 fires only when a run ends with
NOTHING observed (scanned=0) despite artifacts discovered.

## Exit matrix (before -> after, all witnessed this session)

| Scenario | Command (sandbox unless noted) | Before (HEAD) | After |
|---|---|---|---|
| repo --deps green | `python3 scripts/boundary_gate.py --deps` (repo root) | 0 | 0 (unchanged) |
| repo --strings green | `python3 scripts/boundary_gate.py --strings` (repo root; CROSS-GATE GREEN, scanned=73) | 0 | 0 (unchanged) |
| zero_string scan green | `python3 scripts/zero_string_gate.py` (repo root; hits=0, scanned=73) | 0 | 0 (unchanged) |
| zero_string register | `--register` | 0 | 0 (unchanged) |
| bg covered crates missing | sandbox `boundary_gate.py --deps` (no crates/ tree) | 1 | 2 |
| bg target file missing | sandbox `boundary_gate.py --mutate --targets nope.rs` | 1 | 2 |
| bg --strings zero files | sandbox, all 15 covered crate dirs exist, no .rs/.toml | 1 (via canonical-scan refusal) | 3 `[SKIP] STRINGS GATE: 0 scannable ...` |
| zs no corpus | sandbox root, members=[], no ontologies, no archive | 1 | 2 `vocab-missing` |
| zs all artifacts skipped | sandbox root, one bin crate whose artifact is wasm-magic | 1 (crate-no-artifacts) | 3 `[SKIP] SCAN-SUMMARY scanned=0 skipped_artifacts=1 ...` |

## Verification runs (this session)

- `python3 -m py_compile scripts/boundary_gate.py scripts/zero_string_gate.py` -> COMPILE_OK
- `pyright scripts/boundary_gate.py scripts/zero_string_gate.py` -> 0 errors, 0 warnings
- Baseline re-runs after the edits: deps exit 0; strings exit 0; zero_string
  default scan exit 0 (SUMMARY scanned=73 hits=0 gate=1.4.2); register exit 0.
- "Before" codes witnessed by running `git show HEAD:` copies of both scripts
  inside the same sandboxes (rows above).
- Gate version bumped 1.4.1 -> 1.4.2 (zero_string_gate docstring Exit-ABI
  block + GATE_VERSION constant).

## Divergences / notes

1. The zero_string_gate version bump changes the SUMMARY `gate=` field;
   boundary_gate pins A07_REGISTER_VERSION (2) and the canonical-scan
   SUMMARY fields — neither pins GATE_VERSION, so no consumer ABI breaks.
2. boundary_gate exit 3 fires BEFORE the canonical A07 scan spawn (a
   zero-file tree cannot produce a canonical verdict anyway). On the
   witnessed real tree (thousands of files) the path is not taken and
   --strings stays exit 0.
3. The `not ran` defensive branch of mode_deps_strings and
   TYPED_REASON_NOT_IMPLEMENTED remain exit 1 (bad input, not a skip).
4. Compile-freeze/shared lib.rs: not applicable — scripts-only lane; no
   Rust rebuild, no mtime concerns.
