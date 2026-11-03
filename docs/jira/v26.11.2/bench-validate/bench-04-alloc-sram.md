# Lane BENCH-04 — alloc sentry + SRAM truth validation (bench wave, 2026-11-02)

Repo /Users/sac/cgap @ main. Contract: read-only over crates/; sole file
written is this one. No git commands. Format follows
bench-validate/bench-01-ingress.md.

## §1 Claims under test + register rows

- Zero-alloc sentry (red-09 subject): CountingAlloc window over
  cgap-git ingress paths, positive control allocs=1/512 B, sentry window
  allocs=0/bytes=0. red-09 receipt: redteam/red-09-alloc-sentry-breach.md.
- SRAM envelope: baseline 36,048 B <= 131,072 B (CTQ-05); `--subject
  bn3` typed refusal 6,520,984 B = 49.75x ceiling (red-12,
  redteam/red-12-sram-overflow-exploit.md:59-87).
- bn3 fixpoint NOT zero-alloc: 2,662 allocs / 650,294,994 B measured
  in-window (red-09 W5 caveat 1, court lines 299-334).

## §2 Method actually applied

- Build root lease CARGO_TARGET_DIR=/tmp/f07-target (deleted at lane
  end). Build exit 0 (`cargo bench -p bench --bench chains-ingress
  --no-run`, 18.04 s, fresh lease target).
- Courts run with --nocapture to record FULL counts (dispatch: "0/0
  witnesses — record full counts").
- SRAM gate run twice: baseline + `--subject bn3`; output captured
  verbatim to /tmp/f07work/sram_baseline.out / sram_bn3.out; quoted in
  full below.
- Ambient loadavg1 at court/SRAM time: 8.53-10.71 (concurrent lanes);
  these courts are not latency courts — no contention guard applies.
- Commit SHA: NOT RECORDED (no-git contract; divergence).

## §3 RAW results (verbatim)

### §3.1 posh_alloc_sentry_court — EXIT=0, 1 passed; 0 failed (5.93 s)

```
POSH-4 HONEST FINDING: bn3 unify_to_fixpoint over 1293 window-derived
edges allocated 2662 times / 650294994 bytes (warmup, measured; not
folded into the zero claim; C03 pinned warmup allocation in
cgap-bn3/tests/edge_closure_courts.rs e5)
POSH-4 TRANSCRIPT: positive control allocs=1 bytes=512; sentry window
allocs=0 bytes=0 over 46442 relayed bytes; receipt root[0..4]=[208, 15,
81, 80]; fixpoint measured allocs=2662 bytes=650294994, batches=3
derived_rows=2929939
```

### §3.2 quota_alloc_purity_court — EXIT=0, 1 passed; 0 failed (0.00 s)

### §3.3 verify_cscb_sram.py baseline — EXIT=0, measured 36,048 B

Full gate text (verbatim, /tmp/f07work/sram_baseline.out):

```
toolchain: cargo 1.101.0-nightly (cad07cf8e 2026-10-23) | rustc
1.101.0-nightly (12433c307 2026-10-24) | target
riscv32imc-unknown-none-elf: installed
bn3 capacity table (static, from .../noalloc.rs): NA_MAX_ATOMS=8
NA_MAX_MATCHES=2048 NA_MAX_VARS=128 NA_MAX_ROWS=8192 -> NaMatches
2162696 B + NaEngine 4358288 B = 6520984 B
measured SRAM (riscv32imc-unknown-none-elf): 36048 B
[.text+.rodata+.data+.bss families]
empirical-baseline comparison (report-only, NOT gated): baseline 39184
B -> measured 36048 B (delta -3136 B); cite: v26.11.2 OVERNIGHT-LOOP
empirical riscv32imc freestanding witness (report-only; never a gate)
stack estimate (static analysis only): max leaf frame 2048 B (max
observed `addi sp,sp,-N` leaf-frame size over 9 sp-adjusting
prologues); UNVERIFIED as a worst-case chain bound
SRAM GATE: 36048 B <= 131072 B — csct/branchless kernel surface fits
the physical envelope (minimum entry surface; headroom 95024 B)
bn3 enclave surface (report-only here, gate under --subject bn3):
static state-table charge 6520984 B — in ceiling: NO — admission
REFUSED (typed gate failure, exit 1)
```

### §3.4 verify_cscb_sram.py --subject bn3 — EXIT=1 typed refusal

Full gate text (verbatim, /tmp/f07work/sram_bn3.out):

```
BN3_ENCLAVE_REFUSED: static noalloc state-table charge 6520984 B
(NaMatches 2162696 B + NaEngine 4358288 B) > 131072 B hard bound (49.75x
ceiling; red-12 topology). Admitting NaEngine+NaMatches into the SRAM
enclave is a typed gate failure.
```

## §4 Verdict per number

- **Sentry window 0/0: CONFIRMED** — allocs=0 bytes=0 over 46,442
  relayed bytes, positive control flipped (1/512). Full counts recorded.
- **Positive control: FLIPPED** (allocs=1/512 B) — anti-vacuity intact.
- **bn3 fixpoint: NOT zero-alloc, court-scoped OUT** — 2,662 allocs /
  650,294,994 B, disclosed by the court itself as the POSH-4 HONEST
  FINDING line. The zero claim is ring+seal, not the fixpoint.
- **SRAM baseline 36,048 B: CONFIRMED** — exit 0, headroom 95,024 B.
- **bn3 subject: TYPED REFUSAL CONFIRMED** — exit 1, 6,520,984 B =
  49.75x ceiling, exactly red-12's arithmetic.

## §5 Honest scoping caveats (restated from red-09 W5, verbatim sense)

1. The bn3 `unify_to_fixpoint` path allocates heavily (2,662 allocs /
   650,294,994 bytes measured this session) and is EXPLICITLY scoped
   out of the sentry's zero claim by the court itself
   (posh_alloc_sentry_court.rs:3-10, 299-334). The zero-alloc claim is
   a claim about ring+seal, not the fixpoint.
2. The FFI boundary has no CountingAlloc sentry; cgap-ffi host builds
   keep std by design, so cross-FFI zero-alloc is UNSUPPORTED
   (unaudited), not proven — the cross-FFI leg remains unaudited
   (red-09 W1/W5 caveat 2).

## §6 Command transcript (exits)

```
CARGO_TARGET_DIR=/tmp/f07-target cargo bench -p bench --bench \
  chains-ingress --no-run                      -> EXIT=0 (18.04 s)
cargo test -p cgap-git --test posh_alloc_sentry_court -- --nocapture \
                                               -> EXIT=0 (5.93 s)
cargo test -p cgap-git --test quota_alloc_purity_court -> EXIT=0
python3 scripts/verify_cscb_sram.py            -> EXIT=0 (36,048 B)
python3 scripts/verify_cscb_sram.py --subject bn3 -> EXIT=1 (refusal)
```

Anti-vacuity: the sentry's positive control flipped (1/512 B vs 0/0);
the SRAM gate's refusal arm flipped (exit 1 with typed reason). Both
directions witnessed this session.

Fences: none outside the two courts themselves — read-only lane; no
crates/ or scripts/ edits, so no regression surface introduced. Fence
named for the record: SRAM gate's bn3-pin mutant (--mutate bn3-pin,
R-15 W4 flip) was NOT re-run here — court coverage for the mutant is
R-15's receipt, not this lane's.

