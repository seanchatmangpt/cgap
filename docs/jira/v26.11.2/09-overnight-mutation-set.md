# 09-overnight-mutation-set

Consolidated anti-vacuity mutation register for the v26.11.2 overnight loop.
Sources (cited, not redefined): `docs/jira/v26.11.2/01-release-gates.md`
(FLIP blocks, G1-G8) and `docs/jira/v26.11.2/02-pareto-levers.md`
(per-lever mutations, L1-L5). No invented mutations - every row here is
named in one of those two documents.

## Overnight-loop mechanics (applies to every row)

1. The loop applies each mutation to a `/tmp` sandbox copy of the subject
   tree/file - never the shared tree.
2. The owning gate/court must FLIP (exit nonzero or the named typed error).
   An unflipped mutation is `VACUOUS-GATE`: the loop exits nonzero.
3. Restore is sha256-exact AND the restored file's mtime is bumped (or
   `cargo clean -p <crate>` issued) before any rebuild - byte-exact restore
   alone leaves cargo's fingerprint stale and the mutated binary survives
   (`docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184`).
4. Skips are loud: `exit 3` + `[SKIP] <reason>`; a skip is never a pass.
5. Exit-code contract: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.

## Register: G-gates (01-release-gates.md FLIP blocks)

| ID | Subject (file:line) | Mutation | Expected kill signal | Owning gate/court | Flip status |
|---|---|---|---|---|---|
| G1-M1 | one publishable crate Cargo.toml (copy) | Strip `description` | release_check on the copy FAILs (exit 1) | G1-PUBLISH / `scripts/release_check.py` | [UNFLIPPED-t26.11.2] |
| G2-M1 | bench kernel hot loop, per level L1-L5 (copy) | Remove `#[inline]` / inject bounds check / add `core::hint::black_box` around hot loop | Level perf threshold flips to FAIL | G2-PARETO / criterion + `scripts/divan2receipt.py` | [UNFLIPPED-t26.11.2] |
| G3-M1 | mat closure kernel, crates/bench (copy) | Skip one derivation rule or de-duplicate one join arm | Changed derived-fact count detected, or throughput threshold flips | G3-MAT-BENCH / mat_closure_bench | [UNFLIPPED-t26.11.2] |
| G4-M1 | DRED increment path, crates/bench (copy) | Skip the entropy recompute | Entropy-variance number changes; <0.15 ns variance gate flips | G4-DRED / dred_bench | [UNFLIPPED-t26.11.2] |
| G5-M1 | no_std subject, riscv32imc (copy) | Add a `Vec::new()` push (alloc symbol) or a `panic!` site | `scripts/symbol_gate.py` FAILs on the copy | G5-SILICON / symbol_gate.py | [UNFLIPPED-t26.11.2] |
| G6-M1 | a workspace Cargo.toml (copy) | Append `datafusion = "42"` | Gate on the copy FAILs | G6-NO-DF / grep -ri datafusion gate | [UNFLIPPED-t26.11.2] |
| G6-M2 | real temp-copy restore | Self-check corrupts a restore | Gate refuses the restore mismatch (fail-closed) | G6-NO-DF self-check | [UNFLIPPED-t26.11.2] |
| G7-M1 | patent claims.txt (copy) | Merge one claim with a prior-art mechanism (near-duplicate) | Verdict drops to 3/4, gate FAILs | G7-PATENT / verify_ost_design_around.py | [UNFLIPPED-t26.11.2] |
| G8-M1 | mapping table (copy) | Remove one of the 11 mappings | Census exits nonzero AND golden digest breaks | G8-STARDF-F1 / stardf_f1_census + stardf_golden + kd_flip KD-12/KD-14 | [UNFLIPPED-t26.11.2] |

## Register: L-levers (02-pareto-levers.md per-lever mutations)

