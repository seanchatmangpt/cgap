# A04 Finish Receipt

- **Lane**: A04 (triumvirate isomorphism — `cgap-core`: frame, dma_ring,
  isomorphism), v26.11.1 — lane-finisher pass over the landed artifact
- **Subject**: repo `/Users/sac/cgap`, branch `main`, HEAD `cfd085a9`;
  lane lease `crates/cgap-core/src/` + the lane spec doc + this receipt
- **Mode**: assessment first (dispatch: "a wave landed in `9be9109d`").
  Assessment finding: `9be9109d` landed the crate substantially complete
  (Wave 1 ABI freeze, Wave 2 encode/decode/validate, Wave 4 guard-flip
  + rv32imc courts, Wave 5 u64-bound repair, Wave 7 read-gate flip,
  Wave 9 cross-lane gate court) — 69 tests green across 10 suites at
  baseline, exit 0, witnessed this session. One mission gap was real:
  the wire-to-memory INGRESS (§8.1 memory tier, "zero-copy
  addressability") had no kernel API and no court — this pass landed
  both.
- **Verdict**: **ALIVE** — full suite ran fresh this session, exit 0,
  75 passed / 0 failed across 11 suites; anti-vacuity flip witnessed in
  a /tmp sandbox (exit 101) with the exact assertion named.

## Gate (commands + exits, this session)

- `CARGO_TARGET_DIR=/tmp/cgap-lane-A04-target cargo test -p cgap-core`
  -> exit **0**, **75 passed / 0 failed** across 11 suites (lib 17;
  cross_lane_gate 8; doc_offset_table 2; guard_flip 11; identity 4;
  ingress_layout 6; manifest 3; rv32imc 2; source_scan 5;
  static_assert 4; unit_dma_ring 13). Baseline pre-finish was 69/10.
- Sandbox flip (all under `/tmp/a04-finish-sbx.mJR3BF/`, canonical tree
  untouched; subject closure staged: `crates/{cgap-core,cgap-blake3,
  cgap-gate}` + `docs/jira/v26.11.1/lanes/` + `rust-toolchain.toml` —
  the toolchain pin is load-bearing: without it the sandbox baseline
  falls back to the ambient default toolchain missing the riscv target
  and rv32imc_court fails loud):
  - BASELINE `cargo test -p cgap-core` -> exit **0**, 75 passed.
  - SHIM (marked `SHIM-COURT`): `pub const FRAME_HEADER_LEN: usize =
    128;` -> `= 129;` in the sandbox `frame.rs` (the dispatch-named
    "change the header size constant" mutation).
  - AFTER -> **compile abort, exit 101**: `error[E0080]: evaluation
    panicked: assertion failed: FRAME_HEADER_LEN ==
    core::mem::size_of::<CgapFrameHeader>()` at `frame.rs:389` — the
    exact layout assertion the dispatch required, tripping on the new
    constant==layout pin before any court binary even builds. A compile
    abort is not a failure count: it IS the flip, witnessed.
- Restore discipline: the real tree was never mutated by the sandbox
  (mktemp copy; `FRAME_HEADER_LEN: usize = 128` verified on disk in the
  canonical `frame.rs` after the flip, grep exit 0).

## Mission items -> witnessed surfaces

1. **Ingress layout frozen (this pass's landing).** New kernel API
   `CgapFrameHeader::project(&[u8; 128]) -> Result<&CgapFrameHeader,
   FrameError>` (`crates/cgap-core/src/frame.rs`): the
   `slice::from_raw_parts`-shape made safe — runtime 64-byte alignment
   gate refuses misaligned ingress typed (`FrameError::Misaligned`, new
   variant + Display arm, disclosed), `unsafe` block carries the full
   soundness argument (repr(C) + gap-free const lattice + all-POD fields
   => every 128-byte bit pattern valid). Two new const pins tie the wire
   constants to the layouts: `FRAME_HEADER_LEN ==
   size_of::<CgapFrameHeader>()` and the ATTESTATION_LEN twin.
   Court `tests/ingress_layout_court.rs` (6 tests + compile-time row):
   runtime size/align pins (row 02); zero-copy addressability — the
   projected reference aliases the region base and every field sits at
   its §6.1 offset (0/4/5/6/8/16/48, attestation 48+48 for
   upstream_receipt) (row 03); alignment invariance across five
   64-aligned addresses with decode∘project identity (row 04); all 63
   misaligned residues refused typed (row 05); named mutation — a
   region byte flip is VISIBLE through the view, unpassable by a
   copying implementation (row 06); wire→memory identity over the
   8-size doc sweep with real cgap-blake3 digests (row 07).
2. **URN drift (the ONE allowed spec-doc fix, disclosed).**
   `docs/jira/v26.11.1/lanes/A04-triumvirate-isomorphism.md` §6.2
   `license_urn_hash` comment cited `urn:cgap:license:p28-sil:v2.1.0-
   sovereign`; the repo-frozen URN (CLAUDE.md byte-pin,
   `03-rename-receipt.md` decision D2) is `urn:avec:license:p28-sil:
   v2.1.0-sovereign`. The doc line was fixed to the frozen URN; no code
   pin touched (`tests/common/mod.rs:72` URN byte-identical before and
   after; manifest court green). Verified: zero `urn:cgap:license`
   residues in the doc (grep exit 1), exactly one frozen URN occurrence.
3. **Host suite.** Command + exit in Gate section above.
4. **Sandbox flip.** Witnessed, Gate section above.
5. **Cleanup: BLOCKED (typed), left for the coordinator.** The dispatch
   ordered `DELETE /tmp/cgap-lane-A04-target before returning`; the
   permission system denied the deletion — `rm -rf
   /tmp/cgap-lane-A04-target` refused (exit before execution), and the
   multi-path form refused identically. No route-around was attempted
   (a denial is not a routing problem). Residual leases on disk at
   return: `/tmp/cgap-lane-A04-target/` (lane build root),
   `/tmp/a04-finish-sbx.mJR3BF/` (sandbox tree),
   `/tmp/a04-finish-sbx-target/` (sandbox build root),
   `/tmp/frame_shim.rs` (shim intermediate). The canonical tree is
   clean of them; the sandbox SHIM mutation never touched it
   (FRAME_HEADER_LEN == 128 verified on disk post-flip).

## Court inventory run this session

ingress_layout_court rows 01-07 (new, this pass);
static_assert_court (baseline compiles clean; 129 B pad shim -> E0080;
127 B shrink retirement witness; attestation pad shim -> E0080);
guard_flip_court rows 01-10 + anchor self-check;
identity_court (wire+memory 2-tier mode, loud [SKIP] for A17/A19 disk
tiers per §10); doc_offset_table_court (§6.1 + A17 §2.1 machine-checked
against compiled layout); cross_lane_gate_court (A14 dam admits real
A04 frames); rv32imc_court (no_std rlib for riscv32imc + host
baseline); source_scan_gate (E=0 alloc/externals); manifest_court
(frozen URN, no SPDX); unit_dma_ring 13 rows; frame/dma/iso unit
tests 17. All exit 0 this session under the command in the Gate
section. No silent skips: identity_court prints [SKIP] rows for the
two disk tiers it does not own.

## Standing

**ALIVE** — every court named above ran and passed this session under
`CARGO_TARGET_DIR=/tmp/cgap-lane-A04-target cargo test -p cgap-core`
(exit 0, 75/0); the anti-vacuity flip flipped with the exact assertion
named (exit 101); the §6.1 offset table is now address-pinned at
runtime, not just const-pinned.

## Divergences (recorded, not silent)

1. **A04 §6.2 attestation draft yields to A17 §2** (pre-existing,
   disclosed by the code): the spec's local 80-byte P28 draft (magic
   "P28\0", u8 version, 40 B sig_or_id) is NOT what landed; frame.rs
   implements the A17 §2/§2.1 layout (magic "P28S", u16 version+flags,
   u64 ceiling, 32+32 digests) per the dispatch authority pin "A17
   P28LicenseAttestation (80B)". The doc_offset_table_court checks the
   A17 table, and the lane doc's §6 struct declaration (line 186) cites
   "see LICENSE-P28-SIL.md" without restating the field lattice, so the
   only conflicting prose was §6.2's comment lines — the URN line is
   fixed (mission item 2); the remaining §6.2 field names (license_urn_
   hash, sig_or_id) stand as UNVERIFIED prose superseded by A17 §2.1.
2. **Sandbox fidelity note**: rv32imc_court fails loud in a bare /tmp
   staging unless `rust-toolchain.toml` is staged with the crate
   (floating nightly carries the riscv target; the ambient default
   toolchain does not). Recorded so future lane sandboxes stage the
   toolchain pin.
3. **§9 DoD 6 (A14/A17/A19 confirmation links)**: not complete on disk
   for A17/A19 — A14's finish receipt (a14-finish-receipt.md) links
   this crate as layout contract and even re-witnessed this pass's
   frame.rs change mid-session; no A17/A19 finish receipts exist yet
   (`ls docs/jira/v26.11.1/waves/` this session). Unresolved item
   stays with those lanes, not re-asserted here.