Standing: **ALIVE** for every number above — each names a command +
exit run this session against the exact subject. UNVERIFIED remains:
commit SHA (no-git contract).

```json
{"lane":"F-07/BENCH-04",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/bench-04-alloc-sram.md"],
 "commands":[
  "cargo bench -p bench --bench chains-ingress --no-run (exit 0)",
  "cargo test -p cgap-git --test posh_alloc_sentry_court -- --nocapture (exit 0, 0/0 window, control 1/512)",
  "cargo test -p cgap-git --test quota_alloc_purity_court (exit 0)",
  "python3 scripts/verify_cscb_sram.py (exit 0, 36048 B)",
  "python3 scripts/verify_cscb_sram.py --subject bn3 (exit 1, typed refusal 6520984 B = 49.75x)"],
 "courts_run":["posh_alloc_sentry_court (exit 0, positive control flipped)",
  "quota_alloc_purity_court (exit 0)",
  "verify_cscb_sram refusal arm (--subject bn3 exit 1 typed)"],
 "standing":"ALIVE",
 "divergences":[
  "commit SHA not recorded (contract bans all git commands)",
  "--mutate bn3-pin not re-run this session (R-15 W4 governs; this lane re-witnessed the refusal arm only)",
  "bn3 fixpoint 2,662 allocs / 650 MB witnessed again — court-scoped out of the zero claim (caveat 1, verbatim from red-09)",
  "cross-FFI zero-alloc leg remains unaudited (cgap-ffi keeps std by design; red-09 caveat 2)"]}
```