| ID | Subject (file:line) | Mutation | Expected kill signal | Owning gate/court | Flip status |
|---|---|---|---|---|---|
| L1-M1 | ingress stage fold (cgap dam FrameView chain) | Corrupt a slot byte or stage code | Per-stage checksum assert (vs untimed preflight fold) fires; escalation still refuses the spoliated frame | L1 ingress court (P3-16 chain bench) | [FLIPPED-t26.11.2 variant] escalate-always-true injection flipped the same court (`waves/w3-l1-ingress.md:75-83`, /tmp sandbox, sha256-exact restore); the slot-byte form itself remains unexecuted |
| L2-M1 | committed page window byte, cgap-parquet (copy) | Flip one committed window byte before `verify_seal` | `ChecksumMismatch { region: Body }` -> exit 101 | L2 parquet seal court (P3-17) | [FLIPPED-t26.11.2] `(w>>24)`->`(w>>25)` shift-corruption mutant: court FAILED exit 101, sha256-exact restore re-green (`waves/w3-l2-swar-fnv.md:38-42`) |
| L3-M1 | BN3 unify comparator, crates/cgap-bn3/src/unify.rs:11,35-39,274 | Binding-alias comparator swap (doe-4 SHIM-DOE4 class) | Differential oracle assert (match count == raw-word O(N) hash join, no unify code in oracle path) flips; M12-7 source_mask popcount assert rides along | L3 BN3 unify court (P3-18 + doe-4) | [UNFLIPPED-t26.11.2] |
| L4-M1a | committed header byte, cgap-ffi sovereign commit | Corrupt one committed header byte (refusal class) | Per-stage `CgapStatus` asserts fire | L4 sovereign commit court (P3-19) | [UNFLIPPED-t26.11.2] |
| L4-M1b | slab write in the commit path | Drop a slab write | Per-stage `CgapStatus` asserts fire | L4 sovereign commit court (P3-19) | [UNFLIPPED-t26.11.2] |
| L4-M2 | P28 digest byte | Flip one digest byte | Exactly `CGAP_ERR_P28_SPOLIATION` (P28 spoliation negative control) | L4/P28 spoliation negative control | [UNFLIPPED-t26.11.2] |
| L5-M1a | upstream receipt | Bit-flip an upstream receipt | validate refuses | L5 seal chain court (P3-19/BENCH-4) | [UNFLIPPED-t26.11.2] |
| L5-M1b | crossing | Demote a crossing | No proof emission - `proven == N/2` assert flips | L5 seal chain court | [FLIPPED-t26.11.2] W3L5-DEMOTE: exit 101, proven==N/2 fired (511 vs 512), root+checksum diverged (`waves/w3-l5-fused.md:44-52`) |
| L5-M1c | receipt digest | Flip a receipt digest | verify returns DigestMismatch | L5 seal chain court | [FLIPPED-t26.11.2] W3L5-DEMOTE: exit 101, proven==N/2 fired (511 vs 512), root+checksum diverged (`waves/w3-l5-fused.md:44-52`) |

## Flip-status legend and prior witnesses (same mutation CLASS, v26.11.1)

Rows default [UNFLIPPED-t26.11.2] unless marked FLIPPED. As of 2026-11-02,
three register rows carry a witnessed v26.11.2 flip (L1-M1 variant, L2-M1,
L5-M1b; cites in-table); the rest remain unexecuted against the v26.11.2
tree. The wave witness section below registers the 2026-11-02 hardening/code
wave's own anti-vacuity flips. Prior same-class witnesses (citation only,
not a v26.11.2 flip):

- L2-M1 class (committed byte flip -> exit 101): witnessed P3-17,
  `docs/jira/v26.11.1/waves/bench-chains-parquet.md:73-75` ("COURT FLIPS").
- L3-M1 class (SHIM-DOE4 comparator swap): witnessed
  `docs/jira/v26.11.1/waves/a12-finish-receipt.md:87-99` (FLIP rc 101) and
  `docs/jira/v26.11.1/waves/a12-wave8-audit-receipt.md:63` (M12-7 popcount
  court flips under projector dedupe).
- L5-M1 class (bit-flip / demote / digest-flip triple): named and executed
  in `docs/jira/v26.11.1/waves/bench-chains-seal.md:57-58`.
- L4-M2 class (flipped digest byte -> spoliation): witnessed
  `docs/jira/v26.11.1/waves/gate-4-spoliation.md:15-18` (2560 mutations /
  2516 real deaths; flipped-byte negative control fails) and
  `docs/jira/v26.11.1/waves/a17-wave2-audit-re-audit.md:63-69`.
- G5-M1 class (symbol-gate flip): `docs/jira/v26.11.1/waves/gate-5-silicon.md`
  records ELF size-law flips (R11), same gate family.
