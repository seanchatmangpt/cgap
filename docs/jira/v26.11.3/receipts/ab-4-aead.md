# Receipt: AB-4 AEAD/Poly1305 kernel (PG-A3, AoQ Wave-1 W1-3b)

| Field | Value |
|---|---|
| Subject | /Users/sac/cgap @ working tree (lane-local edits, no git ops run) |
| Lane | AB-4, AoQ Wave 1, charter: PG-A3 poly1305.rs + aead.rs + courts |
| Standing | PARTIAL_ALIVE (all owned gates green this session; coordinator owns integration) |

## Diff (owned files)

- `crates/cgap-crypto/src/poly1305.rs` — NEW (250 lines): poly1305-donna
  32-bit-limb Poly1305 (`new`/`push`/`finish`, `one_shot`), no alloc, no
  deps, constant-time finish (branchless mask select between `h` and
  `h - p`), donna `final`-flag partial-block law (hibit 0 on the padded
  tail block).
- `crates/cgap-crypto/src/aead.rs` — NEW (352 lines): detached
  AEAD_CHACHA20_POLY1305 (RFC 8439 §2.8). `AAD_LEN=36`, `TAG_LEN=16`,
  `Nonce96` `#[repr(C, packed(4))]` 12-byte size-pinned (const asserts;
  hand-written by-value trait impls — derives borrow packed fields and
  are a compile error by design), SOLE constructor `Nonce96::new`
  rejecting epoch `0xFFFF_FFFF` (`EpochExhausted`), `seal_detached` /
  `open_detached` with folded constant-time tag compare, RFC §2.8
  mac_data layout (no alloc: chunked `push`), counter 0 = Poly key
  block, message blocks from counter 1. `#[doc(hidden)]
  mac_tag_var_aad` seam for the 12-byte-AAD RFC vector.
- `crates/cgap-crypto/src/lib.rs` — TWO lines: `pub mod aead;` +
  `pub mod poly1305;` (AB-3's `chacha`/`fips202`/`fips203` lines
  untouched).
- `crates/cgap-crypto/Cargo.toml` — one dev-dep:
  `chacha20poly1305 = { version = "0.10", default-features = false,
  features = ["alloc"] }` (dev-only; lib surface stays zero-dep).
- `crates/cgap-crypto/tests/aead_courts.rs` — NEW: 8 courts (below).
- `crates/cgap-crypto/tests/ct_scan_court.rs` — NEW: kernel hygiene scan
  (zero `unsafe` outside the forbid attr / zero `asm!`/`core::arch` over
  poly1305.rs + aead.rs + lib.rs) with 3 sandbox-add-token kills +
  byte-exact restore witness.

## Courts (8 + 1, all witnessed green this session)

1. `poly1305_rfc8439_2_5_2_fixture` — RFC 8439 §2.5.2 (tag
   `a8061dc1…27a9`), one-shot + push-chunking equivalence (1/3/16/17/34/100).
2. `aead_rfc8439_2_8_2_fixture` — RFC 8439 §2.8.2 published vector through
   the var-AAD seam: published mac_data over published ct -> published tag
   `1ae10b59…0691`; wire nonce layout pin (`07 00 00 00 || 40..47`).
3. `cross_check_rustcrypto_byte_identical_both_directions` — REAL
   chacha20poly1305 crate, lengths 0/1/15/16/17/31/64/65/255/4096: ct AND
   tag byte-identical; local open of RustCrypto output; RustCrypto open of
   local output (attached ct‖tag form).
4. `length_sweep_round_trips` — 0/1/15/16/17/35/36/511/4096 round-trips.
5. `flip_kills_are_mismatch` — AAD flips (first/mid/last), ct flips
   (block head/mid/tail), tag BIT flips (first/last byte) all ->
   `Err(Mismatch)`; pt flip changes ct+tag; untampered still `Ok(())`
   (court not trivially red).
6. `epoch_exhausted_boundary` — `0xFFFF_FFFE` lawful (constructor Ok +
   full seal/open at the boundary epoch); `0xFFFF_FFFF` -> `EpochExhausted`.
7. `sole_constructor_count_law_scan` — zero `Nonce96 { epoch…}`
   construction expressions in src/aead.rs (constructor builds `Self {..}`);
   sandbox-add-token kill flips; byte-exact restore witness.
8. `throughput_min_of_n_floor` — min-of-8 MB/s table (dev profile):
   1KB: seal 10.83 / open 5.72; 64KB: seal 7.44 / open 9.53; 1MB: seal
   10.59 / open 10.19. Floor asserted at 64KB: >= 2 MB/s seal+open.
9. `ct_scan_court.rs::kernel_scan_zero_unsafe_zero_asm_with_sandbox_kills`
   — 3 injection kills (unsafe / asm! / core::arch), restore witness.

## Gates (exact commands, observed exits, this session)

- `cargo test -p cgap-crypto --test aead_courts --test ct_scan_court` —
  exit 0, run TWICE (177.75s and 181.00s walls): 8 + 1 passed, 0 failed
  (both runs).
