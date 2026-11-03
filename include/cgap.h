/* ==========================================================================
 * include/cgap.h — lane A19 Wave 1 ABI freeze (v26.11.1).
 *
 * The four sovereign entrypoints (ADR-005 boundary law: every foreign
 * consumer crosses exactly one membrane, the C-ABI; lane doc §2 is the
 * normative spec this header freezes):
 *
 *   cgap_admit_frame    — bytes enter; magic/format/SHACL gate or typed refusal
 *   cgap_eval_bn3       — admitted bytes evaluate deterministically against a
 *                         flash-resident .bn3 network; zero copies of the input
 *   cgap_commit_parquet — evaluated batches commit to Parquet (host targets
 *                         only; an XIP/no_std build omits this symbol via cfg)
 *   cgap_verify_p28     — P28 spoliation evidence verification; read-only
 *
 * Additive to the frozen slot surface in cgap_c_data.h (cgap_i32_batch et
 * al.); the sovereigns never touch the Arrow export slots.
 *
 * Implementation owners (lane doc §2): admit A14+A16, eval A12+A13, commit
 * A02+A03, verify A17. This header and crates/cgap-ffi/src/sovereign.rs are
 * the Wave-1 ABI-freeze pair; the §6.1 header court (Wave 3, in
 * crates/cgap-ffi/src/sovereign.rs::wave3_courts) diffs them mechanically
 * and is non-vacuous by its M1-M12 mutation corpus.
 *
 * Requires C11 for the _Static_assert layout pins; C++ consumers get the
 * same declarations without the pins. No headers beyond stddef.h/stdint.h.
 * ========================================================================== */

#ifndef CGAP_H
#define CGAP_H

#include <stddef.h> /* size_t */
#include <stdint.h> /* uint8_t, uint32_t, uint64_t, int32_t, UINTPTR_MAX */

