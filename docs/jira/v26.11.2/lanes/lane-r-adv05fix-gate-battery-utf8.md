# Lane R-ADV05FIX — gate battery UTF-8 crash repair (ADV-05-A)

Date: 2026-11-02 · Branch: main · Subject:
`scripts/gate_battery.py` sha256 `379bab50a21881a4...41ecfb05`
(post-fix; pre-fix sha from ADV-05 receipt: `e38d5210dff68182...`).

## W1 Subject + diff

Owned file: `scripts/gate_battery.py` ONLY. Two edits:

1. `run_gate` now captures child output as BYTES (dropped
   `text=True`), then decodes UTF-8 strictly, falling back to
   `errors="replace"` with a typed `(binary output, N replacement
   chars)` prefix on the tail. Exit-code rollup logic untouched.
2. No other behavioral change; `battery()`, `run_battery()`,
   `prove_refusal()` untouched.

## W2 Commands + exits (this session)

Sandbox `/tmp/adv05fix` (mock gates, REPO resolves to /tmp):

- baseline mock rollup -> exit 0, 4/4 PASS.
- Probe 2b (invalid UTF-8 stdout, exit 1) -> battery exit 1, row
  `boundary FAIL exit=1 (binary output, 4 replacement chars)` —
  NO traceback (was UnicodeDecodeError crash pre-fix). FIXED.
- Probe 2a (invalid UTF-8, exit 0) -> exit 0, PASS row with the
  binary note prefix. HOLDS.
- Probe 1 (exit 1, empty stdout) -> exit 1. HOLDS.
- Probe 4 (SIGKILL) -> exit 1 (exit=-9). HOLDS.
- Probe 5 (SyntaxError) -> exit 1. HOLDS.
- Probe 6 (os._exit(3) mid-write) -> exit 1, partial tail. HOLDS.
- Probe 3 (lying gate exit 0) -> exit 0 (registered bound). HOLDS.
- Sandbox full battery (fixed script) -> exit 0, 4/4.
- Sandbox `--with-self-check` -> exit 0, 8/8.

Shared tree:

- `python3 scripts/gate_battery.py --prove-refusal` -> exit 0, court
  HOLDS (mutated bn3-pin gate exit 1 flipped the rollup; shared
  noalloc.rs sha `e039d3ab88b2d111` byte-exact).
- `python3 scripts/gate_battery.py` -> exit 1: 3/4 PASS; the
  `ffi-surface` row FAILs with `could not compile cgap-ffi (lib)`
  (2 errors, later 1 error) — a FOREIGN lane's in-flight cgap-ffi
  edit (`crates/cgap-ffi/src/lib.rs` mid-write, "expected `{`, found
  `lease`"), not this diff. Bounded poll x6 over ~minutes: no
  convergence. `--with-self-check` -> exit 1, 6/8, same cause.
- Sandbox mocks with the fixed script prove the 4/4 and 8/8 exit-0
  paths; the shared-tree ffi row is foreign-blocked, not regressed
  (byte-identical battery behavior on the three passing rows).
- Sandbox copy verified sha256-identical to shared
  (`379bab50...41ecfb05`).

## W3 Courts run

- ADV-05 probe battery re-run: all 7 probe outcomes preserved
  (1,2a,2b,3,4,5,6) against the FIXED script. Probe 2b flips crash
  -> typed loud row.
- `--prove-refusal` post-fix -> exit 0, HOLDS.

## W4 Fences

`--prove-refusal` anti-vacuity (run, holds). The four house gates'
own self-checks unaffected (boundary/zero-string/cscb-sram PASS rows
witnessed). No shared lib.rs/Cargo.toml touched.

## W5 Divergences

- Pre-existing (NOT this diff): ffi-surface gate FAIL exit 3 from a
  foreign lane's in-flight cgap-ffi compile break; battery 4/4 and
  8/8 on the shared tree are therefore foreign-BLOCKED this session
  (PROOF-BY-SANDBOX instead: exit 0, 4/4 and 8/8 with the fixed
  script over mock gates). Owner of cgap-ffi supersedes on landing.
- Registered bound carried from ADV-05: 0-exit lying gate remains
  undetectable at battery layer (docstring ABI: truth-source = exit
  code).

## W6 Standing

ALIVE for the fix surface (ADV-05-A): command + exit this session.
Shared-tree full-battery 4/4 and 8/8: BLOCKED (foreign in-flight
cgap-ffi edit), proof-by-sandbox 0/0 exit.

## W7 JSON terminus

{"lane":"R-ADV05FIX","files_written":["scripts/gate_battery.py","docs/jira/v26.11.2/lanes/lane-r-adv05fix-gate-battery-utf8.md"],"commands":["cp fixed battery to /tmp/adv05fix/scripts (sha 379bab50 identical)","sandbox baseline mock -> 0 (4/4)","probe2b garbage-exit1 -> 1 typed row (was UnicodeDecodeError crash)","probe2a garbage-exit0 -> 0 with binary note","probe1 empty-stdout-fail -> 1","probe4 SIGKILL -> 1","probe5 syntax-error -> 1","probe6 mid-write death -> 1","probe3 lying-gate-exit0 -> 0 (registered bound)","sandbox full -> 0 (4/4)","sandbox --with-self-check -> 0 (8/8)","python3 scripts/gate_battery.py --prove-refusal -> 0 (HOLDS)","python3 scripts/gate_battery.py -> 1 (3/4; ffi-surface FAIL exit=3 foreign cgap-ffi compile break)","python3 scripts/gate_battery.py --with-self-check -> 1 (6/8 same cause)"],"courts_run":["ADV-05 7-probe battery vs fixed script (all outcomes preserved; 2b flipped crash->typed row)","--prove-refusal post-fix (HOLDS, exit 0)"],"standing":"ALIVE","divergences":["shared-tree battery 4/4 & 8/8 foreign-BLOCKED by another lane's in-flight cgap-ffi compile break; exit-0 paths proven via sandbox mocks (4/4, 8/8)"]}
