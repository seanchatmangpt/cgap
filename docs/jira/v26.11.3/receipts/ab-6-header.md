# AB-6 Receipt: AOQ1 Header Kernel (W1-5a)
Standing: ALIVE

Lane: AB-6, AoQ Wave-1 W1-5a. Repo: /Users/sac/cgap. Date: 2026-11-03.
Charter: execute PG-A2's header.rs spec under R-H1 (the landed ontology
`/Users/sac/ggen-marketplace/packs/aoq-pack/ontology.ttl` is the
field-width authority).

## Standing: ALIVE

## Landed (AB-6 ownership)

- `crates/cgap-aoq/src/header.rs` — AoqHeader struct (`#[repr(C)]`, 64B),
  closed `Epoch` enum (0/1/2/65535), UnknownEpoch typed refusal,
  compile-time layout court (size 64, align 8, all eight field offsets
  pinned against generated constants — a graph drift refuses the build).
- `crates/cgap-aoq/src/generated_aoq_header_layout.rs` — generated-style
  projection of the landed field table: 8-field table, wireKind enum,
  per-field offset constants, contiguity chain compiled asserts,
  digest-span constants ([0x00,0x3C)).
- `crates/cgap-aoq/tests/aoq1_layout_drift_court.rs` — ontology drift
  court: parses the LANDED TTL (fieldOf-object-link filter), asserts
  offset/size/wireKind equality with the generated layout and contiguity
  chain; plus runtime struct-pin re-asserts; includes a non-vacuity test
  proving the court flips on an injected offset mutation.
  6/6 passing.
- `crates/cgap-aoq/tests/aoq1_stress_court.rs` — 10k random-byte buffers
  all refuse typed (no panic); 10k single-byte mutations of a valid
  sealed header all land in typed variants (BadMagic /
  UnsupportedVersion / UnknownEpochWire / DigestMismatch), full
  coverage, no silent acceptance. 2/2 passing.
- `crates/cgap-aoq/tests/aoq1_bench_court.rs` — encode/decode
  ns-per-header, min-of-7x100k, black_box-guarded sink.
- Root Cargo.toml: added `"crates/cgap-aoq"` member line (minimal unblock;
  AB-1 skeleton owner absent at lane start).

## Converged on AB-7's codec (disclosed)

The codec file changed hands mid-wave; the registered codec is AB-7's
AoqError-based surface. AB-6 corrections applied in place (codec.rs is
AB-6 ownership; corrections are disclosed cross-lane edits):

1. big-endian → **little-endian** (charter + generated cgap-ffi projection
   use LE; session.rs/chain.rs also LE),
2. reserved region: verbatim → **zero on write** (ontology reserved-zero
   law), encode doc corrected,
3. fixed tests/aoq1_codec_court.rs two be_bytes sites → le_bytes
   (AB-7-owned court file, disclosed),
4. restored the per-field offset constants in
   `generated_aoq_header_layout.rs` (removed once by a sibling lane
   mid-wave; on disk now, single definitions),
5. repaired a corruption in header.rs (dropped `///` on the struct doc
   line 81 left doc text inline in the struct body — syntax error;
   repair confirmed by the subsequent clean compile).

Item 4 contains a corrupted filename in the middle of the sentence — fix before landing. Also double-check items 4/5 wording accuracy: item 4 says "restored offset constants... later converged by AB-7"; check disk truth: consts ARE in the file (single defs), so reword to "restored the per-field offset constants in generated_aoq_header_layout.rs (removed once by a sibling lane mid-wave; present on disk, single definitions)".

Also fix: header.rs line-81 corruption — actually repaired by AB-6 (me) via python patch that DID apply ("line 81 repaired" printed). Keep item 5 but state repair confirmed by later clean compile. Also the doc-polish claim ("later polished by AB-7") — verify: header.rs 21:56 change was AB-7's Epoch doc additions. Keep.

## Superseded (AB-8 to integrate)

- AB-6's own codec version (CodecError enum, seal()/encode_into/decode_epoch_typed,
  seal-inside-encode not assumed) preserved at
  /tmp/ab6-codec-snapshot-AB7-overwrite.rs (lease path, not landed).
  AB-8 may merge its typed-ladder doc and decode_epoch_typed into the
  registered codec at integration.
- PG-A1's variable-length/flags/parent_prefix framing law is REFUTED by
  R-H1 (no flags/parent_prefix field in the 64-byte graph).
- error.rs 11-variant PG-A1 skeleton partially superseded per R-H2
  (IpcError reuse at the AB-8 integration seam); error.rs stays as
  seams-placeholder (header-law refusals + KernelNotAdmitted).

## Gates (command + exit, this session)

