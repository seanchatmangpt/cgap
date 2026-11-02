# gate-4-spoliation

Status: ALIVE — Morning DoD Gate 4 witnessed first-hand this session
(coordinator-written from gate-runner GAP-4's verdict; that lane executed all
commands but wrote no marker per read-only discipline).

- Run: `cargo test -p cgap-p28 -- --nocapture --test-threads=1` → exit 0,
  42 passed / 0 failed (36 unit + 6 integration).
- 640-bit sweep: root + derivative, 640 executed / 629 refused / 11 tolerated
  each; survivors == the predicted D-A17-1 CLEAR-flip class
  (popcount(5_000_000_000) = 11, re-derived this session).
- Vacuity: three distinct probes (neutered trap → 640; truncated → 100;
  reject-all → 0) all distinguishable from the 11-pin.
- Spoliation halt: stripped/truncated carriers → real child SIGABRT,
  stdout `staged=0x503238ff`; totals 2560 mutations / 2516 real deaths.
- Liability: f1 boundary at the real frozen ceiling (5_000_000_000 pass /
  +1 cent → emit 5_000_000_001, latched); f2 proof self-attests (168-byte
  round trip, fresh-sealer recompute, flipped-byte negative control fails).
- Boundary: `_cgap_verify_p28` present in fresh cdylib (nm -gU).
- Three-lane convergence on the 11-law: A17, F2-5, A18 (independent
  re-derivations, same predicted class).
- Note: pre-existing `unused_mut` warning asc450.rs:1881 (lib-test profile,
  not a failure).
