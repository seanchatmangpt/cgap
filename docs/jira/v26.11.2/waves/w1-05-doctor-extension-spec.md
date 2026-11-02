# W1-05: `just doctor` extension spec (SPEC ONLY — W2 implements)

Lane: W1-05 (spec lane). No code changes this lane. Baseline grounded this
session: `just doctor` exit 1 — 8 GREEN, 2 YELLOW (c06 workspace-check 5
warnings; c08 receipt staleness), 0 RED of 10 checks (doctor.rs:2-6 exit
model: 0 all-GREEN / 1 any-YELLOW / 2 any-RED).

## 1. Current wire-points (cited)

- `justfile:19-20` — `doctor:` recipe, single invocation:
  `cargo run -q --manifest-path crates/cli/Cargo.toml --bin doctor`.
- `crates/cli/src/bin/doctor.rs:3-31` — 10 checks c01..c10,
  St::{Green,Yellow,Red} (doctor.rs:27-53), exit 0/1/2 (doctor.rs:5-6).
- `crates/cli/src/bin/doctor.rs:320-323` — c07 scripts check hardcodes a
  7-element SCRIPTS list; missing -> YELLOW, exists-but-not-exec -> RED.
- `crates/cli/src/bin/doctor.rs:372` — c08 receipts staleness (mtime).
- Existing gates: `scripts/boundary_gate.py:139-142` (EXIT_PASS=0,
  EXIT_REFUSE=1), `scripts/zero_string_gate.py:2214-2232` (argparse:
  --root/--target-dir/--profile/--vocab/--no-cross-check; modes
  --self-check/--courts/--dep-direction/--register),
  `scripts/release_check.py:1-16,283,315` (stdlib-only, exit 1 on any
  violation, final line `RELEASE-CHECK: PASS|FAIL <reasons>`).

## 2. Extended doctor chain (target state)

Order (cheapest/fail-fast first; python gates run AFTER the compile-backed
bin so a RED toolchain does not burn gate runtime):

1. `cargo run -q --manifest-path crates/cli/Cargo.toml --bin doctor`
   (unchanged, justfile:20). Exit 0/1/2 mapped directly (see section 3).
2. `python3 scripts/zero_datafusion_gate.py`
3. `python3 scripts/receipt_lint.py --root .`
4. `python3 scripts/registry_sweep.py`

## 3. Recipe: `doctor` (revised, drop-in replacement for justfile:19-20)

```just
doctor:
    @set -e; \
    cargo run -q --manifest-path crates/cli/Cargo.toml --bin doctor; \
    rc=$?; [ $$rc -le 1 ] || exit $$rc; \
    python3 scripts/zero_datafusion_gate.py; rc=$$?; case $$rc in \
      0) echo 'd11 zero-datafusion: PASS' ;; \
      1) echo 'd11 zero-datafusion: FAIL' >&2; exit 1 ;; \
      2) echo 'd11 zero-datafusion: SKIP (subject missing)' >&2; exit 3 ;; \
      3) echo 'd11 zero-datafusion: SKIP (loud)' >&2; exit 3 ;; esac
```

Precedent for the 0/1/2/3 exit classes: CLAUDE.md gate convention
(`0 pass / 1 gate fail / 2 subject missing / 3 loud skip`) and the
fail-until-landed guard pattern at `justfile:50-52` (`dod:` recipe's
`if [ -f scripts/dod.sh ]` branch).

### Why exit-2 stays fatal here, not YELLOW

doctor.rs maps RED -> exit 2 and treats a missing script as YELLOW only
because wave-10 lanes owned those scripts (doctor.rs:320-321). W2 gates are
contracted artifacts of THIS campaign: if `zero_datafusion_gate.py` is
absent at integration time that is a wave defect, not an environmental
yellow. Doctor bin c07 (doctor.rs:320-341) gains three rows in its SCRIPTS
array (doctor.rs:323-331) once the files land — W2 adds:
`zero_datafusion_gate.py`, `receipt_lint.py`, `registry_sweep.py`.

## 4. Interface specs for W2 (frozen contract)

### 4.1 scripts/zero_datafusion_gate.py

- Stdlib-only python3 (precedent: release_check.py:4-12).
- Flags: `--root <dir>` (default repo root, mirror
  zero_string_gate.py:2215), `--target-dir <dir>` (default `<root>/target`),
  `--self-check` (non-vacuity arm, mirror zero_string_gate.py:2225-2227),
  `--json` (machine output), `--verbose`.
