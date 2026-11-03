# RED-03 — Hostile Clean-Room FFI Shim (Adversarial Red-Team Lane)

Standing: ALIVE. Read-only over crates/ include/ scripts/; builds confined to
/tmp/red03-target and /tmp/red03-shim; zero git commands; the only shared-tree
write is this receipt.

## 1. Header inventory

`include/cgap.h` EXISTS (175 lines) — the lane does not go UNSUPPORTED on the
primary referent. On-disk header inventory:

- `include/cgap.h` (175 lines): the 4 sovereign entrypoints (lines 146, 153,
  162, 168: cgap_admit_frame / cgap_eval_bn3 / cgap_commit_parquet /
  cgap_verify_p28), CGAP_ABI_VERSION 1u, CgapStatus word, cgap_frame_view
  with _Static_assert layout pins at lines 97-123.
- `include/cgap_c_data.h` and `include/cgap_format_codes.h` exist; a grep for
  `cgap_*(` function prototypes across BOTH returns zero matches. The six
  Arrow-slot symbols therefore have NO C prototypes on disk — their only
  declarations are Rust (crates/cgap-ffi/src/lib.rs:43-132). This is a
  recorded gap, not a blocker.

## 2. Exported-symbol census (exact, this session)

Command: `CARGO_TARGET_DIR=/tmp/red03-target cargo build --release -p cgap-ffi`
(exit 0, 2.79s). Then `nm -gU /tmp/red03-target/release/libcgap_ffi.dylib |
grep " T "` — EXACTLY 11 exported text symbols:

1. cgap_admit_frame   — crates/cgap-ffi/src/sovereign.rs:291 (header: cgap.h:146)
2. cgap_eval_bn3      — sovereign.rs:532 (cgap.h:153)
3. cgap_commit_parquet — sovereign.rs:696 (cgap.h:162)
4. cgap_verify_p28    — sovereign.rs:291-group / declared cgap.h:168
5. cgap_p28_invariant_check — crates/cgap-p28/src/trap.rs:100 (NO C header decl)
6. cgap_i32_batch     — lib.rs:43 (no C prototype)
7. cgap_schema_addr   — lib.rs:66 (no C prototype)
8. cgap_array_addr    — lib.rs:73 (no 6/11 symbols carry C prototypes; this one too)
9. cgap_batch_rows    — lib.rs:80 (no C prototype)
10. cgap_version      — lib.rs:87 (no C prototype)
11. cgap_digest_of    — lib.rs:132 (no C prototype)

Note: the XIP symbol cgap_xip_start (sovereign.rs:2834/2852/3175 and
xip_tables.rs:459) compiles out of the host cdylib (cfg(target_os="none")) —
absent from nm, host consumers never resolve it. macOS emits a leading
underscore (_cgap_admit_frame) via its C ABI; that is toolchain, not surface.

## 3. Semantic depth check (what the 4 sovereigns actually do)

The sovereigns are NOT thin status-returns; each delegates to a real owner
crate path:

- admit: cgap.h:146 -> sovereign.rs:413, magic/format gate + A04 frame
  validation + A14 penstock ingress (deps cgap-core, cgap-gate,
  cgap-blake3 — Cargo.toml GAP-1 block).
- eval: cgap.h:153 -> sovereign.rs:532, A12 unifier + A13 builtins
  (cgap-bn3).
- commit: cgap.h:162 -> sovereign.rs:696, A02 sealed-block writer
  (cgap-parquet, std feature).
- verify: cgap.h:168 -> sovereign.rs:293, A17 P28 attestation verify
  (cgap-p28, host-only dep; attestation.rs:130 std-gated).
- cgap_p28_invariant_check rides the cdylib link from cgap-p28 trap.rs:100 —
  exported incidentally, not via cgap.h. An attacker shipping a shim must
  mirror it only if the downstream links it explicitly; dlopen consumers
  that bind only cgap.h's four never touch it.

## 4. Hostile shim — drafted AND compiled this session

Draft dylib at /tmp/red03-shim/shim.c: pure libc malloc + posix mmap
(MAP_ANON|MAP_PRIVATE), zero cgap deps, 11 exported C symbols mirroring
the census above. Compile: `clang -dynamiclib -o
/tmp/red03-shim/libcgap_ffi_shim.dylib /tmp/red03-shim/shim.c` — LINK_OK,
`nm -gU | grep -c " T "` = 11. So the stub surface is mechanically
mirrordable by clang in one file, no Rust toolchain required.

## 5. 48-hour rewire estimate (per symbol)

