# dod-csct-cscb

DoD audit of crates/csct + CSCB coordinates C1-C8 (EXP-09, 2026-11-02).
All commands run this session against the exact on-disk subjects.

## 6-stage pipeline: ALIVE

- cargo test (crates/csct, /tmp lease target): exit 0, 65 passed / 0 failed.
  - lib 41 (andon 6, warmer 2, reframe 4, drowning 3, emotional 3,
    new_way 3, solution 3, mutation 4, receipt 4, types 5,
    drowning_benchmark 4).
  - bin tests: csct_runner 7; cscb bins carry 1 (scan.rs cfg(test) via
    #[path] include).
  - integration: csct_coverage_court 9, csct_pipeline_court 7.
- Release runner: exit 0, verdict PASS, roi 352x, receipt root
  fa2e7db5016d7e919a9dc5ec1e2b87d74b5cf9507f435834d8443674ec466c7a.
- G14 vacuum-fold barrier confirmed live (solution.rs:105-121):
  black_box(frames) + per-iteration black_box(filter_frame);
  release receipt shows 1.19/10 ns/frame (real, not folded 0.00042).

## Ledger math: VERIFIED

- drowning.rs:61-62 u64 cents: 42_238_600 c/mo; annualized 5_068_632.
- drowning_benchmark.rs:19-33: SUT 1_440_000 c/yr, bleed 506_863_200 c,
  net 505_423_200 c, gross factor 351 (floor of 351.99), asserted.
- REQUIRED_ROI_X = 350.0 gross formula (lib.rs:30-35 coordinator
  ruling, errata csct-lane-01-scaffold.md; verdict lib.rs:79-81).
- Anti-vacuity mutants green: drowning.rs:129-144;
  drowning_benchmark.rs:72-85 (zeroed spark flips both arms).

## CSCB C1-C8 classification

| coord | bin | gate class | status |
|---|---|---|---|
| C1/C2 | cscb_c1_c2_ingress | hard release SLA (1.2M fps, 10 ns p99, BATCH=64 disclosed) | PASS standalone; ladder run showed one timing flake at p99=10 ns gate (honest variance) |
| C3 | cscb_c3_closure | hard 25.0 ms SLA, N=7 min | DEFECT-OPEN: honest FAIL 796 ms vs 25 ms (B02; repair-20 corroborates 24-25 ms on other corpus) |
| C4 | cscb_c4_alloc | typed-fail breach court (budget (0,0), RECORDED breach) | PASS (0 allocs / 0 bytes; Box::new(42) mutant must breach) |
| C5 | scripts/verify_cscb_sram.py | hard 131,072 B bound; mutant under bound = VACUOUS GATE | PASS 36,048 B; flip at 136,068 B |
| C6 | cscb_c6_retraction | honest typed FAIL gate (800M fps) | DEFECT-OPEN: 242M facts/s min (B05; GIT-11 323.8M ungated); integrity court PASS |
| C7 | cscb_c7_receipt_sealing | 1M/s median floor + chain re-derivation + bitflip | PASS (~4.35M/s median) |
| C8 | drowning_benchmark | u64-cents court, gross >= 350 | PASS |
| runner | cscb_runner | strict order, typed SubjectMissing skip | ALIVE; TRAP: `cargo run --bin` alone builds no subjects — `cargo build --release` first |

Ladder exits witnessed: exit 1 with C3/C6 failing (honest), roots
a9b75a72... / 2e8c57cf... across runs.

## Standalone manifest: KEEP STANDALONE

- csct/Cargo.toml own [workspace], disclosed mirror of crates/cli
  precedent; root members marked PROPOSED (root Cargo.toml:19-20,23).
- Rationale: disclosed precedent, identical path-dep builds, CSCB bins
  deliberately #[path]-include siblings during the compile freeze,
  membership change needs a coordinator ruling.
- Revisit after release when root members leave PROPOSED.

## Blocking gaps

1. C3 closure SLA honest FAIL (796 ms vs 25 ms) — claim vs corpus
   mismatch needs owner ruling (B02).
2. C6 retraction SLA honest FAIL (242-248M vs 800M fps) — either gate
   re-scoping or kernel work (B05/GIT-11).
3. CSCB_MUTATE=zero-tax aggregate flip court FAIL (B08) — no passing
   child refuses under the mutation.
4. C1/C2 p99 timing flake under concurrent load — disclosed method
   sensitivity, not a code defect.

## See Also

audit/README.md · audit/audit-ledger.md · lanes/csct-wave-register.md


---

## ERRATA (2026-11-02, TC-08 / cull-08)

Line 35, C1/C2 row: "one timing flake at p99=10 ns gate (honest
variance)" understates the evidence. doctrine-commercial-proofs.md:29
records C2 FAILing the 10 ns gate **3/3 direct runs** (17-21 ns) —
that is host-load variance, not a one-off flake. Equally, a fresh
quiet-host direct run this session (2026-11-02, release,
cscb_c1_c2_ingress, batch64-amortized-instant method disclosed in
[META]) measured **C2 p99 5.00 ns -> PASS exit 0** (log:
/tmp/tc08-c2-direct.log; run witnessed by lane receipt
lanes/lane-tc-08-errata.md). The honest classification is variance
across host-load states (PASS quiet-host, FAIL 3/3 under load), not a
flake; the 10 ns gate is host-load-sensitive on this machine and any
sub-10ns claim must carry the batch64-amortized method + quiet-host
conditions (doctrine-commercial-proofs.md:73). Row retained unedited.
