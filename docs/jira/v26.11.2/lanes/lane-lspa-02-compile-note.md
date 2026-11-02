# Lane LSPA-2 — compile-note WARNING emission (branchless-lsp)

Date: 2026-11-02. Repos: /Users/sac/lsp-max (source) +
/Users/sac/cgap (receipt only). No git commands run in either tree.

## Files written

- `/Users/sac/lsp-max/crates/branchless-lsp/src/rule_pack.rs` — TWO hunks in
  `verdict_diagnostics` + one new fn: when `Tier2Outcome.note` is `Some`
  (rustc-failure save), emit a WARNING-class diagnostic (code
  `COMPILE-NOTE-001`, emitted FIRST for deterministic ordering), message
  `COMPILE_NOTE (court not evaluated): <note>` with a char-boundary-safe
  512-char cap and `...[truncated]` suffix. New pub consts:
  `COMPILE_NOTE_RULE_ID`, `NOTE_TRUNCATE` (private). NO other change: on
  successful compiles `note` is None and the function is byte-identical in
  behavior (ERROR/ANDON path untouched).
- `/Users/sac/lsp-max/crates/branchless-lsp/tests/compile_note_court.rs` (NEW,
  209 lines) — the court.
- `scripts/gate-branchless-lsp.py`: NOT modified — read-verified instead
  (below).
- `/Users/sac/lsp-max/target/release/branchless-lsp` (build output, rebuilt
  with the fix; release exit 0, 16.90s).

## Court (Chicago)

`tests/compile_note_court.rs`, 5 tests, real collaborators only: real
LspService + Client/ClientSocket loopback, real tower initialize, real
`rustc --emit=obj` through run_tier2, real wire publishDiagnostics off the
socket (G06 pattern). Anti-vacuity flip: SYNTAX-ERROR save
(`fn f() -> u32 { 1 } }`) must publish >=1 WARNING COMPILE-NOTE carrying the
rustc text; CLEAN save over the SAME pipeline must publish ZERO notes (and
zero ERROR). Plus: ordering arm (note first), truncation-determinism arm,
gate-parity arm (WARNING=2 != ERROR=1).

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/lspa2-target cargo test -p branchless-lsp --test
  compile_note_court` -> exit 0, 5 passed / 0 failed (twice: first run and
  full-suite pass-through).
- `CARGO_TARGET_DIR=/tmp/lspa2-target cargo build -p branchless-lsp` -> exit 0
  (after a foreign lib.rs freeze cleared; see Divergences).
- `cargo build --release -p branchless-lsp` -> exit 0 (16.90s), binary rebuilt
  with the fix.
- `cargo test -p branchless-lsp` (full): BLOCKED by foreign suites — two
  background runs killed at the 30-minute limit; second run's log shows 72
  lib + 0 main + 5 code_actions + 5 compile_note ALL ok, then >20 min stuck
  inside foreign `conformance_seal_court::session_totals_observable_before_shutdown`
  (hang, not a failure). Individual suites re-run to completion:
  e9_precision_court exit 0 (6/6), riscv32imc_court exit 0 (3/3),
  code_actions_courts exit 0 (5/5), compile_note_court exit 0 (5/5).
  did_save_court exit 101 (1 foreign failure — see Fences).
- Gate parity witness (python, exit 0): `g.is_error({severity:2,...})` =
  False, `g.is_error({severity:1,...})` = True — a WARNING does NOT flip
  gate-branchless-lsp.py:120-137; no guard written, none needed.
- Wire probe `/tmp/lspa2_drain.py` (g05_drain pattern, release binary rebuilt
  with fix), exit 0:
  - ERR fixture (`fn f() -> u32 { 1 } }`): publish[1] n=1,
    sev=2 msg=`COMPILE_NOTE (court not evaluated): compile error (not a
    court verdict): error: ...` — BEFORE this lane the same save published
    n=0 (silent).
  - BRANCHY: unchanged — publish[1] n=3 (2x ANDON_REFUSED sev=1 @line 3 +
    1 blocking diag), conformance.refused=[ANDON_REFUSED].
  - CLEAN: unchanged — publish[1] n=0, no note, no ERROR.
- LSPU-2 falsifier status: a rustc-error save is no longer
  indistinguishable from clean on the wire (WARNING published, code
  COMPILE-NOTE-001). Note on scope: the LSPU-2 "top-5" text frames the
  falsifier as "gate passes a commit staging `fn f() -> u32 { 1 } }`";
  the dispatch's tasks (1)+(3) explicitly pin the design to WARNING-class
  + gate NOT flipping, which is what landed. Tasks (1)+(3) govern;
  recorded as a dispatch-internal tension, not silently resolved.

## Fences (must-not-regress)

- Fence: existing branchless-lsp suites — RAN individually: e9 6/6,
  riscv32imc 3/3, code_actions 5/5, compile_note 5/5, lib 72/72 (suite log).
- Fence: did_save_court.rs (G06 + foreign LSPA-3 additions) — ran, exit 101:
  `lspa3_measured_budget_admits_branchy_on_wire` FAILED. NOT my regression:
  my diff adds diagnostics ONLY when note.is_some() (None on successful
  compiles); the failing arm's ERROR set comes from the classification path,
  untouched. It is another lane's in-flight fixture (their budget arm refuses
  where the test expects admission). Baseline UNVERIFIED — crates/branchless-lsp
  is untracked in lsp-max, so no committed pre-lane baseline exists to diff.
- Fence: gate script untouched; is_error parity witnessed (above).
- Fence: lsp-max tree: no git commands; cgap tree: receipt only.

## Divergences

1. Compile-freeze disclosure (received, not given): foreign lane edited
   lib.rs (CONFORMANCE-SEAL hunk) mid-flight with 3 compile errors; build
   frozen ~3 min; owner fixed within the SLA; no unblock applied by me.
2. Full `cargo test -p branchless-lsp` could not complete: foreign
   `conformance_seal_court` test hangs >20 min (two 30-minute background
   windows exhausted). Per-suite results witnessed instead. The hang is in
   another lane's in-flight test file; not repaired by me.
3. ERRATA (re: docs/jira/v26.11.2/lanes/lane-g05-lsp-wire-rebuild.md:39):
   the G05 receipt's CLEAN arm (`CLEAN: publish[1] n=0`) ran a MALFORMED
   fixture — the probe's `CLEAN.replace("open1","pub")` yields
   `pub extern "C" pub fn` (double `pub`), a rustc syntax error whose
   silence was the exact double-silence defect this lane fixes. G05's clean
   claim was right conclusion / wrong witness; witnessed correct clean
   behavior here with the fixed fixture (`replace("open1 ","")`).
4. Dispatch named writing lsp-max `scripts/gate-branchless-lsp.py` "only if
   ERROR counting needs a guard" — it does not (is_error keys on
   severity in ("ERROR","error",1); WARNING=2 falls through). File untouched.
5. did_save_court.rs gained foreign `lspa3` tests mid-lane (file I did not
   write and did not modify); the 1 failing lspa3 arm is recorded above.
