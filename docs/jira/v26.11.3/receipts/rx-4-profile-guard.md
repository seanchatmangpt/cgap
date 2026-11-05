# RX-4 — profile_guard: reusable release-profile detector + anti-DCE fence

- **Lane**: RX-4 IMPLEMENTATION, milestone v26.11.3, repo /Users/sac/cgap
- **Deliverable**: `crates/cgap-aoq/src/profile_guard.rs` (new, registered in
  `src/lib.rs`), unit courts, two disclosed migrations, this receipt.
- **Standing**: ALIVE (commands + exits below, run this session)

## 1. Component

Generalizes two landed patterns into one reusable, no_std, zero-dep,
`forbid(unsafe_code)`-clean module:

- `assert_release() -> Result<(), ProfileViolation>` (`#[must_use]`) —
  build-time detection on two axes: `cfg!(debug_assertions)` plus
  `option_env!("OPT_LEVEL")`. `OPT_LEVEL=None` reads as UNKNOWN on that
  axis (not a violation by itself); detection falls to the
  debug-assertions axis — never a fake pass.
  The `#[must_use]` Result encodes AF-25's loud-SKIP caller pattern: a
  dropped guard is a compile error, not a silent debug-profile fence.
- `fence<T>(value: T) -> T` — `core::hint::black_box` wrapper (AF-10's
  G1b release anti-DCE fix, generalized). Stable, safe, no_std.

`ProfileViolation` is `Copy` + `Display` (`write_reason`) for loud-SKIP
context strings.

## 2. Disclosed migrations (cross-lane file edits)

- **AF-25 G2 axis-2 guard** (`tests/cbmqs_gates.rs`, G2 gate): inline
  `if cfg!(debug_assertions)` loud-SKIP swapped one-for-one to
  `if let Err(v) = cgap_aoq::profile_guard::assert_release()`, violation
  typed
  into the SKIP reason. Witnessed firing with the composed component:
  `G2 ... SKIP release-profile fence only (AF-17 axis 2; non-release
  profile: debug_assertions=true opt-level=unset); context: amortized-min
  748.1 ns [FENCED...]`.
  (Note: cross-lane minimal edit, disclosed per SLA law; owner supersedes
  at integration.)
- **AF-10/AB-19 G1b anti-DCE** (`tests/cbmqs_gates.rs`, G1b gate): the two
  inline `std::hint::black_box` calls swapped to
  `cgap_aoq::profile_guard::fence(...)` — identical semantics.

## 3. Gates (real output, this session)

| Gate | Command | Result |
|---|---|---|
| lib courts debug | `cargo test -p cgap-aoq --lib` | 14 passed; 0 failed (5 profile_guard courts green) |
| lib courts release | `cargo test -p cgap-aoq --lib --release` | 14 passed; 0 failed |
| CBMQS debug | `cargo test -p cgap-aoq --test cbmqs_gates` | 1 passed; 0 failed (2.88s) |
| CBMQS release | `cargo test -p cgap-aoq --test cbmqs_gates --release` | 1 passed; 0 failed (0.22s) |
| no_std embedded surface | `cargo check -p cgap-aoq --target riscv32imc-unknown-none-elf` | Finished, 0 errors |

- Guard-fires-in-debug witnessed: `guard_axis_matches_build_profile`
  asserts Err in the debug run, Ok in the release run — both runs green.
- Fence blocks constant-folding: `fence_blocks_constant_folding` court
  routes a foldable constant through `fence` and consumes it in a branch
  on fence output — the fence is live code, output identity preserved.
- **Pre-existing failures, not this lane's**: full `cargo test -p
  cgap-aoq` currently fails to COMPILE in untracked in-flight test files
  (`rfc0089_certificate.rs`, `macro_p99_torture.rs`,
  `macro_saturation_soak.rs` — untracked, other lanes'). Lib and
  cbmqs_gates targets pass green ×2.

## 4. Falsifier status

- "Guard fires in debug / passes in release" — witnessed both ways (cfg
  inverse courts, real runs above).
- "Fence prevents constant-folding" — witnessed at the court level
  (live-code branch on fence output); the historical 11.08B pushes/s
  defect is the domain witness (AF-10/AB-19 receipts).
- UNKNOWN: no runtime perf delta measured for the swaps themselves
  (semantics-identical swaps; CBMQS G1b/G2 numbers unchanged in kind:
  debug G2 context 748.1 ns, matching the prior receipt's ~743.5 ns).

result: UNKNOWN
