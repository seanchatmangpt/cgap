# W6 Lane B7 — cgap-gate consumes cgap-branchless (receipt)

# Verdict

BLOCKED-on-B2/B3 for body swaps; dep wiring landed and verified. Standing:
PARTIAL (dep landed, suite green, bench run, zero-regression hold; zero
body-site swaps — see Divergence).

# Diff summary

- `crates/cgap-gate/Cargo.toml` — appended `[dependencies]` with the
  workspace-path dep `cgap-branchless = { path = "../cgap-branchless" }`
  plus a comment block naming the deferral. No other line touched.
- `crates/cgap-gate/src/penstock.rs` — UNCHANGED (0 diff lines). Zero
  sites swapped.
- Receipt file itself.

# Why zero body-site swaps (the divergence)

The dispatch named "penstock stage-1 SWAR mask helpers and prefilter8
arithmetic" as swap candidates. Neither exists as a match:

- `prefilter8` is `(w ^ (w>>7)).wrapping_mul(0x2544_5491_4F6C_DD1D)` —
  no such xor-fold-multiply primitive exists in the shipped branchless
  crate (`arithmetic.rs` carries only `saturated_subs_p28` and
  `poison_mask`; `swar.rs` carries eq/zero/range/lt masks).
- `is_zero_at` is a variable-length early-exit bounds-checked loop;
  `swar_has_zero_byte` is a fixed single-u64 zero-byte detector —
  different signature and semantics, not behavior-identical.
- `read_u64_le`'s byte-loop shape is load-bearing for codegen
  (penstock.rs:620-626 documents 6.3 vs 4.9 ns/frame measured
  2026-11-01); not a SWAR site.
- The escalation mask family (`ESCALATE_MASK`, penstock.rs:271) is
  D10-class admission semantics — out of scope per the lane doc.

Per the lane contract ("swap where the shipped arithmetic matches
cmov/swar primitives exactly; leave everything else untouched" and
"else BLOCKED-on-B2/B3"), the only lawful action was dep-only landing.

# Commands + exits (this session, this host)

- `cargo build -p cgap-branchless` → exit 0 (with one deprecation
  warning in swar.rs, owner B3's to fix, not mine).
- `cargo build -p cgap-gate` → exit 0 (with one manifest warning:
  unused-dep `cgap-branchless` — expected, the body swaps are deferred
  to B2/B3; the in-manifest comment documents the deferral).
- `cargo test -p cgap-gate` → exit 0. Full suite: 12 unit courts + 3
  lever_courts (l1_attestation_byte_flip_escalates_to_spoliation,
  l1_ceiling_law_flip_is_the_named_mutant,
  l1_prefilter_escalation_policy_court) + 0 doc-tests, all ok. The
  l1_* lever courts are the regression gate; all green.
- `cargo bench -p bench --bench chains-ingress` → exit 0.

# Bench numbers (chains-ingress, this run)

- ingress_chain 1M: fastest 10.47 ms = 9.99 ns/frame; median 11.23 ms
  = 10.71 ns/frame.
- Receipted baseline (w3-l1-ingress.md:63-70): fastest 10.20, median
  10.30 ns/frame.
- Verdict: zero-regression HOLD. The hot-path source is byte-identical
  (penstock.rs untouched), so any delta is environmental noise; median
  +4.0% single-run on a machine with concurrent lanes, fastest leg
  9.99 is at/below the receipted fastest. stage_penstock_admission
  median 8.87 ms/1M = 8.46 ns/frame (121-130 Mitem/s), comfortably
  under the <15 ns/frame target. No re-optimization attempted: with an
  unchanged binary the honest reading is noise, not regression.

# Standing + report

{"lane":"B7","files_written":["crates/cgap-gate/Cargo.toml",
"docs/jira/v26.11.2/waves/w6-b7-gate-consume.md"],
"commands":[
{"cmd":"cargo build -p cgap-branchless","exit":0},
{"cmd":"cargo test -p cgap-gate","exit":0},
{"cmd":"cargo bench -p bench --bench chains-ingress","exit":0}],
"courts_run":["penstock 12 unit courts","lever_courts l1_* x3"],
"standing":"PARTIAL",
"divergences":[
"dispatch named 'penstock stage-1 SWAR mask helpers and prefilter8
arithmetic' as swap sites; none match a shipped branchless primitive
exactly (prefilter8 xor-fold-multiply has no counterpart; is_zero_at is
variable-length early-exit vs fixed-u64 swar_has_zero_byte), so zero
body swaps — BLOCKED-on-B2/B3 per the lane contract; dep wiring landed,
suite green, bench within noise"]}