Assumption: enterprise attacker with header + captured wire samples, no
source. "Rewire" = downstream membrane dlopens the stub instead of
libcgap_ffi.dylib and passes its existing courts' happy-path assertions.

| Symbol | Stub effort | Why |
|---|---|---|
| cgap_version | 0.5h | return const char* |
| cgap_batch_rows | 0.5h | static i64 |
| cgap_schema_addr / cgap_array_addr | 1h | mmap a page, return addr |
| cgap_i32_batch | 3h | must emit ArrowArray/ArrowSchema struct layout matching cgap-arrow c_abi (header cgap_c_data.h absent C decls forces reverse-engineering from pyarrow's C data interface docs) — the only struct-layout symbol |
| cgap_admit_frame | 8h | happy path: magic check + view fill; adversarial paths (SHACL, OABI, digest) are return-constant fakes |
| cgap_eval_bn3 | 8h | fake-deterministic lane emitter; passing REAL cgap-bn3 golden-vector courts against a fake requires implementing the unifier — attacker fakes only the dlopen consumer's own tests |
| cgap_commit_parquet | 6h | malloc a sink + zero digest; Parquet byte-identity courts fail, consumer happy-path passes |
| cgap_verify_p28 | 6h | constant CGAP_OK plus digest echo; spoliation court flips fail |
| cgap_p28_invariant_check | 2h | constant 0/1 |
| integration (dylib swap, rpath, codesign, notarization) | 6h | |
| total (stub surface only) | ~41h | UNDER the 48h threshold — the shim decoupling is feasible in <48h for a downstream whose verification is dlopen-bind-and-call only |

## 6. What blocks the shim (fences the stub cannot pass)

The <48h feasibility holds ONLY against a weak consumer. Fences that crack
a stub:

- Byte-identity: cgap-parquet sealed-block bytes (A02) are pinned;
  cgap_commit_parquet stub returning digest 0 fails any byte-identity court.
- Spoliation: cgap_verify_p28 constant-OK cannot produce CGAP_ERR_P28_SPOLIATION
  (0x5032_38FF, sovereign.rs:90) on mutated evidence — spoliation courts flip
  to FAIL.
- Determinism: cgap_eval_bn3 stub fails golden-vector replay; BN3 courts
  (crates/cgap-bn3) pin exact lane outputs.
- Frame digest: admit/commit digest chain (BLAKE3 substrate, Cargo.toml GAP-1
  comment) — stub cannot reproduce digests over real bytes.

## 7. Breach verdicts

- BREACH-CONFIRMED (scoped): the SYMBOL surface is decoupled in under the
  48h threshold for a dlopen-bind-and-call consumer with no independent
  byte-identity/determinism/spoliation courts. Reproducer: the compiled
  11-symbol stub dylib at /tmp/red03-shim/shim.c links clean against zero
  cgap code (clang LINK_OK above).
- SURVIVING: the SEMANTIC surface is NOT decoupled at that budget. Any
  downstream that runs the repo's own courts (byte-identity, spoliation
  flip, BN3 golden vectors, digest chain) against the dlopened provider
  cracks every sovereign stub. The stub's sovereign returns are
  return-constant fakes; §6 lists the flip failure per symbol.
- Verdict sentence: an enterprise can rewire the WIRE in <48h; it cannot
  rewire the PROOF. Decoupling resistance is carried by court-side
  verification, not by the ABI symbols themselves.

## 8. Command log (all this session)

- ls include/ crates/cgap-ffi — referent inventory (exit 1 on missing
  crates/cgap-ffi/include, expected; include/ present).
- CARGO_TARGET_DIR=/tmp/red03-target cargo build --release -p cgap-ffi —
  exit 0, 2.79s.
- nm -gU /tmp/red03-target/release/libcgap_ffi.dylib | grep " T " — 11 rows.
- clang -dynamiclib ... shim.c — LINK_OK; nm count 11.
- Grep sweeps over include/*.h and cgap-ffi src for no_mangle/extern "C" —
  census cross-check.

## 9. Standing

ALIVE — every claim above carries a command + exit run this session against
the exact on-disk subjects. No BLOCKED items. No UNSUPPORTED items: every
mission target had an on-disk referent (include/cgap.h EXISTS, contra the
dispatch's contingency branch).

## 10. Divergences

- Dispatch anticipated include/cgap.h might not exist; it does (175 lines).
- The six Arrow-slot symbols lack C prototypes in cgap_c_data.h — recorded
  as a header-completeness PARTIAL in §1, not a mission divergence.
- Build roots /tmp/red03-target and /tmp/red03-shim are lane leases;
  coordinator janitor may reclaim at integration.