#ifdef __cplusplus
extern "C" {
#endif

/* --------------------------------------------------------------------------
 * ABI version (§2 "ABI versioning"). uint32, starts at 1. Mirrored in Rust
 * as `pub const CGAP_ABI_VERSION: u32 = 1` in
 * crates/cgap-ffi/src/sovereign.rs; the §6.1 header court asserts parity.
 * A future incompatible change bumps both together; cgap_version() remains
 * the human-readable triple for the frozen slot surface and is NOT this
 * version.
 * ------------------------------------------------------------------------ */

#define CGAP_ABI_VERSION 1u

/* --------------------------------------------------------------------------
 * CgapStatus — the sovereign status word (§2). Reachability law: every value
 * MUST stay reachable through a real code path on some supported target;
 * an enum value no injected fault can produce is untested decoration
 * (status court, §6.2).
 * ------------------------------------------------------------------------ */

typedef uint32_t CgapStatus;

enum {
  CGAP_OK                  = 0x00000000u,
  CGAP_ERR_INVALID_MAGIC   = 0x43470001u, /* 'CG' + 01 */
  CGAP_ERR_OABI_MISMATCH   = 0x43470002u, /* 'CG' + 02 */
  CGAP_ERR_SHACL_VIOLATION = 0x43470003u, /* 'CG' + 03 */
  CGAP_ERR_LEASE_EXHAUSTED = 0x43470004u, /* 'CG' + 04 */
  /* repair-10 (lane F8): FA9's concurrent-writer refusal promoted to the
   * C header so C consumers can match it by name; Rust twin
   * crates/cgap-ffi/src/sovereign.rs CGAP_ERR_CONCURRENT_WRITER_DENIED. */
  CGAP_ERR_CONCURRENT_WRITER_DENIED = 0x43470006u, /* 'CG' + 06 */
  CGAP_ERR_P28_SPOLIATION  = 0x503238FFu /* 'P28' + FF */
};

/* --------------------------------------------------------------------------
 * cgap_frame_view — header-defined, non-owning, zero-copy (§2). payload
 * points INTO the caller's bytes; the view dies with bytes. Rust twin:
 * CgapFrameView in crates/cgap-ffi/src/sovereign.rs.
 * ------------------------------------------------------------------------ */

typedef struct {
  const uint8_t* payload; /* points INTO the caller's bytes — zero copy */
  size_t payload_len;
  uint32_t schema_id;
  uint32_t rule_count;
} cgap_frame_view;

/* Wave-1 layout pins (C11 _Static_assert), strengthened in Wave 3 with
 * per-field offsetof pins so a field-type change that sizeof survives via
 * tail padding still fails the compile. The view is pointer-width dependent
 * by construction: host LP64 and XIP ILP32 (ESP32-S3 / ESP32-C3 rv32imc)
 * forms are pinned explicitly. Anti-vacuity: widening, reordering, or
 * retyping any cgap_frame_view member MUST fail this compile. */
#if !defined(__cplusplus) && defined(__STDC_VERSION__) &&                      \
    __STDC_VERSION__ >= 201112L
#include <stddef.h> /* offsetof */
/* Wave-4 width pin: the status word is u32 on EVERY target (Rust twin:
 * `const _: () = assert!(size_of::<CgapStatus>() == 4)` in sovereign.rs);
 * a 16-bit-int MCU target must refuse to compile the header, not silently
 * narrow the register. */
_Static_assert(sizeof(CgapStatus) == 4,
               "cgap ABI freeze: CgapStatus must be 32 bits on every target");
#if UINTPTR_MAX == 0xFFFFFFFFFFFFFFFFu
_Static_assert(sizeof(cgap_frame_view) == 24,
               "cgap ABI freeze: cgap_frame_view must be 24 bytes on LP64");
_Static_assert(_Alignof(cgap_frame_view) == 8,
               "cgap ABI freeze: cgap_frame_view must be 8-aligned on LP64");
_Static_assert(offsetof(cgap_frame_view, payload) == 0,
               "cgap ABI freeze: payload must be at offset 0 on LP64");
_Static_assert(offsetof(cgap_frame_view, payload_len) == 8,
               "cgap ABI freeze: payload_len must be at offset 8 on LP64");
_Static_assert(offsetof(cgap_frame_view, schema_id) == 16,
               "cgap ABI freeze: schema_id must be at offset 16 on LP64");
_Static_assert(offsetof(cgap_frame_view, rule_count) == 20,
               "cgap ABI freeze: rule_count must be at offset 20 on LP64");
#elif UINTPTR_MAX == 0xFFFFFFFFu
_Static_assert(sizeof(cgap_frame_view) == 16,
               "cgap ABI freeze: cgap_frame_view must be 16 bytes on ILP32");
_Static_assert(_Alignof(cgap_frame_view) == 4,
               "cgap ABI freeze: cgap_frame_view must be 4-aligned on ILP32");
_Static_assert(offsetof(cgap_frame_view, payload) == 0,
               "cgap ABI freeze: payload must be at offset 0 on ILP32");
_Static_assert(offsetof(cgap_frame_view, payload_len) == 4,
               "cgap ABI freeze: payload_len must be at offset 4 on ILP32");
_Static_assert(offsetof(cgap_frame_view, schema_id) == 8,
               "cgap ABI freeze: schema_id must be at offset 8 on ILP32");
_Static_assert(offsetof(cgap_frame_view, rule_count) == 12,
               "cgap ABI freeze: rule_count must be at offset 12 on ILP32");
#else
#error "cgap ABI: unsupported pointer width"
#endif /* pointer width */
#endif /* C11 */

/* --------------------------------------------------------------------------
 * Four sovereign entrypoints (§2 normative). All callee-side evaluation is
 * zero-copy: admit produces a view, never a buffer; eval reads view->payload
 * in place (on XIP, through the flash cache — no SRAM staging buffer);
 * digest paths fold raw buffers. The ONLY copy in the whole surface is the
 * out_lanes write-out, caller-owned. The callee performs no allocation in
 * admit/eval/verify; commit allocates on host targets only.
 * ------------------------------------------------------------------------ */

/* Bytes enter; magic/format/SHACL gate or typed refusal.
 * out_view is caller-allocated and MAY be NULL to probe. out_view->payload
 * points into `bytes` — zero copy; the view dies with `bytes` (§2 ownership
 * table). Divergence from the lane-doc §2 sketch: that snippet writes
 * `const cgap_frame_view* out_view`, which no callee could write through;
 * the frozen header uses a non-const out-parameter per the doc's own
 * ownership table (callee fills out_view.payload). */
CgapStatus cgap_admit_frame(const uint8_t* bytes, size_t len,
                            cgap_frame_view* out_view);

/* Admitted bytes evaluate deterministically against a flash-resident .bn3
 * network (layout: A11) under .oabi rule tables (A06 §6); zero copies of the
 * input. Callee never writes past lanes_cap; out_lane_count receives the
 * lane count written. */
CgapStatus cgap_eval_bn3(const cgap_frame_view* view,
                         const uint8_t* bn3_rodata, size_t bn3_len,
                         int32_t* out_lanes, size_t lanes_cap,
                         size_t* out_lane_count);

/* Evaluated batches commit to Parquet — HOST targets only; an XIP/no_std
 * build omits this symbol entirely (cfg), so it never enters the SRAM budget
 * (§2 ownership rules, §5 size budget). `path` is NUL-terminated UTF-8,
 * caller-owned. */
CgapStatus cgap_commit_parquet(const int32_t* lanes, size_t lane_count,
                               const char* path);

/* P28 spoliation evidence verification — digest + license predicate over the
 * embedded 80-byte P28LicenseAttestation (layout: A17 §2, cited not
 * redefined); read-only, no allocation. out_digest MAY be NULL. */
CgapStatus cgap_verify_p28(const uint8_t* evidence, size_t len,
                           uint64_t* out_digest /* MAY be NULL */);

/* --------------------------------------------------------------------------
 * Frozen slot surface (lane FIX-07, closing the RED-03 §1 header gap). The
 * Arrow export slots (cgap_c_data.h defines struct ArrowArray/ArrowSchema;
 * forward-declared here so this header stays self-contained) and the P28
 * invariant check ride the cdylib link and are mirrored here so the C
 * prototype surface is 1:1 with the exported-symbol census (RED-03 §2).
 * Truth-based, not count-pinned: scripts/ffi_surface_gate.py derives the
 * expected surface from the freshly-built dylib via `nm -gU` and requires
 * EVERY exported cgap_* symbol to have exactly one prototype here (and
 * vice versa) — a fixed count silently rots the moment a lane adds a
 * #[no_mangle] export (witnessed: RC-05's cgap_stats_snapshot). Rust
 * owners: crates/cgap-ffi/src/lib.rs:43-138, crates/cgap-p28/src/trap.rs:100,
 * crates/cgap-ffi/src/stats.rs:344.
 * ------------------------------------------------------------------------ */

struct ArrowArray;
struct ArrowSchema;

/* Build and publish an i32 batch of `n` rows into the live export slots;
 * null_every > 0 inserts a null at every `null_every`-th row (0 = all
 * valid). Returns the live ArrowArray pointer; the CALLER (importer) takes
 * release ownership per the Arrow C Data Interface contract. */
struct ArrowArray* cgap_i32_batch(int64_t n, int64_t null_every);

/* Addresses of the exported schema/array slots (slot-polling consumers). */
size_t cgap_schema_addr(void);
size_t cgap_array_addr(void);

/* Length of the last exported batch. */
int64_t cgap_batch_rows(void);

/* cgap version string pointer (NUL-terminated, static lifetime). NOT
 * CGAP_ABI_VERSION — that is the frozen ABI word above. */
const char* cgap_version(void);

/* Rotate-xor fold digest of the live exported array at `array_addr` (the
 * address cgap_array_addr serves). NOT cryptographic — accidental-mutation
 * detection only. Undefined for null/foreign layouts (Rust side asserts). */
uint64_t cgap_digest_of(size_t array_addr);

/* P28 record-intrinsic check over an 80-byte P28LicenseAttestation (A17).
 * Returns 0 iff intrinsically valid. FAIL-CLOSED: a NULL `att` HALTS the
 * process (spoliation halt, crates/cgap-p28/src/trap.rs) and never returns. */
uint32_t cgap_p28_invariant_check(const uint8_t* att);

/* --------------------------------------------------------------------------
 * Runtime stats snapshot (lane RC-05, fix-14 R-A1). repr(C) fixed layout,
 * size 120 / align 8, mirrored from crates/cgap-ffi/src/stats.rs (offsets
 * pinned by its stats_block_court.rs). The CRC word is FNV-1a-32 over bytes
 * CRC word is FNV-1a-32 over the first 112 bytes of the block — tamper
 * evidence, NOT cryptographic (same fold family as the Parquet footer seal).
 * ------------------------------------------------------------------------ */
typedef struct CgasStatsBlock {
    uint32_t magic;               /* 'CSTB' = 0x43535442                */
    uint32_t layout_version;      /* 1                                  */
    uint64_t ingress_admitted;    /* taxonomy row 6                     */
    uint64_t refused_invalid_magic;   /* rows 1-2                       */
    uint64_t refused_oabi_mismatch;   /* row 3                          */
    uint64_t refused_p28_spoliation;  /* rows 4-5                       */
    uint64_t refusals_class_101;  /* ring-level class-101 refusals      */
    uint32_t quota_max_refusals;
    uint32_t quota_window_refusals;
    uint32_t quota_tripped;       /* 0/1                                */
    uint32_t _pad0;
    uint64_t quota_total_refusals;  /* lifetime audit counter           */
    uint64_t receipts_minted;
    uint8_t  last_receipt_digest[32];
    uint32_t block_crc;           /* FNV-1a-32 over first 112 bytes     */
    uint32_t _pad1;
} CgasStatsBlock;

/* Pull-based sealed snapshot into the caller's buffer (>= 120 bytes).
 * Returns CGAP_OK, or CGAP_ERR_INVALID_MAGIC for a NULL `out`. The kernel
 * never sleeps or blocks the caller (fix-14 R-A1.3). */
CgapStatus cgap_stats_snapshot(CgasStatsBlock* out);

#ifdef __cplusplus
}
#endif

#endif /* CGAP_H */