- `cargo check -p cgap-crypto --target riscv32imc-unknown-none-elf` —
  exit 0 (target installed this session via `rustup target add`).
- `cargo check -p cgap-crypto --target wasm32-unknown-unknown` — exit 0.
- `cargo check -p cgap-crypto --target wasm32v1-none` — FAILS at
  E0463 (target std component not installed in this environment);
  disclosed as an environment gap, not a code failure. All other
  wasm checks green.
- `cargo test -p cgap-crypto --lib` — 24 lib tests pass; 1 FAILURE in
  `lease::guard_courts::guard_compile_rejection_state_inversion_is_unrepresentable`
  (src/lease.rs:430) — AB-3's in-flight file, NOT touched by this lane
  (pre-existing to my charter files; AB-3 owns the fix).
- `cargo test -p cgap-crypto` (whole crate) — blocked by AB-3's in-flight
  `tests/dbg5.rs` compile error (E0603, private module path). Not my
  lane's file. Owned-target gates above are the lane truth.

## Fixture provenance

RFC 8439 §2.5.2 and §2.8.2 vectors transcribed from the RFC text fetched
from rfc-editor.org THIS session (curl, exit 0). The §2.5.2 tag was ALSO
independently confirmed by a from-scratch bignum reference in Python
before landing, and both fixtures are re-validated live against RustCrypto
every run (courts 3).

## Defects found and fixed by the courts (anti-vacuity receipts)

1. **Wrong r-clamp placement**: limb extraction with post-extraction
   masking produced a wrong clamped r — §2.5.2 caught it. Fix: donna's
   byte-offset extraction with per-limb clamp masks
   (0x3ffffff / 0x3ffff03 / 0x3ffc0ff / 0x3f03fff / 0x00fffff), verified
   against the actual poly1305-donna-32.h reference (fetched from
   github.com/floodyberry/poly1305-donna this session) and the RFC
   vector. Python limb-level diffing pinned the divergence.
2. **Hibit on the final partial block**: the 2^128 term was injected on
   the padded tail block; donna's `final` flag forbids it. Caught by the
   §2.5.2 34-byte (non-multiple-of-16) fixture. Fix: `hibit` parameter,
   0 on the final partial block.
3. **s-add without carry chain**: the `(h + s) mod 2^128` add was
   word-wise wrapping; donna's finish adds with a 32→128 carry chain.
   Fixed to donna-exact.
4. **Nonce96 packed-layout collision (disclosed coordination)**: my
   `#[repr(C, packed(4))]` fix (spec: size-pinned 12) collided with a
   concurrent AB-3 SLA-unblock (repr(C), size 16). Spec wins; owner's
   packed(4) fix stands, hand-written by-value trait impls (derive
   borrows packed fields = compile error). Reported to coordinator
   mid-flight; the stray `}` at aead.rs:96 came from the concurrent
   edit, removed.
5. **Court-side, disclosed**: (a) `Nonce96::new` counter constant in the
   §2.8.2 fixture was wrong on MY side (0x4342_4140_4746_4544 vs the
   correct LE counter 0x4746_4544_4342_4140 — the relayed "nonce
   byte-order" defect was this test constant, NOT a wire-layout drift;
   the byte-identical RustCrypto cross-check is the wire-law authority
   and passes). (b) RustCrypto `decrypt` takes the ATTACHED form
   (ct‖tag) as `msg` — direction 2 of the cross-check initially passed
   only ct; fixed. (c) throughput floor scoped to the 64KB baseline
   leg (dev-profile variance at 1MB; all legs print, floor asserted
   at the baseline leg).

## SLA minimal-unblocks (disclosed, other lanes' files)

- `src/chacha.rs` (AB-3): `plaintext.chunks(64)[blk]` — `Chunks` is not
  indexable; changed to `.nth(blk).expect(...)`. Disclosed per the
  same-checkout fan-out law.
- AB-3's in-flight fips203.rs compile breaks (const-assert panics,
  i16/u16) froze the shared crate ~35 min mid-session; this lane polled
  until AB-3 landed and ran its gates against the converged tree.

## Disclosed deviations

1. `Nonce96` is `#[repr(C, packed(4))]` to pin the 12-byte layout the
   spec demands (repr(C) alone pads to 16). Trait impls are handwritten
   by-value for exactly this reason.
2. `#[doc(hidden)] pub fn mac_tag_var_aad` — the RFC §2.8.2 vector uses a
   12-byte AAD while the production surface pins AAD_LEN=36; the seam
   exists so the published vector is drivable exactly. Production
   seal/open remain fixed-AAD.
3. The count-law scan forbids `Nonce96 { epoch…}` construction
   expressions (the struct definition and impls contain "Nonce96 {" as
   substring, so the law targets construction sites only; the sandbox
   kill injects a construction expression).
4. Throughput numbers are DEV-PROFILE (unoptimized) — release numbers
   will be several times higher; the court pins a dev-profile floor at
   the 64KB baseline leg only.
5. Environment gaps: wasm32v1-none target std absent; AB-3's two
   in-flight compile breaks at gate time. Both disclosed above.
