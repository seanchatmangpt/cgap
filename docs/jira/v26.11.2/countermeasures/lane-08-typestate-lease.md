# Lane AE8 — type-state lease guard (scoped feasibility)

Standing: **ALIVE** (courts ran + passed this session, exits cited below).

## Frozen-semantics analysis (§5 pin, A16 / SP3)

The dispatch proposed an `ActiveLease<Validated/Expired>` affine refactor of
`Lease` itself. Verdict: **DESIGN-TARGET** — not implemented, requires the
§5 owner's decision. Exact semantic deltas the refactor would introduce:

1. **Exhaustion: panic → typed.** §5 fixes exhaustion as a TRAP
   (`lease.rs:118-124`); SP3's typed `LeaseBoundRefusal` ledger already
   provides the typed boundary at the cgap layer. Converting the trap into
   a typed refusal inside the crate would soften the §5 trap discipline
   and change the pinned law.
2. **Tick mutability surface.** `Tick` is a private monotonic newtype with
   no public constructor from a raw value (`lease.rs:30-36`); an affine
   state machine on `Lease` itself wants consumption-by-value (`fn
   tick(self) -> (u64, ActiveLease<...>)`) — a public surface change to the
   frozen type.
3. **`retire`-style state insertion.** Any `Fresh/Spent` parameter on
   `Lease` changes the frozen public surface (`Lease::new`, `tick`,
   `current_tick`, `phi`) that the wave-6/7 shim battery pins byte-for-byte
   (M1/M2/M3d flip receipts cited in `crates/cgap-crypto/tests/lease_courts.rs:10-18`).

## Landed: additive wrapper (compatible subset)

`crates/cgap-crypto/src/lease.rs` gained an additive `pub mod guard`
(lease.rs:220-330 region; no frozen line touched — `git diff` is +276/−3,
the 3 "deletions" being a rustfmt reflow of one §3.2 layout assert with
identical semantics). The guard:

- `LeaseGuard<Fresh>` / `LeaseGuard<Spent>` via `mint` → `spend_tick` →
  `retire` → (terminal) `release`.
- All tick semantics DELEGATE verbatim to the frozen `Lease::tick`:
  exhaustion still TRAPS; the guard does not soften it.
- No `rewind`/`to_fresh`/`Default`/`Into<LeaseGuard<Fresh>>` on
  `LeaseGuard<Spent>` — state inversion is unrepresentable at the type
  level (the AE8 law).

## Compile-rejection court (E0599 witness)

`lease::guard_courts::guard_compile_rejection_state_inversion_is_unrepresentable`
(`crates/cgap-crypto/src/lease.rs`, `#[cfg(test)]` module) compiles a REAL
rustc subprocess against the built crate artifacts:

- BASELINE: `spent.rewind()` against the real `cgap_crypto` — MUST be
  rejected with an E0599-class signature. Manual witness this session:
  `error[E0599]: no method named rewind found for struct LeaseGuard<S> ...
  method not found in LeaseGuard<Spent>` (rustc against
  `/tmp/ae8-target/debug/build/cgap-crypto/.../out/libcgap_crypto-*.rmeta`).
- CONTROL (anti-vacuity): the identical call shape against an in-file
  stand-in that HAS `rewind` MUST compile. Flip witness this session:
  `FLIP-WITNESS: rewind-shaped call COMPILES when the API exists (exit 0)`.
  If the guard ever grew a rewind, BASELINE compiles and the court's
  `AE8 VACUOUS GATE` assert fires.

## Runtime courts

- `guard_spend_sequence_is_monotonic_delegated` — strictly increasing
  pre-advance values through the guard, read-through agreement, `retire`
  preserves the tick, `release` hands back a frozen `Lease` with the
  advanced tick (same M1 anti-vacuity class as the wave-7 monotonicity
  court).
- `retire_consumes_and_terminal_state_has_no_tick_path` — affine-by-value
  transition; `retire` itself spends no tick.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/ae8-target cargo test -p cgap-crypto` → **exit 0**
  (4 lib tests incl. 3 guard courts, + lease_courts/chatman_courts suites;
  all `test result: ok`).
- Manual rustc mutant witness (E0599) and flip witness (exit 0) — commands
  above, both observed.

## Divergences

1. `docs/jira/v26.11.2/countermeasures/` did not exist — created for this
   receipt (dispatch named the path).
2. The dispatch named `lease.rs` as the only code file; crate `lib.rs` is
   contract-off-limits, so the additive `guard` module + courts live inside
   `lease.rs` rather than a new `lease_guard.rs` (which lib.rs could not
   re-export).
3. Dispatch's `ActiveLease<Validated/Expired>` on `Lease` itself:
   DESIGN-TARGET (see analysis); the landed wrapper is the compatible
   subset.
4. Workspace target layout is nonstandard (artifacts under
   `build/<crate>/<hash>/out/`, not `debug/deps/`); the court's artifact
   finder walks up from the test binary accordingly. The compile court
   type-checks the mutant via `--emit=metadata` + `--extern` (crypto
   `.rmeta`, blake3 `.rlib`) — a metadata-stub rlib is not linkable, so a
   full-link mutant compile is out of reach in this layout (recorded, not
   silent).
