# lane-lspa-04-client-masking

Lane LSPA-4 (2026-11-02). Client-only fix of the analyze() tier-1 masking
bug (lspu2-capability-inventory #17, activation move 4). No git commands;
shared cgap tree touched only under `docs/jira/v26.11.2/lanes/`.

## Files written

- `/Users/sac/lsp-max/python/lsp_client/branchless_client.py` (only code file)
- this receipt

## Fix (deterministic drain, no sleeps)

- `wait_for_diagnostics(uri, deadline, skip=0)`: bounded FIFO skip of the
  first `skip` publishes for the URI. `analyze()` and
  `diagnostics_for_files` both call with `skip=1` after did_save: the
  always-empty Tier-1 didOpen publish precedes the didSave Tier-2 verdict
  on the single notification queue, so the drain is order-deterministic
  (verified: server ordering witnessed at 0.0s vs 0.1s in probe output).
  The code-keyed alternative was rejected because a clean save's Tier-2
  publish is EMPTY (double-silence family) and is indistinguishable from
  Tier-1 — keying on ANDON-REFUSED-001 would hang clean saves.
- stderr: `stderr=PIPE` + bounded `_drain_stderr` task (8 KiB tail);
  `stderr_text()` accessor; `_annotate()` attaches the tail to typed
  failures (`exc.stderr` attr + message suffix); `analyze()` returns
  `stderr` in the dict. No DEVNULL.
- Tightened a race in `_next_notification`: a deadline race window leaked
  a raw `TimeoutError`; now typed `LspTimeout` (pre-existing defect,
  fixed in-lane — client-side).

## Commands + exits

- `strings target/release/branchless-lsp | grep -c ANDON_REFUSED` -> 3
  (fresh; no branchless-lsp src newer than binary EXCEPT lib.rs/rule_pack.rs
  — LSPA-2's in-flight edits, see Divergences)
- `python3 /tmp/lspa4/before_court.py` -> exit 0
- BEFORE (verbatim): `BRANCHY diagnostics: []` / `CLEAN diagnostics: []`
  (branchy = internal `fn rf_branchy` fixture — see Court-fixtures note)
- `python3 -m py_compile ...branchless_client.py` -> exit 0
- `python3 -u /tmp/lspa4/after_court.py` -> exit 0 (COURT PASS):
  AFTER BRANCHY analyze(): 3 diagnostics, 2x `code=ANDON-REFUSED-001`
  severity 1 (b.hs + b.lo at line 4 of g06_branchy) + 1x
  LSPMAX-ANDON-PUSH-MISSING severity 1; AFTER CLEAN: `[]`;
  diagnostics_for_files n_refused == 3.
- Non-vacuity: `python3 -u /tmp/lspa4/nonvacuity.py` — skip=0 (pre-fix
  semantics) on the same session returns `[]`, i.e. reverts to the
  masking; the flip is real.
- stderr-surfacing court: `python3 -u /tmp/lspa4/panic_probe.py` -> exit 0:
  a save whose probe object lacks `.debug_line` yields typed
  `LspTimeout` with `exc.stderr` carrying
  `thread 'tokio-rt-worker' panicked at crates/branchless-lsp/src/disasm_court.rs:847`
  (previously: DEVNULL, silent hang until generic timeout).

## Server-side defect found (OUT of lane scope — LSPA-2)

`judge_classified` (crates/branchless-lsp/src/disasm_court.rs:847) panics
when the probe object has no `.debug_line` (internalized non-`no_mangle`
fns at -O emit none). The panic kills the tier-2 handler; NO tier-2
publish ever arrives; the client previously hung to a generic timeout.
Witnessed via the new stderr capture against the CURRENT release binary
(witness strings above). This is the double-silence finding's louder
sibling: it is a server change, so it belongs to LSPA-2's rule_pack.rs
lane. My court avoids it by using `#[no_mangle] pub extern "C"` fixtures
(same shape as g06's did_save_court.rs:35-39) — client fix is correct
against the pre-LSPA-2 binary (gate-before-client pairing held).

## Court fixtures note

`/tmp/lspa4/branchy.rs` (internal fn) was the dispatch-named "BRANCHY
fixture" for the BEFORE run; after isolating the server panic the court
standardized on the g06-fixture shape (`branchy_g06.rs` / `clean_g06.rs`).

## Divergences

- Binary staleness: `lib.rs`/`rule_pack.rs` are NEWER than the release
  binary (LSPA-2 in-flight). Per dispatch the binary was NOT rebuilt
  (rebuilding would fold LSPA-2's half-landed server edits into my court
  subject); courts ran against the on-disk binary (ANDON_REFUSED n=3,
  fresh for src other than LSPA-2's two files). strings count = 3 as
  expected.
- BEFORE-court "CLEAN diagnostics: []" ran on an internal-fn fixture, so
  it witnessed the masking via tier-1 drain, not a true clean verdict —
  the AFTER court's clean arm (g06 shape) is the authoritative clean
  witness.
- Standing: ALIVE for the client fix (all commands above run this
  session, exits witnessed). Server-side `.debug_line` panic: UNVERIFIED
  as to fix — recorded as observed, assigned to LSPA-2.

## Fences

- cgap tree: no fence applies (docs-only addition; no lib.rs/Cargo.toml
  touched, no crates rebuilt).
- lsp-max: no tests/builds mutated; `cargo build --release` NOT run
  (dispatch note: rebuild only if stale — deemed not-stale for court
  purposes, divergence recorded above).