- G-gates G1/G3/G4/G6/G7/G8 FLIP blocks: no v26.11.1 same-class witness found
  in waves/ this session; UNVERIFIED, remains [UNFLIPPED-t26.11.2].

## Wave witnesses (2026-11-02 hardening/code wave)

Every row below is a mutation actually executed this session against a /tmp
sandbox (never the shared tree), with a fail-closed restore, cited to the
executing lane receipt. Sources: lane receipts, not 01/02; this section
extends the register beyond the G/L docs.

- **WV-bn3-pin-1** (FLIPPED)
  subject: bn3 charge model in scripts/verify_cscb_sram.py. mutation: NA_MAX_MATCHES 2048->1024 in a
    sandboxed noalloc.rs copy. flip: --mutate bn3-pin court trips the red-12 size_of pin check (exit
    1, DIVERGED, "consts drifted?").
  cite: `lanes/lane-fix02-bn3-sram-gate.md:45-47,59-60`
- **WV-bn3-pin-2** (FLIPPED)
  subject: cgap-bn3 init-time capacity bound. mutation: NA_MAX_MATCHES 2048->4096 in the trimmed
    /tmp/rc03-sandbox (full-repo copy aborts on a vendor/nanoarrow symlink loop; recorded
    divergence). flip: capacity_bound_court 1 passed / 2 failed; charge pin + refusal courts flip
    (right: 2162696 vs doubled actual).
  cite: `lanes/lane-rc03-bn3-capacity-bound.md:54-60`
- **WV-sram-bloat** (FLIPPED (pre-existing B04 court, re-witnessed))
  subject: SRAM gate baseline subject. mutation: static 100 KB blob injected. flip: measured 136068
    B > 131072 B bound; gate flips.
  cite: `lanes/lane-fix02-bn3-sram-gate.md:48-49,62-63`
- **WV-sram-shim** (FLIPPED + restore-detection non-vacuous)
  subject: SRAM gate subject admission. mutation: bn3 injected as default enclave subject in a /tmp
    gate copy. flip: gate pass -> exit 1 BN3_ENCLAVE_REFUSED; corrupted-restore probe detected by
    sha256 mismatch.
  cite: `lanes/lane-fix02-bn3-sram-gate.md:50-55,63-65`
- **WV-quota-M1** (FLIPPED)
  subject: ingress RefusalQuota self-clear. mutation: level-triggered self-clear reverted. flip:
    andon court FAILED, 2 legs flip.
  cite: `lanes/lane-r-adv12fix-quota-repair.md:38-44`
- **WV-quota-M2** (FLIPPED)
  subject: ingress window_refusals. mutation: saturating_add -> +=. flip: saturation leg FAILED
    (attempt to add with overflow).
  cite: `lanes/lane-r-adv12fix-quota-repair.md:38-44`
- **WV-quota-M3** (FLIPPED)
  subject: egress RefusalQuota self-clear. mutation: level-triggered self-clear reverted. flip:
    quiet-window + egress flood legs flip (2).
  cite: `lanes/lane-r-adv12fix-quota-repair.md:38-44`
- **WV-quota-M4** (FLIPPED)
  subject: egress window_refusals. mutation: saturating_add -> +=. flip: saturation leg FAILED
    (overflow).
  cite: `lanes/lane-r-adv12fix-quota-repair.md:38-44`
- **WV-andon-esc-1** (FLIPPED)
  subject: ingress escalation path. mutation: note_refusal escalation deleted (/tmp/fix04-sandbox2).
    flip: andon court 2 failed / 1 passed, exit 1.
  cite: `lanes/lane-fix04-andon-quota.md:41-42`
- **WV-andon-esc-2** (FLIPPED)
  subject: egress escalation path. mutation: note_refusal escalation deleted (same sandbox). flip:
    andon court 1 failed / 2 passed, exit 1; targeting precise (egress mutant fails only the egress
    court).
  cite: `lanes/lane-fix04-andon-quota.md:41-48`
- **WV-breach-M1** (FLIPPED)
  subject: pack_demux poison leg. mutation: Phase::Done => {} revert. flip: breach_regression_court
    breach3 exit 101 (post-trailer poison silently absorbed).
  cite: `lanes/lane-r14-breach-regression-court.md:56`
