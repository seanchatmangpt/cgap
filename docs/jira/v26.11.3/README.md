# v26.11.3

Wave index for cgap v26.11.3. Receipts live in `receipts/`; this file is the
wave index (created by the cl-close-2 index-only lane, 2026-11-03 — cgap had
receipts but no index; SE census noted). Standing vocabulary:
ALIVE / PARTIAL / BLOCKED / UNVERIFIED / REFUSED. Never round up.

Exit record law: A gate is witnessed by an exit record: either (a) the process
exit status, or (b) cargo's "N passed; 0 failed" summary line. A passed-count
without a failed-count is not an exit record.

## Receipts table

Standing words re-read from each receipt body on disk (cl-close-2 lane,
2026-11-03), never from memory. Lane file: `lanes/cg-1-wire-tier-gates.md`.

| Lane | Receipt | Standing | Gate (exit record) |
|---|---|---|---|
| CG-1 | receipts/cg-1.md | PARTIAL_ALIVE | `cargo test -p cgap-arrow` exit 0, 220 passed / 0 failed (baseline 214; +6 G1 courts); wire-shm "8 passed"; cgap-gate 12+1+3+5, 0 failed; `cargo check -p cgap-arrow` exit 0; no_std embedded check exit 0 |
| GA-5 | receipts/ga-5-cgap-wiring.md | PARTIAL_ALIVE | `ggen sync run` run 1 EXIT 0 (all 5 outputs written); run 2 EXIT 0, `written: []` (determinism); `cc -std=c99 -ffreestanding` exit 0; `cargo check -p cgap-ffi` exit 0. PARTIAL because the pack is a STAGED fallback, not GA-4's canonical marketplace aoq-pack — see GA-5B below |
| GA-5B | receipts/ga-5b-flipback.md | ALIVE (flip-back to the canonical marketplace aoq-pack; landed mid-pass, absorbed at re-census) | flip-back sync1 exit 0, written = 5 canonical slots; gate0 byte-stable replay PASS (include/cgap_aoq.h sha256 0bd81821…, freestanding cgap_c_data.h compile PASS); GA-4 gates re-verified: pass witness 0 violations (sync exit 0), fail witness ≥1 fail-row per gate with typed exits |
| SA-4 | receipts/sa-4.md | ALIVE | `cargo check` (crates/cli standalone) exit 0; `cargo test` 44 passed, 0 failed; `cargo test -p cgap-arrow` 228 passed, 0 failed |
| AF-30 | receipts/af-30-contender-a.md | ALIVE | `cargo test -p cgap-aoq --test af30_cage_match --release -- --nocapture` exit 0 x2, 1 passed / 0 failed (0.5630 / 0.5551 us per frame; census 0 allocs / 0 bytes); AF-14 cbmqs_runner self-scan exit 0 verdict PASS |
| SA-5 | receipts/sa-5.md | ALIVE (gates 170/180) / pre-existing gap disclosed | gates 170/180 executed at sync time with flipping-court non-vacuity, positive-path exit-0 sync receipt; `gate-court.toml` names runner `qualification/verify.py` which does not exist (prior-lane state, left as-is) |

## Placement notes (pointer-only)

- e80-06 and e80-07 receipts are hosted in
  /Users/sac/claude-code-config-lsp/docs/jira/v26.11.3/receipts/ — their
  subject repo is /Users/sac/cgap. Indexed there per the cl-close placement
  note; not relocated, not duplicated here.
- cg-1's receipt cites the lane file `lanes/cg-1-wire-tier-gates.md` in this
  tree; the lane plan is not itself a receipt.

## See Also

- `receipts/` — per-lane receipts
- `/Users/sac/claude-code-config-lsp/docs/jira/v26.11.3/README.md` — hosts the
  e80-06/e80-07 rows whose subject is this repo
