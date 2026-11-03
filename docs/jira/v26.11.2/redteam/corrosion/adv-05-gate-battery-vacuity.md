# Lane ADV-05 — gate battery polarity & vacuity probe

Date: 2026-11-02 · Branch: main · READ-ONLY over `scripts/` (all
mutations on /tmp copies). Receipt is the only file written.

## W1 Subject

`scripts/gate_battery.py` (sha256 `e38d5210dff68182...a617cd`, copied
byte-exact to `/tmp/adv05/scripts/gate_battery.py`; REPO resolves to
/tmp so the GATES registry points at sandbox mock child-gates — clean
isolation, shared tree untouched). Mock gates: `boundary_gate.py`,
`zero_string_gate.py`, `verify_cscb_sram.py`, `ffi_surface_gate.py`,
each `print("mock PASS")`; baseline rollup 4/4 PASS exit 0 witnessed
before every probe arm.

## W2 Commands + exits (this session)

- `python3 gate_battery.py --only boundary --only zero-string --only
  cscb-sram --only ffi-surface` (mocks) -> **exit 0**, 4/4 PASS
  (baseline).
- **Probe 1** (child exits 1, empty stdout) -> battery **exit 1**,
  row `boundary FAIL exit=1 (no output captured)`. No swallow; the
  empty-output sentinel is loud. HOLDS.
- **Probe 2a** (binary garbage stdout, exit 0) -> exit 0, PASS row with
  mojibake tail (`ÿþ garbage`). Exit wins, tail cosmetic. HOLDS.
- **Probe 2b** (binary garbage stdout, exit 1) -> battery **CRASH**:
  `UnicodeDecodeError: 'utf-8' codec can't decode byte 0xff` inside
  `subprocess.run(..., text=True)` at `gate_battery.py:92-94` ->
  traceback, exit 1. See verdict.
- **Probe 3** (lying gate: exit 0, prints FAIL) -> battery **exit 0**
  PASS. Truth-source = exit code; tail text is display-only. The
  docstring (lines 2-29) claims exit-code rollup only, no output
  auditing — so this is a REGISTERED bound, not a spoiler.
- **Probe 4** (child SIGKILLs itself) -> row `FAIL exit=-9`, battery
  **exit 1**. Negative returncode is not read as PASS. HOLDS.
- **Probe 5** (truncated gate: SyntaxError) -> row `FAIL exit=1
  SyntaxError: invalid syntax`, battery **exit 1**. No skip-silent.
  HOLDS.
- **Probe 6** (child dies mid-write via os._exit(3)) -> row `FAIL
  exit=3`, partial line captured as tail, battery **exit 1**. HOLDS.

## W3 Courts run

- Battery flip court via mock mutated gate: 6/7 arms flipped the
  rollup to FAIL (probes 1, 2b-crash, 4, 5, 6 all exit 1) — battery
  is not vacuous.
- **Probe 7 — `--prove-refusal` re-run post-R-18**:
  `python3 /Users/sac/cgap/scripts/gate_battery.py --prove-refusal`
  -> **exit 0**, court HOLDS: NA_MAX_MATCHES 2048->1024 injected into
  `/Users/sac/.cache/tmp/gate_battery_prove.0yq65eja/noalloc.rs`,
  `cscb-sram:mutated FAIL exit=1` (static NaMatches 1081352 B !=
  pinned 2162696 B), shared `crates/cgap-bn3/src/noalloc.rs` sha256
  `e039d3ab88b2d111...ceb483` byte-exact after (re-read from disk this
  session). Post-R-18 restores did not dent the court.

## W4 Fences (must-not-regress surface)

Read-only lane: no script edited; `scripts/gate_battery.py` sha256
unchanged on disk (`e38d5210dff68182...`). The battery's own fences are
the four house gates; probe 7 exercised the cscb-sram/bn3-pin court
end-to-end. Sandbox `/tmp/adv05` is lane debris, not shared-tree.

## W5 Divergences / verdicts

- **Verdict: IMMUNITY RECEIPT (no masking found) with two findings.**
- **Finding ADV-05-A (robustness defect, NON-masking)**: a child gate
  emitting invalid UTF-8 on a failing run crashes the battery with an
  unhandled `UnicodeDecodeError` (`run_gate`, `text=True`, no
  errors='replace'). Exit is still nonzero (fail-closed preserved, no
  PASS masking), but the crash aborts remaining rows and prints a
  traceback instead of a typed row. Suggested fix (owner lane):
  `text=True, errors="replace"` or capture bytes and decode with
  `errors="replace"` at tail-render time.
- **Registered bound (not a spoiler)**: a 0-exit lying gate
  (prints FAIL, exits 0) is UNDETECTABLE at the battery layer —
  truth-source is the child's exit code by documented ABI; the
  docstring does not claim output auditing, and per-gate self-checks
  are the designated output-audit layer. Reproducer: probe 3 mock
  (`p3_liar_pass.py` pattern) -> battery exit 0.
- Reproducers for all arms live in `/tmp/adv05/scripts/p*.py` (lane
  debris; not preserved in-tree — each probe is a 3-line mock,
  reconstructible from W2).

## W6 Standing

ALIVE — every probe carries a command + exit run this session against
the exact sha256-pinned subject.

## W7 JSON terminus

{"lane":"ADV-05","files_written":["docs/jira/v26.11.2/redteam/corrosion/adv-05-gate-battery-vacuity.md"],"commands":["cp gate_battery.py /tmp/adv05/scripts (sha e38d5210 byte-exact)","baseline mock rollup -> 0 (4/4)","probe1 empty-stdout-fail -> 1","probe2a garbage-exit0 -> 0","probe2b garbage-exit1 -> UnicodeDecodeError crash (exit 1)","probe3 lying-gate-exit0 -> 0 (registered bound)","probe4 SIGKILL -> 1 (exit=-9)","probe5 syntax-error -> 1","probe6 mid-write death -> 1 (exit=3)","python3 scripts/gate_battery.py --prove-refusal -> 0 (court HOLDS; shared noalloc.rs sha e039d3ab byte-exact)"],"courts_run":["battery flip battery (6/7 arms flipped FAIL)","--prove-refusal post-R18 (HOLDS, exit 0)"],"standing":"ALIVE","divergences":["ADV-05-A: invalid-UTF-8 child output crashes battery (non-masking, fail-closed preserved)","0-exit lying gate undetectable at battery layer — registered bound per docstring (no output-audit claim)"]}