- **WV-breach-M2** (FLIPPED)
  subject: ingress quota escalation. mutation: note_refusal escalation deleted. flip: breach4 exit
    101 (1M flood did not escalate the quota).
  cite: `lanes/lane-r14-breach-regression-court.md:57`
- **WV-breach-M3** (FLIPPED)
  subject: bn3 init_within typed refusal. mutation: refusal deleted. flip: breach2 exit 101
    (init_enclave admitted).
  cite: `lanes/lane-r14-breach-regression-court.md:58`
- **WV-breach-M4** (FLIPPED)
  subject: fa9 canonical_key. mutation: raw-text key + identity_key -> None (full pre-fix shape;
    canonical_key alone no longer flips under ADV-10 inode keying -- two-layer protection, D5).
    flip: breach1 exit 101 (alias race admitted).
  cite: `lanes/lane-r14-breach-regression-court.md:59,102-108`
- **WV-breach-M5** (FLIPPED)
  subject: SRAM gate budget. mutation: SRAM_BUDGET neutralized to 100000000. flip: breach5 exit 101
    (gate admits bn3 exit 0 -> typed exit-1 assert fires).
  cite: `lanes/lane-r14-breach-regression-court.md:60`
- **WV-mac-shim-1** (FLIPPED)
  subject: p28 attestation mac_subkey. mutation: derives with the zero key instead of the device
    key. flip: attestation_mac_court FAILED 2/4.
  cite: `lanes/lane-rc04-attestation-mac.md:68-71`
- **WV-mac-shim-2** (FLIPPED)
  subject: p28 verify_with_key. mutation: constant-time tag compare skipped (always Ok). flip:
    attestation_mac_court FAILED 3/4 (first attempt all-pass = vacuous stale-binary run, superseded
    on the verified on-disk mutation; recorded, not hidden).
  cite: `lanes/lane-rc04-attestation-mac.md:72-78,102-104`
- **WV-fleet-ev** (FLIPPED)
  subject: fleet.rs RMA evidence checks. mutation: evidence checks removed (/tmp/rc13-fleetops-
    sandbox). flip: court2 exit 101.
  cite: `lanes/lane-rc13-fleet-ops.md:33-35`
- **WV-fleet-th** (FLIPPED)
  subject: slo.rs AlertPredicate. mutation: > -> >= threshold swap. flip: court4 exit 101.
  cite: `lanes/lane-rc13-fleet-ops.md:36-37`
- **WV-ocel2-gate** (FLIPPED)
  subject: ocel2 projector verify gate. mutation: verify gate in Ocel2Projector::project removed
    (let _ = (chain, blocks, license)). flip: tamper_court 3 passed / 2 failed, exit nonzero;
    license_tamper_refuses_projection + shape_verdicts_are_typed flip; tamper_one_link still passes
    via the independent per-block digest cross-check (defense in depth, disclosed).
  cite: `lanes/lane-ocel01-cgap-ocel2-adapter.md:60-77`
- **WV-zstr-T17** (FLIPPED)
  subject: zero-string gate debris path. mutation: debris try/except -> if False at 2 sites
    (/tmp/rc08_mutant_copy.py). flip: mutant leg A scan raised sections-unreadable rc=-1, NOT a loud
    skip; leg B unchanged; T17/debris legs flip.
  cite: `lanes/lane-rc08-zero-string-debris-skip.md:54`
- **WV-lint-key** (FLIPPED)
  subject: receipt_lint strict guard. mutation: guard stripped in a /tmp copy emitting a key-less
    finding. flip: key_crash_court arm 2 (no typed row -> court REFUSES, anti-vacuity); arm 1 flip:
    typed RECEIPT-LINT-MISSING-KEY row, exit 1, no traceback.
  cite: `lanes/lane-rc09-receipt-lint-keycrash.md:27-34,49-52`

All rows FLIPPED; zero skips in this section; every restore sha256-exact
with mtime bump (or sandbox-only, where the shared tree was never mutated).
Mutants NOT yet witnessed this wave: G1-M1, G2-M1, G3-M1, G4-M1, G5-M1,
G6-M1/M2, G7-M1, G8-M1, L3-M1, L4-M1a/M1b/M2, L5-M1a/M1c — stay
[UNFLIPPED-t26.11.2], TARGET-fenced until the overnight loop executes them.