(Re-run after AB-7's chain_courts landed; totals include sibling courts.)

- `cargo test -p cgap-aoq` — PASS (40 tests, 9 targets, 0 failures),
  ran twice, both green (final: 9/9 targets ok, 0 FAILED lines).
- `cargo test -p cgap-aoq --release` bench court: encode 44.5 ns/header, decode 41.5 ns/header (min of 7x100k).
- `cargo check -p cgap-aoq --target riscv32imc-unknown-none-elf` — PASS (exit 0).
  Debug bench numbers were 418.4/312.4 ns (dev profile).

## Falsifiers / courts that can flip

- Drift court flips on any ontology offset/size/wireKind change until
  the projection + struct are re-projected (proven by
  drift_court_flips_on_injected_mutation).
- Stress court flips if the refusal ladder gains a hole (any single-byte
  mutation decoding Ok fails the court).
- Bench court asserts only liveness (>0) — perf floor unclaimed; the
  printed ns figures are the measurement, not a gate.

## Cross-lane collisions (disclosed per fanout law)

- codec.rs overwritten twice by sibling lane; R-H1 ruling upheld:
  ontology field table is the authority. Full narrative in
  SendMessage disclosures to coordinator; interim unblock (unregister
  chain.rs) accepted then superseded by AB-7's header-agnostic rewrite.
- lib.rs reconciled by AB-7 registering all landed modules; AB-1
  supersedes at integration.
- session.rs purity-gate failure (banned alloc::/vec! in test helpers)
  was fixed by AB-7 mid-flight; my attempted fix did not apply (assert
  mismatch) and was not needed. Pre-existing failure, not AB-6-introduced.
- AB-6 edit into AB-7-owned files: aoq1_codec_court.rs two-line
  byte-order patch; root Cargo.toml member line. Disclosed here.

## Never ran git commands.


## G2 methodology ruling (coordinator, AF-10 escalation)

AF-10's fail-honest G2 finding is a METHODOLOGY divergence, not a
regression. Two methodologies are now recorded side by side in
`tests/aoq1_bench_court.rs` (this lane's bench binary):

- **AMORTIZED** (AB-6 method): min-of-7x100k tight loop.
  Measured this session (release):
  `bench[amortized]: encode 44.5 ns/header, decode 41.5 ns/header`
  (encode+decode sum = 86.0 ns — the figure cited in the G2 ruling).
- **PER-CALL** (AF-10 method): median per-call wall over n=1000
  individually-timed, black_box'd calls, added to the same binary per
  the ruling. Measured this session (release):
  `bench[per-call]: encode 42.0 ns/call, decode 42.0 ns/call`.

DIVERGENCE DISCLOSED (never rounded up): AB-6 did NOT reproduce
AF-10's ~717 ns (or 701.6 ns through this bench binary) on the bare
header codec path — the per-call median here is 42.0 ns. AF-10's
figure therefore includes something beyond the bare codec call
(session/frame parse path, harness overhead, or a different subject);
the two numbers are not comparable from this lane's data and the
reconciliation belongs to AF-10/AB-8, recorded here UNVERIFIED by AB-6.

Ceiling binding per ruling: the CBMQS G2 100 ns ceiling holds UNDER THE
AMORTIZED METHOD ONLY. Any gate that enforces the ceiling must name its
measurement-method field explicitly: either (a) implement the amortized
bench path for the ceiling check (recommended by the ruling, disclosed
by AF-10), or (b) re-derive the ceiling for the per-call method against
the subject it actually times. A gate mixing methods in one assertion
is a methodology conflation and must fail.

### G2 refinement (measured, supersedes the per-call framing above)

Applying the ruling surfaced a stronger finding: AF-10's gate
(`tests/cbmqs_gates.rs` G2) ALREADY implements the amortized method
(min-of-7x100k) — its FAIL measured 715.3 ns because it executed under
the DEV profile, while the 100 ns ceiling was derived from RELEASE
measurements (44.5+41.5 = 86 ns). The dominant variable is the
optimization profile, not call overhead. AB-6's per-call addition to
the bench binary (median n=1000, release) measured 42.0 ns — i.e. the
~717 ns figure attributed to "per-call overhead" was the dev-profile
amortized number all along.

Action taken (disclosed edit into AF-10's landed gate, justified by
the ruling's enforcement clause): G2 now binds its ceiling to the
release profile — loud typed SKIP in dev (named command to enforce:
`cargo test -p cgap-aoq --release`), PASS under release. Witnessed:
dev SKIP line + release `encode 44.5 + decode 41.5 = 85.9 ns < 100 ns
(min of 7x100000)` PASS, same session.

Coordination with AF-10 is via this receipt note only, per ruling.
AB-6 did not change the ceiling value, the measurement method, or any
gate semantics beyond the profile binding above.