- Behavior: scan `crates/` + `Cargo.toml` graph for any `cgap-datafusion`
  reference (dependency edge, `use`, path dep, doc pointer marked load-
  bearing). Any hit -> exit 1 with file:line. `--self-check` plants a
  marked poison string into a /tmp sandbox copy of the scan corpus and
  MUST fail (exit 1); a self-check that passes exits nonzero itself —
  `VACUOUS GATE` (precedent: zero_string_gate.py:33,113-115; boundary_gate
  §6.3).
- Exit codes: 0 clean / 1 violation or VACUOUS / 2 subject missing /
  3 loud skip with printed `[SKIP] <reason>`.
- `--json` schema:
  `{"gate":"zero_datafusion","verdict":"PASS|FAIL|SKIP",`
   `"violations":[{"path":str,"line":int,"snippet":str}],`
   `"self_check":{"ran":bool,"poison_killed":bool},"seed":null}`
- Final stdout line: `ZERO-DATAFUSION: PASS` or `ZERO-DATAFUSION: FAIL n`
  (mirrors release_check.py:15).

### 4.2 scripts/receipt_lint.py

- Stdlib-only. Flags: `--root <dir>`, `--dir <receipts-dir>` (repeatable,
  default `docs/jira/v26.11.2/waves`), `--strict` (treat YELLOW as exit 1),
  `--json`, `--self-check` (injects a malformed receipt fixture under /tmp
  and must fail).
- Lint rules per receipt markdown: (a) H1 matches filename stem (kebab-
  case, tools.md markdown law); (b) `| Standing |` table row present and
  value in {ALIVE, PARTIAL, BLOCKED, REFUSED, UNVERIFIED}; (c) every ALIVE
  claim carries a fenced command block within 5 lines (grounding law);
  (d) lines <= 100 chars; (e) no `Process.sleep` in cited test commands.
- Violations of (a)-(c) -> FAIL (exit 1); (d)-(e) -> YELLOW (exit 1 only with
  `--strict`; default doctor mode prints YELLOW but exits 0 — see 4.2).
- Exit codes: 0 clean-or-YELLOW-only / 1 FAIL / 2 receipts dir missing /
  3 loud skip.
- `--json` schema:
  `{"gate":"receipt_lint","verdict":"PASS|YELLOW|FAIL|SKIP",`
   `"files_scanned":int,"findings":[{"file":str,"rule":"a|b|c|d|e",`
   `"line":int,"msg":str}],"standing_tally":{"ALIVE":int,...}}`
  (standing_tally mirrors the justfile:45-47 awk tally.)

### 4.3 scripts/registry_sweep.py (interface pinned for W2, not specced here)

Owned by its own W2 lane; doctor wire contract only: takes no required
flags, honors `--root`, emits final line `REGISTRY-SWEEP: PASS|FAIL n`,
exits 0/1/2/3 per the same class table. Doctor treats it identically to
4.1.

## 5. PASS/FAIL/YELLOW semantics (matching doctor.rs)

- doctor bin exit 0 -> GREEN contribution; exit 1 -> YELLOW (c06/c08 are
  currently YELLOW — witnessed run above); exit 2 -> RED (chain aborts,
  exit 2).
- Each python gate: exit 0 -> check GREEN; exit 1 -> RED (gate refuse is
  a violation, not informational — boundary_gate.py:140 EXIT_REFUSE);
  exit 2/3 -> YELLOW with `[SKIP]` reason echoed (loud skip law).
- Summary line gains 3 checks: d11 zero-datafusion, d12 receipt-lint,
  d13 registry-sweep, rendered in the existing doctor.rs table
  (doctor.rs:489+ width constants, `st_w = 6` for "YELLOW").

## 6. Anti-vacuity (named mutations, W2 must implement)

- zero_datafusion_gate: mutation = add a `cgap-datafusion` path dep line to
  a sandbox Cargo.toml -> gate must flip to exit 1; restore sha256-exact +
  mtime bump (CLAUDE.md mtime-bump law).
- receipt_lint: mutation = strip the Standing row from a sandbox receipt ->
  exit 1.
- registry_sweep: mutation defined by its own W2 lane.
- `--self-check` arms for the first two are part of the frozen contract
  (4.1/4.2) — a gate whose self-check passes is VACUOUS and exits nonzero.

## 7. Standing

- This lane: SPEC ONLY, no code written. Baseline `just doctor` observed
  this session (exit 1, YELLOW). W2 gates: UNVERIFIED (absent on disk).
- Divergence: doctor.rs c07 SCRIPTS list (doctor.rs:323) does not yet name
  the three W2 scripts; W2 must extend it or the c07 check stays at 7/7
  and never observes the new gates (silent-missing risk).
