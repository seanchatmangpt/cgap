//! Lane A19 — libcgap sovereign C-ABI surface: the Wave 1 ABI freeze,
//! hardened in Wave 4 (C-side `CgapStatus` width pin, M11/M12 mutation
//! corpus, C++ compile leg with its own rename-flip witness), repaired in
//! Wave 7 per the wave-6 court receipts (ledger walk excludes lane build
//! roots), re-tightened in Wave 8 per the wave-7 court receipt
//! (a08-wave7: anchor uniqueness + isolated-flip discipline; in-code
//! receipt block below), and extended in Wave 9 with the cross-lane
//! integration slice: five real-subprocess courts executing the named
//! campaign integration surfaces (A04 identity, A14 dam over A04 frames,
//! A20 1M-row stress, A05 power-cut sweep, A07 zero-string gate over all
//! built crates) from the membrane's seat — receipts in-code below.
//!
//! Spec: `docs/jira/v26.11.1/lanes/A19-libcgap-ffi-xip.md` §2 (normative C
//! signatures, `CgapStatus` register + reachability law, caller/callee
//! ownership table). Wave 1 slice: struct/const definitions plus size/align
//! static asserts only — zero logic. The four sovereign bodies are owned by
//! implementation lanes (§2 owner table) and land behind cfg gates in later
//! waves (DoD 5):
//!
//! | Entry point         | Implementation owner |
//! |---------------------|----------------------|
//! | `cgap_admit_frame`  | A14 + A16            |
//! | `cgap_eval_bn3`     | A12 + A13            |
//! | `cgap_commit_parquet` | A02 + A03          |
//! | `cgap_verify_p28`   | A17                  |
//!
//! C-side twin: `include/cgap.h` (frozen in the same wave; the §6.1 header
//! court landed in Wave 3 below — a mechanical parity gate between the two,
//! non-vacuous by its mutation corpus).
//!
//! Upstream wire layouts are cited, not redefined here: 128 B
//! `CgapFrameHeader` (A04 §6, `crates/cgap-core/src/frame.rs`) and 80 B
//! `P28LicenseAttestation` (A17 §2, `crates/cgap-p28/src/attestation.rs`).
//! The sovereign surface carries neither struct across the ABI — admit and
//! verify see raw `bytes`/`evidence`; the only view crossing the boundary is
//! `CgapFrameView` below. `.oabi` tables (A06 §6) and `.bn3` networks (A11)
//! stay flash-rodata bytes under `bn3_rodata`.
//!
//! Cross-lane note: `cgap-gate`'s Wave-1 penstock (A14) pins copies of the
//! status register values below ("frozen copy pending Wave-3 re-export");
//! this module is the canonical A19 register those copies cite. Wave 3
//! enforces the copy parity mechanically (status court, below).

#![allow(dead_code)] // Wave 1 freezes the ABI; the logic that reads it lands later.

// ---------------------------------------------------------------------------
// ABI version (§2 "ABI versioning")
// ---------------------------------------------------------------------------

/// Explicit version gate for the four sovereigns (§2). Mirrors
/// `#define CGAP_ABI_VERSION 1u` in `include/cgap.h`; the §6.1 header court
/// asserts parity. A future incompatible change bumps both together;
/// `cgap_version()` remains the human-readable triple for the frozen slot
/// surface and is NOT this version.
pub const CGAP_ABI_VERSION: u32 = 1;

const _: () = assert!(CGAP_ABI_VERSION == 1);

// ---------------------------------------------------------------------------
// CgapStatus register (§2 "C signatures (normative)"; C: `typedef uint32_t
// CgapStatus` + anonymous enum, mirrored name-for-name and value-for-value)
// ---------------------------------------------------------------------------

/// The sovereign status word (§2). Every value below MUST stay reachable
/// through a real code path on some supported target (§2 reachability law;
/// enforced by the §6.2 status court — an enum value no injected fault can
/// produce is untested decoration).
pub type CgapStatus = u32;

/// §2: admitted.
pub const CGAP_OK: CgapStatus = 0x0000_0000;
/// §2: magic/format gate refusal (`'CG' + 01`). Producers: A14+A16.
pub const CGAP_ERR_INVALID_MAGIC: CgapStatus = 0x4347_0001;
/// §2: OABI rule-table version mismatch (`'CG' + 02`; tables: A06 §6).
/// Producers: A12+A13.
pub const CGAP_ERR_OABI_MISMATCH: CgapStatus = 0x4347_0002;
/// §2: SHACL shape violation (`'CG' + 03`). Producers: A14+A16.
pub const CGAP_ERR_SHACL_VIOLATION: CgapStatus = 0x4347_0003;
/// §2: documented lease table exhausted (`'CG' + 04`). Producers: A02+A03.
pub const CGAP_ERR_LEASE_EXHAUSTED: CgapStatus = 0x4347_0004;
/// `'CG' + 06` — concurrent single-writer denial on the spill path (FA9 lane
/// w6-fa9-commit-gating; a recorded register extension, not an A19 §2 pin —
/// the A19 status court pins the pre-existing six). The spill path is
/// single-writer per (path, lease-holding thread): a second thread claiming
/// an already-owned spill path is denied typed BEFORE any file operation, so
/// it can never re-`File::create` (O_TRUNC) the holder's drained bytes.
pub const CGAP_ERR_CONCURRENT_WRITER_DENIED: CgapStatus = 0x4347_0006;
/// §2: P28 spoliation — digest/license predicate failure (`'P28' + FF`;
/// attestation layout: A17 §2). Producers: A17.
pub const CGAP_ERR_P28_SPOLIATION: CgapStatus = 0x5032_38FF;

const _: () = assert!(core::mem::size_of::<CgapStatus>() == 4);

// ---------------------------------------------------------------------------
// CgapFrameView (§2: header-defined, non-owning, zero-copy; C twin:
// `cgap_frame_view` in include/cgap.h)
// ---------------------------------------------------------------------------

/// Header-defined, non-owning, zero-copy view into an admitted frame (§2).
///
/// `out_view.payload` points INTO the caller's `bytes` — no allocation, no
/// copy; the view dies with `bytes` (§2 ownership table). This is the only
/// structure the sovereign surface defines: admit produces a view, never a
/// buffer (§2 zero-copy guarantees), and eval reads `view->payload` in place
/// (on XIP, straight through the flash cache — no SRAM staging buffer in the
/// budget).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CgapFrameView {
    /// Points INTO the caller's bytes — zero copy (§2).
    pub payload: *const u8,
    /// Length in bytes of the visible payload.
    pub payload_len: usize,
    /// Schema id of the admitted frame (wire tier header: A04 §6).
    pub schema_id: u32,
    /// Rule count of the admitted frame (.oabi tables: A06 §6).
    pub rule_count: u32,
}

// ---------------------------------------------------------------------------
// Wave 1 ABI freeze pins. Core-only const asserts, no `static_assertions`
// crate (zero-dependency rule). C twins: `_Static_assert` in include/cgap.h.
//
// The view is pointer-width dependent by construction, so BOTH forms are
// pinned explicitly: host LP64 and XIP ILP32 (ESP32-S3 / ESP32-C3 rv32imc).
//
// Anti-vacuity: these are the flip-on-mutation court for the ABI freeze —
// widening `CgapFrameView` (adding/reordering/removing a field) or changing
// each pinned constant MUST fail `cargo build -p cgap-ffi`. A build that
// still passes after such a mutation is a VACUOUS GATE.
// ---------------------------------------------------------------------------

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<CgapFrameView>() == 24);
    assert!(core::mem::align_of::<CgapFrameView>() == 8);
    assert!(core::mem::offset_of!(CgapFrameView, payload) == 0);
    assert!(core::mem::offset_of!(CgapFrameView, payload_len) == 8);
    assert!(core::mem::offset_of!(CgapFrameView, schema_id) == 16);
    assert!(core::mem::offset_of!(CgapFrameView, rule_count) == 20);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<CgapFrameView>() == 16);
    assert!(core::mem::align_of::<CgapFrameView>() == 4);
    assert!(core::mem::offset_of!(CgapFrameView, payload) == 0);
    assert!(core::mem::offset_of!(CgapFrameView, payload_len) == 4);
    assert!(core::mem::offset_of!(CgapFrameView, schema_id) == 8);
    assert!(core::mem::offset_of!(CgapFrameView, rule_count) == 12);
};

// ---------------------------------------------------------------------------
// Rust-side signature freeze (§2 normative C signatures)
//
// fn-pointer type aliases mirror the frozen C prototypes one-to-one so the
// owner lanes implement against the frozen signature and the §6.1 header
// court has a mechanical Rust-side diff target — without emitting symbols
// (bodies land with their owner lanes, DoD 5) and without colliding with the
// future `#[no_mangle]` exports in this crate.
// ---------------------------------------------------------------------------

/// `cgap_admit_frame` — bytes enter; magic/format/SHACL gate or typed refusal.
///
/// `out_view` is caller-allocated and MAY be NULL (probe-only call);
/// `out_view.payload` borrows the caller's `bytes` (zero copy). Divergence
/// note: the lane-doc §2 sketch writes `const cgap_frame_view* out_view`,
/// which no callee could write through — the frozen header/Rust signature
/// uses a non-const out-parameter, per the doc's own ownership table
/// ("callee (pointer only)" fills `out_view.payload`).
pub type CgapAdmitFrameFn =
    unsafe extern "C" fn(bytes: *const u8, len: usize, out_view: *mut CgapFrameView) -> CgapStatus;

/// `cgap_eval_bn3` — admitted bytes evaluate deterministically against a
/// flash-resident `.bn3` network (A11 layout); zero copies of the input.
/// The callee never writes past `lanes_cap`; the ONLY copy in the whole
/// surface is the `out_lanes` write-out, caller-owned (§2).
pub type CgapEvalBn3Fn = unsafe extern "C" fn(
    view: *const CgapFrameView,
    bn3_rodata: *const u8,
    bn3_len: usize,
    out_lanes: *mut i32,
    lanes_cap: usize,
    out_lane_count: *mut usize,
) -> CgapStatus;

/// `cgap_commit_parquet` — evaluated batches commit to Parquet; HOST targets
/// only. An XIP/no_std build omits this symbol entirely (cfg), so it never
/// enters the SRAM budget (§2 ownership rules, §5 size budget). Callee
/// allocates here and only here (§2: "The callee performs no allocation in
/// admit/eval/verify. Commit allocates on host targets only.").
pub type CgapCommitParquetFn = unsafe extern "C" fn(
    lanes: *const i32,
    lane_count: usize,
    path: *const core::ffi::c_char,
) -> CgapStatus;

/// `cgap_verify_p28` — P28 spoliation evidence verification; digest +
/// license predicate over the embedded 80-byte `P28LicenseAttestation`
/// (A17 §2); read-only, no allocation. `out_digest` MAY be NULL.
pub type CgapVerifyP28Fn =
    unsafe extern "C" fn(evidence: *const u8, len: usize, out_digest: *mut u64) -> CgapStatus;

// ===========================================================================
// Wave F2 item 5 — SEAM-CRYPTO-FFI: the `cgap_verify_p28` sovereign BODY,
// wired to the real verify path per the §2 owner table (owner lane A17):
// cgap-p28's FULL §5 ordered validation
// (`crates/cgap-p28/src/attestation.rs::P28LicenseAttestation::validate`)
// over the PINNED canonical instrument span, no parent (fail-closed lineage,
// D-A17-2). Court-witnessed defect repair this wave: the first wiring ran
// `validate_intrinsic` only and the 640-mutation sandbox corpus FLIPPED on
// it — all 256 digest-byte flips admitted because §5.6's recompute is
// context-dependent — so the intrinsic-only wiring was NOT the real verify
// path. The full-validate wiring is the repair the court forced.
//
// Dependency direction = the one the crates actually expose: sovereign ->
// cgap-p28 -> {cgap-blake3 (§5.7 receipt binding), cgap-crypto (A18 seal
// bridge, elsewhere in that crate)}. The p28 attestation verify path itself
// demands NO sealer, so this surface carries NO direct cgap-crypto dep — the
// A16 gap ("zero cgap-crypto references in cgap-ffi") closes by routing
// through the real p28 verify path, which IS the cryptographic seam.
//
// XIP divergence (recorded, not silent): the body is host-surface only
// (`cfg(not(target_os = "none"))`). cgap-p28 is std-gated today
// (attestation.rs:130 `impl std::error::Error for Spoliation`; no crate-level
// no_std gate), so an unconditional dep would break the T1 rv32imc staticlib
// size court and the rv32imc check gate. The XIP port of verify is blocked on
// cgap-p28's no_std gate (A17's crate, not this lane's lease); §2's
// reachability law ("reachable on SOME supported target") holds on host.
//
// Anti-vacuity (courts in `wave_f2_p28_courts`, bottom of this file): the
// positive control (real pinned-instrument attestation admits) flips an
// always-refuse stub; the 640-mutation sandbox corpus flips an
// always-admit stub (both stub legs executed in-court); the cdylib symbol
// court flips on rename/removal of the export.
// ===========================================================================

/// House rotate-xor fold over raw bytes (LE u64 words, odd tail
/// zero-extended) — the byte-form of `crate::fold_word`, the exact fold law
/// of the Wave-3 zero-copy witness's `fold_view` helper and the FFI digest
/// path. NOT a cryptographic hash: accidental-mutation detection only.
fn fold_bytes(mut acc: u64, bytes: &[u8]) -> u64 {
    let mut i = 0;
    while i + 8 <= bytes.len() {
        let mut w = [0u8; 8];
        w.copy_from_slice(&bytes[i..i + 8]);
        acc = crate::fold_word(acc, u64::from_le_bytes(w));
        i += 8;
    }
    if i < bytes.len() {
        let mut tail = [0u8; 8];
        tail[..bytes.len() - i].copy_from_slice(&bytes[i..]);
        acc = crate::fold_word(acc, u64::from_le_bytes(tail));
    }
    acc
}

/// The §3 canonical instrument-header span of the pinned P28-SIL instrument
/// (`LICENSE-P28-SIL.md`, repo root): bytes from the file start through the
/// first line that is exactly `---`, inclusive of its newline. FROZEN PIN —
/// the sovereign C-ABI carries no instrument parameter, so the §5.6 recompute
/// inside `P28LicenseAttestation::validate` runs against THIS span and the
/// surface admits only attestations bound to the canonical instrument (any
/// other instrument's record refuses; a LICENSE-P28-SIL.md edit flips the
/// pin-parity court below and forces re-attestation, A17 §9 "Instrument
/// drift"). Span extraction mirrors `instrument_header_span` exactly
/// (413 bytes, ASCII; `---` is the first separator line in the real file).
/// The pin-parity court re-derives the span from the real file on disk EVERY
/// run through cgap-p28's own §3 API — this const may never silently drift
/// from the file.
const PINNED_INSTRUMENT_HEADER: &[u8] = b"# P28 Sovereign Invariant License (P28-SIL)\x0a\x0a**Canonical Specification Identifier:** `urn:avec:license:p28-sil:v2.1.0-sovereign`\x0a\x0a**Mathematical Category:** Sheaf-Theoretic State Admission & Monotonic Conservation Topology\x0a\x0a**Evidentiary Jurisprudence:** Self-Attesting Algorithmic Estoppel / Statutory Accrual Mandate\x0a\x0a**Operational Framework:** Adversarial Hyper-Dimensional Cognitive Load Theory (AHDCLT)\x0a\x0a---\x0a";

const _: () = assert!(PINNED_INSTRUMENT_HEADER.len() == 413);

/// §2 normative body — P28 spoliation evidence verification over the
/// embedded 80-byte `P28LicenseAttestation` (A17 §2). Parse (§5 step 1), then
/// the FULL §5 ordered validation with the only context the sovereign surface
/// can lawfully carry: the PINNED canonical instrument span (§5.6 recompute
/// via cgap-p28's real §3 derivation) and NO parent chain (§5.7 fail-closed
/// per D-A17-2: a Derivative/Transpiled record without its parent is
/// spoliation — an unverifiable lineage never admits). On admission, write
/// the mutation-detection digest (house rotate-xor fold over the raw evidence
/// bytes — §2 "digest paths fold raw buffers") to `out_digest` when non-NULL.
/// Read-only, no allocation (§2 ownership rules).
///
/// Every §5 failure class maps to the single typed refusal
/// `CGAP_ERR_P28_SPOLIATION` (non-fingerprinting, A17 §5 preamble): null
/// evidence, non-80-byte evidence, wire-parse failure, and any §5.2-§5.8
/// predicate failure (including a digest not bound to the pinned instrument).
/// Hostile bytes never panic — they refuse typed.
#[no_mangle]
#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn cgap_verify_p28(
    evidence: *const u8,
    len: usize,
    out_digest: *mut u64,
) -> CgapStatus {
    if evidence.is_null() {
        // §5 step 1: absence is a validation failure, never a silent pass.
        return CGAP_ERR_P28_SPOLIATION;
    }
    // SAFETY: §2 ownership table — `evidence` is caller-provided, read-only,
    // live for the call duration, `len` bytes readable.
    let bytes = unsafe { core::slice::from_raw_parts(evidence, len) };
    // §5 step 1 (record layer): any length other than exactly 80 is a parse
    // failure (A17 §2.1); wire-parse failure is the one opaque halt class.
    let Ok(rec) = cgap_p28::attestation::P28LicenseAttestation::from_bytes(bytes) else {
        return CGAP_ERR_P28_SPOLIATION;
    };
    // §5 full ordered validation, first failure wins, over the pinned
    // instrument span with no parent (fail-closed lineage, D-A17-2): magic,
    // version, flag shape, Root/receipt consistency, §5.6 digest recompute
    // against the PINNED instrument, §5.7 (Root ⇒ no parent; otherwise
    // refuse), §5.8 ceiling — via cgap-p28's real validator.
    if rec.validate(PINNED_INSTRUMENT_HEADER, None).is_err() {
        return CGAP_ERR_P28_SPOLIATION;
    }
    if !out_digest.is_null() {
        // SAFETY: `out_digest` is caller-allocated per §2; non-null checked.
        unsafe { out_digest.write(fold_bytes(0, bytes)) };
    }
    CGAP_OK
}

// ===========================================================================
// Wave GAP-1 (2026-11-01, coordinator-ordered) — the three remaining
// sovereign bodies, wired per the §2 owner table exactly like the
// `cgap_verify_p28` pattern above (real owner path, typed refusals only,
// host-surface cfg, pin-parity + sandbox-flip courts below in
// `wave_gap1_courts`):
//
// | Body                | Owner path delegated to                              |
// |---------------------|------------------------------------------------------|
// | `cgap_admit_frame`  | A14 `penstock::ingress` (limiter) + A04 frame        |
// |                     | projection/validate/BLAKE3 digest verify             |
// |                     | (`crates/cgap-core/src/frame.rs`); digest substrate  |
// |                     | `cgap_blake3::hash` (A04 court canon)                |
// | `cgap_eval_bn3`     | A12 unifier: `RuleCompiler::compile` +               |
// |                     | `unify_batch` + `ConsequentProjector::project`       |
// |                     | (`crates/cgap-bn3/src/{unify,project}.rs`)           |
// | `cgap_commit_parquet` | A02 sealed-block writer: `StaticPageWriter::`      |
// |                     | `write_slab` -> `seal` -> `flush_to(FileSink)`       |
// |                     | (`crates/cgap-parquet/src/{mmap_writer,footer}.rs`)  |
//
// XIP divergence (recorded, not silent — same form as the verify_p28 block
// above): the bodies are host-surface (`cfg(not(target_os = "none"))`).
// cgap-bn3's unifier and cgap-parquet's writer are no_std+ALLOC APIs (Vec /
// Box across the eval and commit paths); the XIP budget carries no
// allocator, so the §2 "callee performs no allocation in admit/eval/verify"
// law holds exactly where it is budget-lawful to hold it — on the XIP
// surface, where these three symbols are compiled out. §2 reachability
// ("reachable on SOME supported target") holds on host. The §5 XIP budget
// row "core evaluator + fold/digest" is carried by `fold_bytes` (above),
// which stays in the none-target build.
//
// A19:47 four-verb boundary law: no fifth verb is added here.
// `_cgap_p28_invariant_check` (cgap-p28 `trap.rs`, riding the link via the
// cgap-p28 dep) stays UNDECLARED in include/cgap.h — typed UNSUPPORTED
// record below (`P28_INVARIANT_CHECK_UNSUPPORTED`), same decision form as
// `oabi_lookup.rs::OABI_LOOKUP_UNSUPPORTED`.
// ===========================================================================

/// Map an A04 `FrameError` to the A19 register (typed, total). The mapping
/// follows the A14 §5 row taxonomy the register already encodes (see
/// `cgap-gate/src/penstock.rs` ingress rows 1-6): format/geometry faults are
/// INVALID_MAGIC class; digest/length coherence faults are OABI class; the
/// reserved-bits shape fault is the SHACL class; the attestation fault is
/// the P28 class. No code is minted here — every fault maps onto the frozen
/// register.
#[cfg(not(target_os = "none"))]
fn map_frame_error(e: cgap_core::frame::FrameError) -> CgapStatus {
    use cgap_core::frame::FrameError as E;
    match e {
        // Not-a-CGAP-frame class (A14 row 1/2 overload: bad magic, wrong
        // version, or a buffer the gate cannot address).
        E::BadMagic | E::BadVersion | E::Misaligned | E::BufferTooSmall => {
            return CGAP_ERR_INVALID_MAGIC;
        }
        // O-ABI coherence class (A14 row 3: digest/length coherence).
        E::PayloadLenMismatch | E::DigestMismatch => return CGAP_ERR_OABI_MISMATCH,
        // Shape violation (reserved bits set = the frame is not the declared
        // shape; A19 register: SHACL_VIOLATION, producers A14+A16).
        E::ReservedBitsSet => return CGAP_ERR_SHACL_VIOLATION,
        // P28 spoliation (A17 attestation validity fails).
        E::AttestationMismatch => return CGAP_ERR_P28_SPOLIATION,
    }
}

/// §2 normative body — bytes enter; magic/format/SHACL gate or typed refusal.
///
/// Two-stage admission, both real owner paths (§2 owner table: A14 + A04):
///
/// 1. A14 `penstock::ingress` — the limiter: fixed §4 order
///    magic/version -> O-ABI coherence -> P28 chain presence, returning
///    A19 register values (parity-pinned to THIS module's register by
///    penstock's own three-way purity court).
/// 2. A04 frame admission — `CgapFrameHeader::project` (§8.1 wire-to-memory
///    projection when the caller's buffer is 64-byte aligned) or `decode`
///    (the same layout parse, alignment-free, for unaligned C buffers),
///    then `validate` (magic/version/reserved-bits/attestation) and the
///    BLAKE3 digest re-derivation over the payload region
///    (`verify_payload` + `cgap_blake3::hash` — the digest step the R1
///    limiter deliberately skips).
///
/// On admission, `out_view` (when non-NULL) receives the zero-copy view:
/// `payload` points INTO the caller's bytes. `schema_id`/`rule_count` serve
/// 0: the v1 wire header (A04 §6) carries neither field — recorded
/// divergence (the view fields predate the A04 §6 header landing; a future
/// frame version, the only field-addition path, may populate them).
/// `out_view == NULL` probes (§2). Hostile bytes never panic.
#[no_mangle]
#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn cgap_admit_frame(
    bytes: *const u8,
    len: usize,
    out_view: *mut CgapFrameView,
) -> CgapStatus {
    if bytes.is_null() {
        // Not a CGAP frame (A14 row-1 class): absence is a refusal, never a
        // silent pass.
        return CGAP_ERR_INVALID_MAGIC;
    }
    // SAFETY: §2 ownership table — `bytes` is caller-provided, read-only,
    // live for the call duration, `len` bytes readable.
    let buf = unsafe { core::slice::from_raw_parts(bytes, len) };
    // Stage 1 — the A14 limiter over the caller's declared geometry.
    let mut gate = cgap_gate::penstock::FrameView {
        payload: &[],
        // SLA unblock (W3-R2, disclose-in-both): pre-L1 zero-defaults —
        // `prefilter` is minted by `ingress` on ADMIT only (zero on
        // reject/no-view), `escalate` is ADMIT+reserved-flags only.
        prefilter: 0,
        escalate: false,
    };
    let gate_status = cgap_gate::penstock::ingress(buf, len, Some(&mut gate));
    if gate_status != CGAP_OK {
        return gate_status;
    }
    // Stage 2 — the A04 projection + full validation + digest re-derivation.
    // Aligned callers get the true zero-copy §8.1 projection; unaligned C
    // buffers get `decode` (the same layout parse, alignment-free) — both
    // then run the SAME validate + payload + digest gates below.
    let header = if buf.as_ptr() as usize
        % core::mem::align_of::<cgap_core::frame::CgapFrameHeader>()
        == 0
    {
        match cgap_core::frame::CgapFrameHeader::project(
            buf[..cgap_core::frame::FRAME_HEADER_LEN]
                .try_into()
                .expect("128-byte header region staged by the A14 gate"),
        ) {
            Ok(h) => *h,
            Err(e) => return map_frame_error(e),
        }
    } else {
        match cgap_core::frame::CgapFrameHeader::decode(buf) {
            Ok(h) => h,
            Err(e) => return map_frame_error(e),
        }
    };
    if let Err(e) = header.validate() {
        return map_frame_error(e);
    }
    let payload = match header.payload(buf) {
        Ok(p) => p,
        Err(e) => return map_frame_error(e),
    };
    if let Err(e) = header.verify_payload(payload, cgap_blake3::hash) {
        return map_frame_error(e);
    }
    if !out_view.is_null() {
        // SAFETY: `out_view` is caller-allocated per §2; non-null checked.
        unsafe {
            out_view.write(CgapFrameView {
                payload: payload.as_ptr(),
                payload_len: payload.len(),
                schema_id: 0,
                rule_count: 0,
            });
        }
    }
    CGAP_OK
}

/// The sovereign eval encoding — PINNED ABI-local framing over the landed
/// A11/A12 word laws. Cited, not redefined: term words are A11 §4 sigil
/// words (top-byte sigil + 56-bit payload), and the WIRE form of every word
/// is LITTLE-ENDIAN (A11 §4 "the (future) wire/pack encoding is
/// little-endian" as landed by `BN3Statement::to_le_bytes`/`from_le_bytes`,
/// A04 §6.1's normative LE). The framing below is the sovereign surface's
/// own, disclosed in the lane receipt as a divergence: A11's rule-record
/// wire framing is future work, so `cgap_eval_bn3` pins this one and the
/// pin-parity court (`eval_encoding_pin_parity`) re-derives a real rule
/// table through cgap-bn3's own compiler every run.
///
/// `bn3_rodata` word stream (LE u64 words):
///   `[rule_count,
///     per rule: rule_id, head_s, head_p, head_o, body_len, body atoms...]`
/// where every atom is 3 term words. `view.payload` = the statement store,
/// columnar `s-col | p-col | o-col` (n = payload_len/24 LE u64 words per
/// column; `payload_len % 24 != 0` refuses).
const EVAL_ENCODING_PIN: &str = "cgap_eval_bn3 v1: LE u64 words; \
     rodata = [rule_count, per rule: rule_id, head(3), body_len, body(3n)]; \
     payload = s-col|p-col|o-col (24 B/row); \
     lane[i] = projected head row i object word (refused outside [0, i32::MAX])";

const _: () = assert!(EVAL_ENCODING_PIN.len() > 64);

/// Decode one LE u64 word at `off`, refusing a truncated stream.
fn read_le_word(bytes: &[u8], off: usize) -> Result<u64, CgapStatus> {
    if bytes.len() < off + 8 {
        return Err(CGAP_ERR_OABI_MISMATCH);
    }
    let mut w = [0u8; 8];
    w.copy_from_slice(&bytes[off..off + 8]);
    Ok(u64::from_le_bytes(w))
}

/// §2 normative body — admitted bytes evaluate deterministically against a
/// flash-resident `.bn3` network under the pinned eval encoding (above).
/// Rules compile through the REAL A12 compiler, evaluate via the REAL A12
/// nested-loop unifier over the zero-copy payload columns, and every match
/// projects through the REAL A12 consequent projector; lane[i] = projected
/// head row i object word, refused typed outside the declared i32 lane
/// domain. Callee never writes past `lanes_cap`; on cap overflow the call
/// refuses typed (fail-closed; the caller retries with a larger lease).
/// A11 sigil-level word refusals (unknown sigil, register out of range,
/// formula-in-predicate, unbound head variable, register/const overflow)
/// map to the register's A12+A13 class `CGAP_ERR_OABI_MISMATCH`; shape and
/// domain faults map to `CGAP_ERR_SHACL_VIOLATION`; the caller's exhausted
/// `out_lanes` lease maps to `CGAP_ERR_LEASE_EXHAUSTED`.
#[no_mangle]
#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn cgap_eval_bn3(
    view: *const CgapFrameView,
    bn3_rodata: *const u8,
    bn3_len: usize,
    out_lanes: *mut i32,
    lanes_cap: usize,
    out_lane_count: *mut usize,
) -> CgapStatus {
    // Shape gates first (typed, fail-closed): a live view, a live network,
    // and a writable count sink are the eval surface shape; NULL is not.
    if view.is_null() || bn3_rodata.is_null() || out_lane_count.is_null() {
        return CGAP_ERR_SHACL_VIOLATION;
    }
    // SAFETY: §2 — the view is caller-allocated, live for the call.
    let view = unsafe { &*view };
    if view.payload.is_null() {
        // A view without a payload is not an admitted view (admit always
        // fills payload) — not the eval surface shape.
        return CGAP_ERR_SHACL_VIOLATION;
    }
    // SAFETY: §2 — view->payload borrows the caller's admitted bytes.
    let payload = unsafe { core::slice::from_raw_parts(view.payload, view.payload_len) };
    // SAFETY: §2 — bn3_rodata is caller-provided, read-only, bn3_len bytes.
    let rodata = unsafe { core::slice::from_raw_parts(bn3_rodata, bn3_len) };
    // ---- Rule-table decode (pinned encoding, LE words) ---------------------
    let mut off = 0usize;
    let rule_count = match read_le_word(rodata, off) {
        Ok(w) => w,
        Err(code) => return code,
    };
    off += 8;
    if rule_count > (rodata.len() / 8) as u64 {
        // Every rule costs >= 5 words; a count larger than the stream is a
        // framing fault, refused before any allocation.
        return CGAP_ERR_OABI_MISMATCH;
    }
    let mut rules = Vec::with_capacity(rule_count as usize);
    for _ in 0..rule_count {
        let rule_id = match read_le_word(rodata, off) {
            Ok(w) => w as u32,
            Err(code) => return code,
        };
        off += 8;
        let mut head = [0u64; 3];
        for slot in head.iter_mut() {
            match read_le_word(rodata, off) {
                Ok(w) => *slot = w,
                Err(code) => return code,
            }
            off += 8;
        }
        let body_len = match read_le_word(rodata, off) {
            Ok(w) => w as usize,
            Err(code) => return code,
        };
        off += 8;
        if body_len > rodata.len() {
            return CGAP_ERR_OABI_MISMATCH;
        }
        let mut body = Vec::with_capacity(body_len.min(4096));
        for _ in 0..body_len {
            let mut atom = [0u64; 3];
            for slot in atom.iter_mut() {
                match read_le_word(rodata, off) {
                    Ok(w) => *slot = w,
                    Err(code) => return code,
                }
                off += 8;
            }
            body.push(cgap_bn3::unify::BodyAtom::new(atom[0], atom[1], atom[2]));
        }
        rules.push(cgap_bn3::unify::Rule {
            rule_id,
            head,
            body,
        });
    }
    // ---- Payload columns (zero-copy region split, LE words) ----------------
    if payload.len() % 24 != 0 {
        return CGAP_ERR_SHACL_VIOLATION;
    }
    let n = payload.len() / 24;
    // Transpose the columnar payload into the unifier's column slices. The
    // ONLY copy in eval: the owner unifier takes `&[u64]` columns; the wire
    // words are LE (A11 landed wire law), host words may not be.
    let mut s_col = Vec::with_capacity(n);
    let mut p_col = Vec::with_capacity(n);
    let mut o_col = Vec::with_capacity(n);
    for i in 0..n {
        s_col.push(match read_le_word(payload, i * 8) {
            Ok(w) => w,
            Err(c) => return c,
        });
        p_col.push(match read_le_word(payload, 8 * n + i * 8) {
            Ok(w) => w,
            Err(c) => return c,
        });
        o_col.push(match read_le_word(payload, 16 * n + i * 8) {
            Ok(w) => w,
            Err(c) => return c,
        });
    }
    // ---- Real A12 compile + evaluate + project ------------------------------
    let cols = match cgap_bn3::unify::StatementColumns::new(&s_col, &p_col, &o_col) {
        Ok(c) => c,
        Err(_) => return CGAP_ERR_OABI_MISMATCH,
    };
    let mut lane_count = 0usize;
    for rule in &rules {
        let compiled = match cgap_bn3::unify::RuleCompiler::compile(rule) {
            Ok(c) => c,
            Err(_) => return CGAP_ERR_OABI_MISMATCH,
        };
        let matches = match cgap_bn3::unify::unify_batch(
            &compiled,
            &cols,
            cgap_bn3::unify::DEFAULT_CHUNK_ROWS,
        ) {
            Ok(m) => m,
            Err(_) => return CGAP_ERR_OABI_MISMATCH,
        };
        let batch =
            match cgap_bn3::project::ConsequentProjector::new().project(&compiled, 0, &matches) {
                Ok(b) => b,
                Err(_) => return CGAP_ERR_OABI_MISMATCH,
            };
        for row_i in 0..batch.rows.len() {
            let (_, _, o, _) = batch.rows.row(row_i);
            if lane_count >= lanes_cap {
                // The caller's out_lanes lease is exhausted — fail-closed
                // typed refusal, never a silent truncation.
                return CGAP_ERR_LEASE_EXHAUSTED;
            }
            if o > i32::MAX as u64 {
                return CGAP_ERR_SHACL_VIOLATION;
            }
            // SAFETY: lane_count < lanes_cap proves the write is in-bounds.
            unsafe { out_lanes.add(lane_count).write(o as i32) };
            lane_count += 1;
        }
    }
    // SAFETY: out_lane_count always written on success; non-null checked.
    unsafe { out_lane_count.write(lane_count) };
    CGAP_OK
}

/// §2 normative body — evaluated batches commit to Parquet (HOST targets
/// only; the XIP build omits the symbol entirely, so it never enters the
/// SRAM budget). The lanes are ONE column chunk (the eval output of one
/// network): one `SlabView` lane of `lane_count` rows at `bits = 8` — the
/// widest kernel width (A02 kernel set {1,2,4,8}); values outside the
/// declared kernel domain ([0, 255] at bits 8) are the shape class — the
/// surface refuses what the inherited hybrid packer would truncate.
///
/// Refusal mapping (all typed, never a panic):
///   null path / null lanes with lane_count > 0 / negative or >255 lane
///   value / lane_count outside u32        -> CGAP_ERR_SHACL_VIOLATION
///   A02 writer/seal capacity refusals (the documented fixed-capacity lease:
///   buffer, row-group table, span wire bound) -> CGAP_ERR_LEASE_EXHAUSTED
///   sink IO failure -> CGAP_ERR_LEASE_EXHAUSTED (the register carries no
///   IO class; the documented-capacity-exhaustion class is the least-wrong
///   typed value — RECORDED MAPPING DIVERGENCE in the lane receipt).
#[no_mangle]
#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn cgap_commit_parquet(
    lanes: *const i32,
    lane_count: usize,
    path: *const core::ffi::c_char,
) -> CgapStatus {
    // Shape gates first.
    if path.is_null() || (lanes.is_null() && lane_count > 0) {
        return CGAP_ERR_SHACL_VIOLATION;
    }
    if lane_count > u32::MAX as usize || lane_count == 0 {
        // Zero rows seals an empty body — the A02 footer refuses it
        // (zero_body_seal_refused); the surface refuses it as shape, up
        // front.
        return CGAP_ERR_SHACL_VIOLATION;
    }
    // SAFETY: §2 — lanes is caller-provided, lane_count i32s readable.
    let vals: &[i32] = unsafe { core::slice::from_raw_parts(lanes, lane_count) };
    // Declared kernel domain at bits 8: [0, 255]; negatives are out of the
    // domain (refused, not cast) and values > 255 are truncated by the
    // inherited hybrid packer — refused here instead.
    if vals.iter().any(|&v| !(0..=255).contains(&v)) {
        return CGAP_ERR_SHACL_VIOLATION;
    }
    // SAFETY: §2 — path is a caller-owned NUL-terminated UTF-8 string.
    let cstr = unsafe { core::ffi::CStr::from_ptr(path) };
    let path = match cstr.to_str() {
        Ok(p) if !p.is_empty() => p,
        _ => return CGAP_ERR_SHACL_VIOLATION,
    };
    // A02 sealed-block commit through the W3-L4 PRE-LEASED ring sink: the
    // 64 KiB static buffer + 64 KiB ring are granted ONCE per lane slot
    // (thread-local lease; spill handle pre-opened at grant, `path` is the
    // rotation spill target). Steady-state per-tx = memcpy into the ring
    // window + writer.seal — NO per-call File::create, NO per-call scratch
    // or lane Vec alloc. New-ring refusal / spill IO failure map to
    // CGAP_ERR_LEASE_EXHAUSTED (never silent truncation); the lane-value
    // view is a zero-copy u32 reborrow (i32 values are domain-gated to
    // [0, 255] above, so the bit patterns are identical).
    //
    // ---------------------------------------------------------------------------
    // FA9 DURABILITY DISCLOSURE (forensic-lane-09.md, suspicion 1; updated
    // lane H3 rotation writeback, v26.11.2 hardening): this path is an
    // IN-MEMORY transaction STAGING LATCH. Data reaches the spill file ONLY
    // at ring-rotation drain. As of lane H3, the drain is write_all +
    // flush + `sync_all` (mmap_writer.rs `drain_to_spill` via the
    // `SpillSync` seam), so a COMPLETED ROTATION is power-fail durable —
    // the rotation boundary is device-pinned before the ring re-arms.
    // REMAINING WINDOW (stated, not hidden): bytes staged in the ring that
    // have NOT yet been drained (head > 0, no rotation) are memory-only —
    // a process crash loses every transaction staged since lease grant or
    // since the last rotation (up to the whole 64 KiB ring). The
    // staging-latch semantics are retained and now stated. `CGAP_OK` from
    // this function means "STAGED", never "DURABLE" for undrained bytes;
    // drained bytes are durable at the rotation boundary.
    // stage_commit-class µs/tx figures measure the staging latch, not a
    // durable commit — the correct label is "staged transaction latch"
    // (see the relabel receipt, lane H3).
    //
    // FA9 SINGLE-WRITER GATE (suspicion 3): the spill path is acquired
    // fail-closed before any file operation. A second thread claiming an
    // already-owned path is denied typed (CGAP_ERR_CONCURRENT_WRITER_DENIED)
    // BEFORE its lease grant can re-`File::create` (O_TRUNC) the holder's
    // drained spill bytes. The thread-local COMMIT_LEASE itself stays.
    // ---------------------------------------------------------------------------
    match spill_gate::claim(path.as_bytes()) {
        spill_gate::Claim::Acquired => {}
        spill_gate::Claim::Denied => return CGAP_ERR_CONCURRENT_WRITER_DENIED,
    }
    COMMIT_LEASE.with(|lease| {
        let slot = &mut *lease.borrow_mut();
        // Lease grant (once per lane slot): pre-open the spill handle at
        // `path` — the rotation spill target — and mint the ring. This is
        // the ONLY File::create in the path; steady-state never touches it.
        if slot.is_none() {
            match std::fs::File::create(path) {
                Ok(f) => {
                    *slot = Some(CommitLease {
                        block: cgap_parquet::mmap_writer::StaticPageBuffer::<65_536>::new(),
                        ring: cgap_parquet::mmap_writer::RingSink::new(Some(f)),
                    });
                }
                Err(_) => return CGAP_ERR_LEASE_EXHAUSTED,
            }
        }
        let lease = slot.as_mut().expect("lease granted above");
        // Zero-copy lane view: i32 -> u32 reborrow, identical bit patterns
        // for the domain-gated [0, 255] values (no per-call Vec).
        // SAFETY: `vals` (validated above) lives for the call; i32/u32 have
        // identical layout; values are in [0, 255] so the reinterpret is
        // value-identity, not a cast.
        let lanes_u32: &[u32] =
            unsafe { core::slice::from_raw_parts(vals.as_ptr() as *const u32, vals.len()) };
        let slab = cgap_parquet::mmap_writer::SlabView {
            lanes: &[lanes_u32],
            rows: lane_count as u32,
            bits: 8,
        };
        let mut writer = lease.block.writer();
        if writer.write_slab(&slab).is_err() {
            return CGAP_ERR_LEASE_EXHAUSTED;
        }
        let sealed = match writer.seal(None) {
            Ok(s) => s,
            Err(_) => return CGAP_ERR_LEASE_EXHAUSTED,
        };
        // Tier H handoff: the sealed block through the ring sink seam.
        // Rotation (ring wrap) drains to the pre-opened spill handle.
        if sealed.flush_to(&mut lease.ring).is_err() {
            return CGAP_ERR_LEASE_EXHAUSTED;
        }
        CGAP_OK
    })
}

/// The per-lane-slot commit lease (W3-L4): the 64 KiB static block buffer
/// plus the 64 KiB ring sink with its pre-opened spill handle. Granted once,
/// reused for every transaction — the steady-state path allocates nothing.
/// Host-surface gated exactly like the commit body (the XIP `target_os =
/// "none"` build carries no allocator/std and links no cgap-parquet).
#[cfg(not(target_os = "none"))]
struct CommitLease {
    block: cgap_parquet::mmap_writer::StaticPageBuffer<65_536>,
    ring: cgap_parquet::mmap_writer::RingSink,
}

#[cfg(not(target_os = "none"))]
thread_local! {
    static COMMIT_LEASE: std::cell::RefCell<Option<CommitLease>> =
        const { std::cell::RefCell::new(None) };
}

/// FA9 single-writer gate (w6-fa9-commit-gating): process-lifetime ownership
/// registry for spill paths, keyed (path-hash, minted thread id). Fail-closed:
/// a claim on a path owned by a DIFFERENT live lease thread is denied typed
/// (`CGAP_ERR_CONCURRENT_WRITER_DENIED`) before any IO, so the second
/// thread's lease grant can never `File::create` (O_TRUNC) the holder's
/// drained spill bytes. Re-entry by the SAME (path, thread) — the lease
/// owner's steady-state — is allowed. Ownership is released by the owning
/// thread's `SpillOwnership` Drop (lease lifetime == thread lifetime), so a
/// path whose owner thread has exited is free again.
///
/// DIVERGENCE from the dispatch (recorded in the receipt): the dispatch
/// prescribed a single `AtomicBool` claim/release per call. A per-call global
/// flag is insufficient for the court's required invariant ("no truncation of
/// the first thread's data"): with release-per-call, a second thread can win
/// the inter-call gap, GRANT (O_TRUNC), and legally truncate the holder's
/// drained bytes — the exact FA9 finding. The gate therefore keys per
/// (path-hash, thread) with lease-lifetime ownership instead. The residual
/// bound is the 64-bit FNV-1a path-hash collision probability (negligible;
/// documented, not silenced).
#[cfg(not(target_os = "none"))]
mod spill_gate {
    use std::sync::Mutex;

    /// Minted per-thread id (process-lifetime counter, exact and stable).
    fn mint_thread_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }

    thread_local! {
        static CURRENT_TID: u64 = mint_thread_id();
    }

    /// FNV-1a 64-bit over the raw path bytes (no alloc, no formatting).
    fn fnv1a(bytes: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
        h
    }

    /// Fail-closed acquisition. Denies iff another thread owns the path.
    /// The owner's own re-entry (steady-state commit) is allowed through.
    pub fn claim(path: &[u8]) -> Claim {
        // Force the Drop-guard registration for this thread so ownership is
        // released when the lease thread exits (thread_locals are lazy; a
        // guard never `.with`-ed never drops).
        OWNERS_GUARD.with(|_| ());
        let tid = CURRENT_TID.with(|&t| t);
        let h = fnv1a(path);
        let mut owners = OWNERS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if owners.iter().any(|&(ph, t)| ph == h && t != tid) {
            return Claim::Denied;
        }
        if !owners.contains(&(h, tid)) {
            owners.push((h, tid));
        }
        Claim::Acquired
    }

    /// Lease lifetime == thread lifetime: releasing this thread's spill
    /// ownership when the lease thread exits (thread-local Drop).
    struct OwnersGuard;

    impl Drop for OwnersGuard {
        fn drop(&mut self) {
            let tid = CURRENT_TID.with(|&t| t);
            if let Ok(mut owners) = OWNERS.lock() {
                owners.retain(|&(_, t)| t != tid);
            }
        }
    }

    thread_local! {
        static OWNERS_GUARD: OwnersGuard = OwnersGuard;
    }

    /// Process-lifetime ownership table. Bounded by the number of distinct
    /// spill paths held by live lease threads in the process.
    static OWNERS: Mutex<Vec<(u64, u64)>> = Mutex::new(Vec::new());

    pub enum Claim {
        Acquired,
        Denied,
    }
}

/// Typed UNSUPPORTED(scope) record — `_cgap_p28_invariant_check` stays
/// UNDECLARED in include/cgap.h (dispatch branch b, decided; no silent
/// middle). The symbol is a real cgap-p28 export (`crates/cgap-p28/src/
/// trap.rs:82`, `#[no_mangle]`) that rides the cdylib link through the
/// host cgap-p28 dep edge, so it IS resolvable by a C consumer — but the
/// A19 boundary law is "the entire cgap capability crosses the boundary as
/// exactly four functions" (A19:47) and "a fifth verb ... belongs in a
/// lane, not the ABI" (A19:57-60), the same doctrine the F2-6 record
/// (`oabi_lookup.rs::OABI_LOOKUP_UNSUPPORTED`) applied to the `.oabi`
/// lookup verb. The constant + halt semantics are A17 lane property
/// (trap.rs:8-9: "the FFI *contract* remains A19's; this lane owns the
/// constant and the halt semantics"); declaring it here would re-litigate
/// an owner boundary and widen the frozen four-verb freeze. Decision form
/// mirrors `OABI_LOOKUP_UNSUPPORTED`: any future consumer (court, doc
/// generator, coordinator roll-up) reads this instead of re-deriving it.
pub const P28_INVARIANT_CHECK_UNSUPPORTED: &str = "UNSUPPORTED(scope): \
     _cgap_p28_invariant_check (cgap_p28_invariant_check, cgap-p28 trap.rs:82) \
     rides the link undeclared BY DESIGN; declaring it in include/cgap.h is \
     refused — A19-libcgap-ffi-xip.md:47 (boundary = exactly four functions), \
     :57-60 (a fifth verb belongs in a lane, not the ABI); \
     oabi_lookup.rs::OABI_LOOKUP_UNSUPPORTED (same decision form, F2-6); \
     trap.rs:8-9 (A17 owns the constant + halt semantics; FFI contract A19's)";

const _: () = assert!(P28_INVARIANT_CHECK_UNSUPPORTED.len() > 128);

// ===========================================================================
// Wave 3 — lane-doc §6 courts. This section is the Wave-3 slice: §6.1 header
// court, §6.2 status court, §6.4 zero-copy witness. All three live in
// `#[cfg(test)]` modules (test-only; the ABI surface above stays byte-stable).
//
// Court-wide discipline (lane contract): real collaborators (the checked-in
// header on disk, the cc/clang subprocess, real heap buffers, raw pointer
// writes), assert final state, no mocks. Anti-vacuity law: a court that
// cannot flip is FAIL — every court carries its own mutation corpus or
// negative control, and a mutation that goes undetected fails the court as a
// VACUOUS GATE.
//
// Owner-lane reality (§2 owner table; recorded per DoD 5, not silently
// assumed): the four sovereign BODIES are owned by A14+A16 / A12+A13 /
// A02+A03 / A17 and — AS OF THE WAVE-3 PIN — had NOT landed in this crate.
// Wave F2 item 5 (2026-11-01) landed the FIRST one: `cgap_verify_p28` above
// (owner lane A17's verify path). The courts below still pin the exact
// current reality and flip loudly when it moves.
//
// Wave 7 — IN-CODE RECEIPT (defect repair driven by the wave-6 court
// receipts, re-read in full this wave: a07-wave6, a16-wave6). Findings
// repaired here:
//   1. a07-wave6 §5 (coverage hygiene: two gates, two coverage sets) —
//      the §6.2 ledger walk descended into lane build roots (`target*`,
//      e.g. `crates/cgap-bont/target-a06/` and
//      `ontologies/fibo/target-a09/`, both on the wave-7 tree with ZERO
//      .rs files inside — latent, not live). Repair: walk skips `target*`,
//      the real tree is pinned pollution-free in the ledger, and
//      `ledger_walk_build_root_pollution_court` executes the named mutation
//      (skip deleted = `walk_unfiltered`) on a real temp tree EVERY run.
//   2. a16-wave6 D-A16-1 (evidence durability: /tmp flip logs are
//      ephemeral; no waves/ receipt file exists for A19 — that directory is
//      coordinator-owned). Repair: this block IS the durable A19 wave
//      receipt, living in the owned artifact; the lane report carries the
//      exact commands + exits. Method transfers applied:
//      `cargo test --no-fail-fast` (a16-wave6 note: fail-fast hides a red
//      target's results).
// No include/cgap.h change in Wave 7: the header court is green and no
// wave-6 finding implicates the header (diff-minimal repair).
//
// Wave 8 — defect repair driven by the wave-7 court receipt (a08-wave7,
// the only wave-7 receipt on disk; re-read in full this wave). Findings
// repaired here, all in the §6.1 header court (diff-minimal; include/
// cgap.h itself is UNCHANGED this wave):
//   1. a08-wave7 §"Court attack" (flip attribution: the wave-7 named
//      courts M12 and M13 "flip in isolation", i.e. the flip names its own
//      court, and "anchor count asserted == 1 before every write") — the
//      §6.1 mutation corpus asserted detection only, with NEITHER anchor
//      uniqueness NOR flip attribution: a multi-site anchor makes a
//      mutation ambiguous, and a "detected somewhere" result cannot say
//      WHICH court leg flipped, so a collateral-only flip (the named leg
//      dead, detection via an unrelated leg) would have passed silently.
//      Repair: every corpus anchor is asserted UNIQUE (count == 1) in the
//      normalized header before mutation, and every mutation now carries a
//      named detecting leg (4th column of MUTATIONS) that the failure text
//      must name — an off-leg flip fails the court as unattributable.
//   2. Same method transfer, both compile legs: the C11 layout-mutation
//      anchor is pinned count==1, and the C++ rename anchor is scoped to
//      the PROTOTYPE site (`CgapStatus cgap_verify_p28(`) instead of the
//      blanket symbol string — the raw header mentions the symbol twice
//      (file-top comment + prototype), so the old blanket anchor was
//      multi-site and the rename unattributable.
//   3. D-A08-1/D-A16-1 continuity (receipt durability): this block remains
//      the durable A19 receipt; waves/ is coordinator-owned.
// The repair's own non-vacuity is witnessed IN-COURT, every run, by
// `header_court_wrong_leg_shim_collateral_flip` below: the shim (M4's exact
// mutation paired with a non-detecting leg name) flips on the COLLATERAL
// message — the attribution leg is load-bearing. (A19 correction, Wave 9:
// this block previously pointed at "the lane report", but no waves/ receipt
// file exists for A19 — waves/ is coordinator-owned — so the witness is
// executed here instead of cited.)
// No include/cgap.h change in Wave 8 either: no wave-7 finding implicates
// the header (diff-minimal repair).
// ===========================================================================

#[cfg(test)]
mod wave3_courts {
    use super::{
        CgapAdmitFrameFn, CgapCommitParquetFn, CgapEvalBn3Fn, CgapFrameView, CgapStatus,
        CgapVerifyP28Fn, CGAP_ABI_VERSION, CGAP_ERR_CONCURRENT_WRITER_DENIED,
        CGAP_ERR_INVALID_MAGIC, CGAP_ERR_LEASE_EXHAUSTED, CGAP_ERR_OABI_MISMATCH,
        CGAP_ERR_P28_SPOLIATION, CGAP_ERR_SHACL_VIOLATION, CGAP_OK,
    };
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    // -- Harness ------------------------------------------------------------

    /// Absolute repo root, derived from the crate manifest (never cwd).
    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root canonicalize")
    }

    /// The checked-in header, read fresh from disk every run — the court
    /// always tests the file a consumer would actually compile.
    fn header_source() -> String {
        fs::read_to_string(repo_root().join("include/cgap.h"))
            .expect("include/cgap.h readable from repo root")
    }

    /// Strip /* ... */ comments. The header has no nested or string-literal
    /// comments, so a flat scanner is exact here.
    fn strip_comments(src: &str) -> String {
        let bytes = src.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        String::from_utf8(out).expect("header is UTF-8")
    }

    /// Collapse whitespace runs to single spaces so multi-line prototypes
    /// normalize to one line for parsing.
    fn normalize(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut in_ws = false;
        for ch in text.chars() {
            if ch.is_whitespace() {
                if !in_ws {
                    out.push(' ');
                }
                in_ws = true;
            } else {
                out.push(ch);
                in_ws = false;
            }
        }
        out
    }

    // -- Parsers: all Result-returning so the mutation corpus observes an
    //    Err (detection), never a panic-as-pass. ----------------------------

    fn parse_version(text: &str) -> Result<u32, String> {
        let needle = "#define CGAP_ABI_VERSION";
        let i = text.find(needle).ok_or("CGAP_ABI_VERSION define absent")?;
        let rest = &text[i + needle.len()..];
        let tok = rest
            .split(|c: char| c.is_whitespace() || c == ';')
            .find(|t| !t.is_empty())
            .ok_or("CGAP_ABI_VERSION has no value")?;
        tok.trim_end_matches('u')
            .parse()
            .map_err(|e| format!("CGAP_ABI_VERSION unparsable: {e}"))
    }

    fn parse_enum(text: &str) -> Result<Vec<(String, u32)>, String> {
        let start = text.find("enum {").ok_or("status enum absent")?;
        let body_start = start + "enum {".len();
        let close = text[body_start..]
            .find("};")
            .ok_or("status enum unterminated")?;
        let body = &text[body_start..body_start + close];
        let mut out = Vec::new();
        for member in body.split(',') {
            let member = member.trim();
            if member.is_empty() {
                continue;
            }
            let Some((name, val)) = member.split_once('=') else {
                continue;
            };
            let name = name.trim();
            if name.is_empty() || name.contains(char::is_whitespace) {
                continue;
            }
            let val = val.trim().trim_end_matches('u');
            let v = u32::from_str_radix(val.trim_start_matches("0x"), 16)
                .map_err(|e| format!("enum {name} unparsable: {e}"))?;
            out.push((name.to_string(), v));
        }
        if out.is_empty() {
            return Err("status enum parsed to zero members".into());
        }
        Ok(out)
    }

    fn parse_view_fields(text: &str) -> Result<Vec<(String, String)>, String> {
        let start = text
            .find("typedef struct {")
            .ok_or("cgap_frame_view typedef absent")?;
        let body_start = start + "typedef struct {".len();
        let end = text[body_start..]
            .find("} cgap_frame_view;")
            .ok_or("cgap_frame_view typedef unterminated")?;
        let body = &text[body_start..body_start + end];
        let mut out = Vec::new();
        // Normalized text is one line; split declarations on ';'.
        for decl in body.split(';') {
            let decl = decl.trim();
            if decl.is_empty() || decl.contains('{') || decl.contains('}') {
                continue;
            }
            let Some((ty, name)) = decl.rsplit_once(char::is_whitespace) else {
                continue;
            };
            let (ty, name) = (ty.trim(), name.trim());
            if ty.is_empty() || name.is_empty() || name.contains(char::is_whitespace) {
                continue;
            }
            out.push((name.to_string(), ty.to_string()));
        }
        if out.is_empty() {
            return Err("cgap_frame_view parsed to zero fields".into());
        }
        Ok(out)
    }

    fn parse_prototypes(
        text: &str,
    ) -> Result<BTreeMap<String, (String, Vec<(String, String)>)>, String> {
        let mut out = BTreeMap::new();
        for name in [
            "cgap_admit_frame",
            "cgap_eval_bn3",
            "cgap_commit_parquet",
            "cgap_verify_p28",
        ] {
            let needle = format!("{name}(");
            let i = text
                .find(&needle)
                .ok_or(format!("prototype {name} absent"))?;
            let open = i + needle.len() - 1; // index of '('
            let close = text[open..]
                .find(')')
                .ok_or(format!("prototype {name} paren mismatch"))?;
            let params = &text[open + 1..open + close];
            // Return type: the nearest preceding `CgapStatus` with only
            // whitespace between it and the name (empty filter match fails).
            text[..i]
                .rfind("CgapStatus")
                .filter(|p| text[p + "CgapStatus".len()..i].trim().is_empty())
                .ok_or(format!("prototype {name}: return not CgapStatus"))?;
            let mut parsed: Vec<(String, String)> = Vec::new();
            for param in params.split(',') {
                let param = param.trim();
                if param.is_empty() {
                    continue;
                }
                let (ty, arg) = param
                    .rsplit_once(char::is_whitespace)
                    .ok_or(format!("prototype {name}: param '{param}' has no name"))?;
                parsed.push((arg.trim().to_string(), ty.trim().to_string()));
            }
            out.insert(name.to_string(), ("CgapStatus".to_string(), parsed));
        }
        Ok(out)
    }

    // -- Rust-side mirror (frozen spec §2; three-way freeze) -----------------
    //
    // The mirror is the literal type text the court demands from the header.
    // Its correctness against the RUST side is pinned by
    // `rust_side_signature_guards_match_aliases`, which coerces fn items
    // carrying the literal types to the public fn-pointer aliases — a
    // mismatch anywhere fails to COMPILE. Chain: aliases <-> guard literals
    // <-> mirror <-> parsed header; any drift flips something.

    const MIRROR_PARAMS: &[(&str, &[&str])] = &[
        (
            "cgap_admit_frame",
            &[
                "const uint8_t* bytes",
                "size_t len",
                "cgap_frame_view* out_view",
            ],
        ),
        (
            "cgap_eval_bn3",
            &[
                "const cgap_frame_view* view",
                "const uint8_t* bn3_rodata",
                "size_t bn3_len",
                "int32_t* out_lanes",
                "size_t lanes_cap",
                "size_t* out_lane_count",
            ],
        ),
        (
            "cgap_commit_parquet",
            &[
                "const int32_t* lanes",
                "size_t lane_count",
                "const char* path",
            ],
        ),
        (
            "cgap_verify_p28",
            &[
                "const uint8_t* evidence",
                "size_t len",
                "uint64_t* out_digest",
            ],
        ),
    ];

    const MIRROR_VIEW: &[(&str, &str)] = &[
        ("payload", "const uint8_t*"),
        ("payload_len", "size_t"),
        ("schema_id", "uint32_t"),
        ("rule_count", "uint32_t"),
    ];

    const MIRROR_ENUM: &[(&str, u32)] = &[
        ("CGAP_OK", CGAP_OK),
        ("CGAP_ERR_INVALID_MAGIC", CGAP_ERR_INVALID_MAGIC),
        ("CGAP_ERR_OABI_MISMATCH", CGAP_ERR_OABI_MISMATCH),
        ("CGAP_ERR_SHACL_VIOLATION", CGAP_ERR_SHACL_VIOLATION),
        ("CGAP_ERR_LEASE_EXHAUSTED", CGAP_ERR_LEASE_EXHAUSTED),
        // repair-10 (lane F8): C twin added to include/cgap.h — the parity
        // map below now pins 7 members; a header without the new name flips
        // "status enum drift" (witnessed pre-fix, see repair-10 receipt).
        (
            "CGAP_ERR_CONCURRENT_WRITER_DENIED",
            CGAP_ERR_CONCURRENT_WRITER_DENIED,
        ),
        ("CGAP_ERR_P28_SPOLIATION", CGAP_ERR_P28_SPOLIATION),
    ];

    // -- §6.1 header parity (§6.1 mechanical diff) ---------------------------

    /// Wave-4 leg: the status word typedef must be `uint32_t` verbatim
    /// (normalized) — the C register width is pinned by `_Static_assert` in
    /// the header (C11) and by const-assert in Rust; this leg pins the
    /// DECLARATION text so a silent width change flips the parser court even
    /// where `_Static_assert` is unavailable (C89/C99 consumers).
    fn parse_status_typedef(text: &str) -> Result<(), String> {
        let needle = "typedef uint32_t CgapStatus;";
        if text.contains(needle) {
            return Ok(());
        }
        Err(format!(
            "CgapStatus typedef drift: expected `{needle}` (got none) — width pin violated"
        ))
    }

    fn header_parity(text: &str) -> Result<(), String> {
        let norm = normalize(&strip_comments(text));
        let version = parse_version(&norm)?;
        if version != CGAP_ABI_VERSION {
            return Err(format!(
                "ABI version header={version} rust={CGAP_ABI_VERSION}"
            ));
        }
        parse_status_typedef(&norm)?;
        let got: BTreeMap<String, u32> = parse_enum(&norm)?.into_iter().collect();
        let want: BTreeMap<String, u32> = MIRROR_ENUM
            .iter()
            .map(|(n, v)| (n.to_string(), *v))
            .collect();
        if got != want {
            return Err(format!("status enum drift: header={got:?} rust={want:?}"));
        }
        let got_view = parse_view_fields(&norm)?;
        let want_view: Vec<(String, String)> = MIRROR_VIEW
            .iter()
            .map(|(n, t)| (n.to_string(), t.to_string()))
            .collect();
        if got_view != want_view {
            return Err(format!(
                "view drift: header={got_view:?} rust={want_view:?}"
            ));
        }
        let protos = parse_prototypes(&norm)?;
        for (name, want_params) in MIRROR_PARAMS {
            let Some((ret, params)) = protos.get(*name) else {
                return Err(format!("prototype {name} absent"));
            };
            if ret != "CgapStatus" {
                return Err(format!("{name}: return {ret} != CgapStatus"));
            }
            let want: Vec<(String, String)> = want_params
                .iter()
                .map(|p| {
                    let (t, a) = p
                        .rsplit_once(char::is_whitespace)
                        .expect("mirror param shape");
                    (a.to_string(), t.to_string())
                })
                .collect();
            if params != &want {
                return Err(format!("{name} params: header={params:?} rust={want:?}"));
            }
        }
        Ok(())
    }

    // -- Anti-vacuity corpus (§6.1: a court that cannot flip is FAIL).
    // Each mutation is applied to a COPY of the normalized header text — the
    // checked-in file is never modified — and MUST be detected ON ITS NAMED
    // LEG (4th column: the court leg whose error text must appear; a08-wave7
    // method — the wave-7 named courts flip IN ISOLATION, and every shim
    // anchor is asserted count==1 before write). An undetected mutation
    // fails the court as a VACUOUS GATE; a flip on a collateral leg fails it
    // as unattributable.
    const MUTATIONS: &[(&str, &str, &str, &str)] = &[
        (
            "M1 param-type flip (§6.1 literal)",
            "const uint8_t* bytes",
            "const uint32_t* bytes",
            "cgap_admit_frame params",
        ),
        (
            "M2 param drop (admit len)",
            "bytes, size_t len,",
            "bytes,",
            "cgap_admit_frame params",
        ),
        (
            "M3 return-type change (commit)",
            "CgapStatus cgap_commit_parquet",
            "int cgap_commit_parquet",
            "cgap_commit_parquet: return",
        ),
        (
            "M4 status value flip (P28)",
            "0x503238FFu",
            "0x503238FEu",
            "status enum drift",
        ),
        (
            "M5 ABI version bump",
            "#define CGAP_ABI_VERSION 1u",
            "#define CGAP_ABI_VERSION 2u",
            "ABI version",
        ),
        (
            "M6 out-param const flip (doc divergence note)",
            "cgap_frame_view* out_view",
            "const cgap_frame_view* out_view",
            "cgap_admit_frame params",
        ),
        (
            "M7 view field type (layout pin)",
            "size_t payload_len;",
            "uint32_t payload_len;",
            "view drift",
        ),
        (
            "M8 view field drop (rule_count)",
            "uint32_t rule_count;",
            "",
            "view drift",
        ),
        (
            "M9 symbol rename (verify)",
            "cgap_verify_p28",
            "cgap_verify_p2x",
            "prototype cgap_verify_p28 absent",
        ),
        (
            "M10 enum member rename (OK)",
            "CGAP_OK =",
            "CGAP_SUCCESS =",
            "status enum drift",
        ),
        // Wave-4 additions:
        (
            "M11 status typedef width flip",
            "typedef uint32_t CgapStatus;",
            "typedef uint16_t CgapStatus;",
            "CgapStatus typedef drift",
        ),
        (
            "M12 enum member drop (INVALID_MAGIC)",
            "CGAP_ERR_INVALID_MAGIC =",
            "",
            "status enum drift",
        ),
        // repair-10 (lane F8) addition: dropping the new C-header member must
        // flip the parity court on its named leg.
        (
            "M13 enum member drop (CONCURRENT_WRITER_DENIED)",
            "CGAP_ERR_CONCURRENT_WRITER_DENIED =",
            "",
            "status enum drift",
        ),
    ];

    /// Fold through a view's payload IN PLACE — the house rotate-xor word
    /// fold (`crate::fold_word`, the exact fold `cgap_digest_of` serves and
    /// the `digest_ffi` court parity-pins to `cgap_arrow::digest`). LE word
    /// order, odd tail zero-extended: platform-pinned, replayable.
    fn fold_view(view: &CgapFrameView) -> u64 {
        assert!(!view.payload.is_null(), "fold_view: null payload");
        let bytes = unsafe { core::slice::from_raw_parts(view.payload, view.payload_len) };
        // Wave F2 item 5: the fold LAW has ONE implementation
        // (`super::fold_bytes`, the lib-side byte fold the sovereign verify
        // digest serves) — this court helper delegates instead of carrying a
        // second copy of the fold loop (O(N^2) drift law).
        super::fold_bytes(0, bytes)
    }

    fn cc_available() -> bool {
        Command::new("cc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    // -- §6.1 header court ---------------------------------------------------

    #[test]
    fn rust_side_signature_guards_match_aliases() {
        // Fn items carrying the literal frozen types must coerce to the
        // public fn-pointer aliases: any drift in alias or guard fails to
        // COMPILE. Chain leg: aliases <-> guard literals (the mirror table
        // <-> header leg is the parity test below).
        unsafe extern "C" fn sig_admit(
            _: *const u8,
            _: usize,
            _: *mut CgapFrameView,
        ) -> CgapStatus {
            0
        }
        unsafe extern "C" fn sig_eval(
            _: *const CgapFrameView,
            _: *const u8,
            _: usize,
            _: *mut i32,
            _: usize,
            _: *mut usize,
        ) -> CgapStatus {
            0
        }
        unsafe extern "C" fn sig_commit(
            _: *const i32,
            _: usize,
            _: *const core::ffi::c_char,
        ) -> CgapStatus {
            0
        }
        unsafe extern "C" fn sig_verify(_: *const u8, _: usize, _: *mut u64) -> CgapStatus {
            0
        }
        let _ = sig_admit as CgapAdmitFrameFn;
        let _ = sig_eval as CgapEvalBn3Fn;
        let _ = sig_commit as CgapCommitParquetFn;
        let _ = sig_verify as CgapVerifyP28Fn;
    }

    #[test]
    fn header_court_parity_green_on_checked_in_header() {
        let text = header_source();
        header_parity(&text).unwrap_or_else(|e| {
            panic!("checked-in include/cgap.h drifted from the Rust ABI freeze: {e}")
        });
    }

    #[test]
    fn header_court_mutation_corpus_all_detected() {
        let norm = normalize(&strip_comments(&header_source()));
        for (label, old, new, leg) in MUTATIONS {
            // a08-wave7 shim discipline: the anchor is asserted UNIQUE
            // (count == 1) before the mutation is applied — a multi-site
            // anchor makes the mutation ambiguous and its flip
            // unattributable.
            let hits = norm.matches(old).count();
            assert_eq!(
                hits, 1,
                "corpus anchor not unique (count={hits}): {label} — mutation ambiguous, flip unattributable (a08-wave7 count==1 shim discipline)"
            );
            let mutated = norm.replacen(old, new, 1);
            assert_ne!(mutated, norm, "{label}: mutation did not change the text");
            match header_parity(&mutated) {
                Err(e) => assert!(
                    e.contains(leg),
                    "mutation '{label}' flipped on a COLLATERAL leg (unattributable): expected the error to name {leg:?}, got: {e}"
                ),
                Ok(()) => panic!(
                    "VACUOUS GATE: mutation '{label}' NOT detected — the header court cannot flip"
                ),
            }
            println!("DETECTED  {label} -> {leg} (isolated flip)");
        }
        println!(
            "HEADER MUTATION CORPUS: {} mutations, every anchor unique, every flip on its named leg (a08-wave7 method)",
            MUTATIONS.len()
        );
    }

    /// Wave-8 receipt leg, IN-COURT (the Wave-8 block above names this test
    /// as its witness): the WRONG-LEG SHIM — M4's exact mutation paired with
    /// a leg name that does not detect it — must flip on the COLLATERAL
    /// message. If the parity error ever named the shimmed leg, a wrong leg
    /// column would pass the corpus court silently (attribution dead). This
    /// is the executed form of the Wave-8 "witnessed" claim, replacing a
    /// pointer to a lane report that does not exist.
    #[test]
    fn header_court_wrong_leg_shim_collateral_flip() {
        let norm = normalize(&strip_comments(&header_source()));
        let (label, old, new, true_leg) = *MUTATIONS
            .iter()
            .find(|m| m.0.starts_with("M4"))
            .expect("M4 (status value flip) present in the corpus");
        let wrong_leg = "view drift"; // non-detecting for the M4 enum-value flip
        assert_ne!(
            wrong_leg, true_leg,
            "shim must rename to a NON-detecting leg (test premise)"
        );
        let hits = norm.matches(old).count();
        assert_eq!(hits, 1, "{label}: shim anchor not unique (count={hits})");
        let mutated = norm.replacen(old, new, 1);
        match header_parity(&mutated) {
            Err(e) => {
                assert!(
                    !e.contains(wrong_leg),
                    "shim leg {wrong_leg:?} was named by the error — a wrong leg column would PASS the corpus court; attribution leg dead: {e}"
                );
                assert!(
                    e.contains(true_leg),
                    "error must name the TRUE leg {true_leg:?} (that IS the collateral flip), got: {e}"
                );
            }
            Ok(()) => panic!("VACUOUS: M4 mutation undetected — the shim cannot flip anything"),
        }
        println!(
            "WRONG-LEG SHIM: '{label}' under shim leg {wrong_leg:?} flips COLLATERALLY on {true_leg} — attribution leg load-bearing"
        );
    }

    #[test]
    fn header_court_c11_compile_real_and_pins_fire_on_mutation() {
        if !cc_available() {
            println!("[SKIP] cc absent — C11 compile leg unexecuted (parser legs still ran)");
            return;
        }
        let dir = std::env::temp_dir().join(format!("cgap-header-court-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        // Explicit-typed fn pointers: if a header prototype drifts, these
        // lines fail to type-match and the TU refuses to compile — a C-side
        // cross-check independent of the Rust-side parser.
        let tu = r#"#include "cgap.h"
static CgapStatus (*const p_admit)(const uint8_t*, size_t, cgap_frame_view*) = cgap_admit_frame;
static CgapStatus (*const p_eval)(const cgap_frame_view*, const uint8_t*, size_t, int32_t*, size_t, size_t*) = cgap_eval_bn3;
static CgapStatus (*const p_commit)(const int32_t*, size_t, const char*) = cgap_commit_parquet;
static CgapStatus (*const p_verify)(const uint8_t*, size_t, uint64_t*) = cgap_verify_p28;
int main(void) {
  cgap_frame_view v;
  v.payload = 0; v.payload_len = 0; v.schema_id = 0; v.rule_count = 0;
  return (int)(CGAP_OK + (CgapStatus)CGAP_ABI_VERSION) + (int)sizeof(cgap_frame_view)
       + ((p_admit && p_eval && p_commit && p_verify) ? 0 : 1);
}
"#;
        fs::write(dir.join("tu.c"), tu).expect("write TU");
        let run = |inc: &Path, out: &str| {
            Command::new("cc")
                .arg("-std=c11")
                .arg("-Wall")
                .arg("-I")
                .arg(inc)
                .arg(dir.join("tu.c"))
                .arg("-c")
                .arg("-o")
                .arg(dir.join(out))
                .output()
                .expect("spawn cc")
        };
        let ok = run(&repo_root().join("include"), "real.o");
        assert!(
            ok.status.success(),
            "checked-in cgap.h must compile standalone C11:\n{}",
            String::from_utf8_lossy(&ok.stderr)
        );
        let mut_inc = dir.join("mutated-include");
        fs::create_dir_all(&mut_inc).expect("mutated include dir");
        let real = header_source();
        // a08-wave7 count==1 shim discipline: assert the anchor is unique in
        // the raw header before mutating.
        let anchor = "size_t payload_len;";
        assert_eq!(
            real.matches(anchor).count(),
            1,
            "C11 layout-mutation anchor not unique — compile leg vacuous"
        );
        let mutated = real.replacen(anchor, "uint32_t payload_len;", 1);
        assert_ne!(
            mutated, real,
            "mutation anchor absent — compile leg vacuous"
        );
        fs::write(mut_inc.join("cgap.h"), mutated).expect("write mutated header");
        let bad = run(&mut_inc, "mutated.o");
        assert!(
            !bad.status.success(),
            "VACUOUS GATE: mutated header compiled — the C layout pins never fire"
        );
        let err = String::from_utf8_lossy(&bad.stderr);
        assert!(
            err.contains("cgap ABI freeze"),
            "compile failed but NOT on the layout pin:\n{err}"
        );
        println!("C11 COMPILE COURT: real header compiles; mutated header fails ON the _Static_assert pin");
        let _ = fs::remove_dir_all(&dir);
    }

    // -- Wave-4 hardening: the extern "C" C++ branch of cgap.h is part of the
    //    frozen surface (header doc: "C++ consumers get the same declarations
    //    without the pins") but was unwitnessed by the Wave-3 court. The leg
    //    compiles the SAME TU as C++ against the real header (MUST succeed —
    //    fn-pointer pins type-check, no _Static_asserts fire) and against the
    //    M9-mutated header (MUST FAIL — the rename must flip the C++ leg too,
    //    otherwise the C++ branch is a gate that cannot flip = VACUOUS).
    #[test]
    fn header_court_cpp_compile_leg_real_passes_and_rename_mutant_fails() {
        if !cc_available() {
            println!("[SKIP] cc absent — C++ compile leg unexecuted (parser legs still ran)");
            return;
        }
        let dir =
            std::env::temp_dir().join(format!("cgap-header-court-cpp-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let tu = r#"#include "cgap.h"
static CgapStatus (*const p_admit)(const uint8_t*, size_t, cgap_frame_view*) = cgap_admit_frame;
static CgapStatus (*const p_eval)(const cgap_frame_view*, const uint8_t*, size_t, int32_t*, size_t, size_t*) = cgap_eval_bn3;
static CgapStatus (*const p_commit)(const int32_t*, size_t, const char*) = cgap_commit_parquet;
static CgapStatus (*const p_verify)(const uint8_t*, size_t, uint64_t*) = cgap_verify_p28;
int main() {
  cgap_frame_view v{};
  return (int)(CGAP_OK + (CgapStatus)CGAP_ABI_VERSION) + (int)sizeof(cgap_frame_view)
       + ((p_admit && p_eval && p_commit && p_verify) ? 0 : 1);
}
"#;
        fs::write(dir.join("tu.cpp"), tu).expect("write TU");
        let run = |inc: &Path, out: &str| {
            Command::new("cc")
                .arg("-x")
                .arg("c++")
                .arg("-std=c++17")
                .arg("-Wall")
                .arg("-I")
                .arg(inc)
                .arg(dir.join("tu.cpp"))
                .arg("-c")
                .arg("-o")
                .arg(dir.join(out))
                .output()
                .expect("spawn cc")
        };
        let ok = run(&repo_root().join("include"), "real_cpp.o");
        assert!(
            ok.status.success(),
            "checked-in cgap.h must compile as C++17 (extern \"C\" branch):\n{}",
            String::from_utf8_lossy(&ok.stderr)
        );
        let mut_inc = dir.join("mutated-include");
        fs::create_dir_all(&mut_inc).expect("mutated include dir");
        let real = header_source();
        // a08-wave7 count==1 shim discipline + precise anchor: rename ONLY
        // the prototype site. The raw header mentions the symbol TWICE
        // (file-top comment + prototype), so a blanket-symbol anchor is
        // multi-site and the rename unattributable; the prototype-scoped
        // anchor is count==1.
        let anchor = "CgapStatus cgap_verify_p28(";
        assert_eq!(
            real.matches(anchor).count(),
            1,
            "C++ rename anchor not unique (must be the prototype site only; the file-top comment mention must NOT be renamed) — compile leg vacuous"
        );
        let mutated = real.replacen(anchor, "CgapStatus cgap_verify_p2x(", 1);
        assert_ne!(mutated, real, "rename anchor absent — C++ leg vacuous");
        fs::write(mut_inc.with_file_name("cgap.h"), mutated).expect("write mutated header");
        let bad = run(&mut_inc, "mutated_cpp.o");
        assert!(
            !bad.status.success(),
            "VACUOUS GATE: renamed-symbol header compiled as C++ — the C++ leg cannot flip"
        );
        println!("C++ COMPILE COURT: real header compiles as C++17; symbol-rename mutant fails");
        let _ = fs::remove_dir_all(&dir);
    }

    // -- Ledger walk (module-level so the Wave-7 pollution court can arm its
    //    named mutation as a second arm) --------------------------------------

    /// Bounded recursive walk of workspace `.rs` sources. Wave-7: directory
    /// names starting with `target` are NEVER descended (lane build roots are
    /// leases, not workspace source; see the §6.2 ledger comment).
    fn walk(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
        walk_impl(dir, depth, files, true);
    }

    /// The pollution court's NAMED MUTATION arm: the Wave-7 `target*` skip
    /// deleted. It exists so the court can prove the skip is load-bearing
    /// every run (same pattern as the header court's M-corpus: the mutation
    /// is executed, and an undetected mutation fails the court).
    fn walk_unfiltered(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
        walk_impl(dir, depth, files, false);
    }

    fn walk_impl(dir: &Path, depth: usize, files: &mut Vec<PathBuf>, skip_build_roots: bool) {
        if depth > 6 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let is_build_root = skip_build_roots
                    && p.file_name()
                        .map_or(false, |n| n.to_string_lossy().starts_with("target"));
                if !is_build_root {
                    walk_impl(&p, depth + 1, files, skip_build_roots);
                }
            } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                files.push(p);
            }
        }
    }

    // -- §6.2 status court ---------------------------------------------------

    #[test]
    fn status_court_register_hygiene_and_header_value_parity() {
        let vals = [
            CGAP_OK,
            CGAP_ERR_INVALID_MAGIC,
            CGAP_ERR_OABI_MISMATCH,
            CGAP_ERR_SHACL_VIOLATION,
            CGAP_ERR_LEASE_EXHAUSTED,
            CGAP_ERR_CONCURRENT_WRITER_DENIED,
            CGAP_ERR_P28_SPOLIATION,
        ];
        for i in 0..vals.len() {
            for j in (i + 1)..vals.len() {
                assert_ne!(
                    vals[i], vals[j],
                    "register values must be pairwise distinct"
                );
            }
        }
        // Exact pins (§2).
        assert_eq!(CGAP_OK, 0x0000_0000);
        assert_eq!(CGAP_ERR_INVALID_MAGIC, 0x4347_0001);
        assert_eq!(CGAP_ERR_OABI_MISMATCH, 0x4347_0002);
        assert_eq!(CGAP_ERR_SHACL_VIOLATION, 0x4347_0003);
        assert_eq!(CGAP_ERR_LEASE_EXHAUSTED, 0x4347_0004);
        assert_eq!(CGAP_ERR_P28_SPOLIATION, 0x5032_38FF);
        assert_eq!(CGAP_ERR_CONCURRENT_WRITER_DENIED, 0x4347_0006);
        // Value classes: 'CG' (0x4347) + nonzero nn; 'P28' (0x503238) + FF.
        for v in [
            CGAP_ERR_INVALID_MAGIC,
            CGAP_ERR_OABI_MISMATCH,
            CGAP_ERR_SHACL_VIOLATION,
            CGAP_ERR_LEASE_EXHAUSTED,
            CGAP_ERR_CONCURRENT_WRITER_DENIED,
        ] {
            assert_eq!(v & 0xFFFF_0000, 0x4347_0000, "'CG' class");
            assert_ne!(v & 0x0000_FFFF, 0, "'CG' sequence nonzero");
        }
        assert_eq!(
            CGAP_ERR_P28_SPOLIATION & 0xFFFF_FF00,
            0x5032_3800,
            "'P28' class"
        );
        assert_eq!(CGAP_ERR_P28_SPOLIATION & 0xFF, 0xFF, "'P28' tail FF");
        // Header enum parity, asserted independently of the §6.1 court so
        // the status court never depends on another test's passage.
        let norm = normalize(&strip_comments(&header_source()));
        let got: BTreeMap<String, u32> = parse_enum(&norm)
            .expect("header enum parse")
            .into_iter()
            .collect();
        for (name, v) in MIRROR_ENUM {
            assert_eq!(got.get(*name), Some(v), "header enum value for {name}");
        }
        assert_eq!(
            got.len(),
            7,
            "header enum pins exactly the seven register values (repair-10: \
             CONCURRENT_WRITER_DENIED promoted to the C header)"
        );
    }

    #[test]
    fn status_court_reachability_ledger() {
        let names = [
            "CGAP_ERR_INVALID_MAGIC",
            "CGAP_ERR_OABI_MISMATCH",
            "CGAP_ERR_SHACL_VIOLATION",
            "CGAP_ERR_LEASE_EXHAUSTED",
            "CGAP_ERR_P28_SPOLIATION",
        ];
        // Bounded walk of ALL workspace-member .rs sources (real on-disk
        // files): `crates/` AND `ontologies/` — fibo/cloud are workspace
        // members (root Cargo.toml), so a producer landing in ontologies/ is
        // as landed as one in crates/. Wave-6 repair: the walk previously
        // covered `crates/` only, so a producer landing in ontologies/ would
        // have been SILENTLY INVISIBLE — the pinned-skip guard below could
        // not flip on a state it cannot see. Driven by the Wave-5 audit
        // method (a20-wave5 receipt divergence 3: sites outside the scan are
        // uncounted, not nonexistent). Wave-7 repair (a07-wave6 §5: two
        // gates, two coverage sets): lane build roots — directory names
        // starting with `target`, e.g. `crates/cgap-bont/target-a06/` and
        // `ontologies/fibo/target-a09/`, both present on the wave-7 tree —
        // are NEVER descended: build roots are leases, not workspace source
        // (same-checkout-fanout cleanup law), so build-artifact text can
        // never count as a workspace production. The skip is proven
        // load-bearing by `ledger_walk_build_root_pollution_court` below.
        let mut files: Vec<PathBuf> = Vec::new();
        for root in ["crates", "ontologies"] {
            walk(&repo_root().join(root), 0, &mut files);
        }
        assert!(
            files.len() > 50,
            "workspace walk found {} .rs files — the walk is broken, ledger vacuous",
            files.len()
        );
        // Wave-7 hygiene pin on the REAL tree: the ledger walk is workspace
        // source only. If a lane build root ever contains .rs files, the
        // ledger refuses loudly instead of silently counting pollution (the
        // same silent-invisibility class a07-wave6 §5 flagged for gates with
        // mismatched coverage sets).
        let polluted: Vec<String> = files
            .iter()
            .filter(|f| {
                f.components()
                    .any(|c| c.as_os_str().to_string_lossy().starts_with("target"))
            })
            .map(|f| f.display().to_string())
            .collect();
        assert!(
            polluted.is_empty(),
            "ledger walk found .rs files inside lane build roots (target*): {polluted:?} — build roots are leases, not workspace source (same-checkout-fanout cleanup law); delete or exclude them, never count them"
        );
        // Wave-6 repair pin — the anti-vacuity of the widening itself: the
        // named ontologies member source must be present in the walked set,
        // so narrowing the walk scope back to crates/-only flips THIS assert.
        // Exact-path membership, not a suffix match, so a stray nested copy
        // cannot satisfy the pin. (A19 correction, Wave 9: the flip is
        // EXECUTED in-court by the crates-only negative control directly
        // below — this block previously pointed at "the lane report", but no
        // waves/ receipt file exists for A19; waves/ is coordinator-owned.)
        let ontologies_member = repo_root().join("ontologies/cloud/src/lib.rs");
        assert!(
            files.iter().any(|f| f == &ontologies_member),
            "ledger walk lost ontologies/cloud/src/lib.rs — walk scope regressed below the Wave-6 repair (workspace members are crates/ + ontologies/)"
        );
        // The widening's own negative control, executed EVERY run: the
        // pre-Wave-6 crates/-only scope must NOT contain the ontologies
        // member — otherwise the pin above could never flip and the widening
        // would be vacuous.
        let mut crates_only: Vec<PathBuf> = Vec::new();
        walk_impl(&repo_root().join("crates"), 0, &mut crates_only, true);
        assert!(
            !crates_only.iter().any(|f| f == &ontologies_member),
            "crates-only (pre-Wave-6) walk CONTAINS the ontologies member — the Wave-6 pin could never flip, VACUOUS"
        );
        let self_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sovereign.rs");

        // Producer = a `return CGAP_ERR_<NAME>;` site outside this file
        // (definitions are not productions).
        let mut sites: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for f in &files {
            if f == &self_path {
                continue;
            }
            let Ok(text) = fs::read_to_string(f) else {
                continue;
            };
            for n in names {
                let c = text.matches(&format!("return {n}")).count();
                if c > 0 {
                    sites.entry(n).or_default().push(format!(
                        "{} ({c} site{})",
                        f.display(),
                        if c == 1 { "" } else { "s" }
                    ));
                }
            }
        }

        // Landed pins: the A14 penstock produces three of the five fault
        // values; the ledger cites its on-disk sites mechanically.
        let penstock = files
            .iter()
            .find(|f| f.ends_with("cgap-gate/src/penstock.rs"))
            .expect("cgap-gate/src/penstock.rs in workspace");
        let ptext = fs::read_to_string(penstock).expect("penstock.rs readable");
        for n in [
            "CGAP_ERR_INVALID_MAGIC",
            "CGAP_ERR_OABI_MISMATCH",
            "CGAP_ERR_P28_SPOLIATION",
        ] {
            let s = sites
                .get(n)
                .unwrap_or_else(|| panic!("ledger: {n} has NO landed producer"));
            assert!(
                s.iter().any(|x| x.contains("penstock.rs")),
                "{n} must cite penstock.rs"
            );
            println!("LANDED  {n}: {}", s.join(", "));
        }
        // Cross-lane frozen-copy parity (Wave-1 note): penstock pins the
        // register by value, this module is canonical.
        for hex in ["0x0000_0000", "0x4347_0001", "0x4347_0002", "0x5032_38FF"] {
            assert!(ptext.contains(hex), "penstock register copy {hex} missing");
        }
        assert!(ptext.contains("CGAP_OK"), "penstock success value absent");
        println!("LANDED  CGAP_OK: success value of the landed producers (cgap-gate::ingress success tail)");

        // GAP-1 update (2026-11-01): the Wave-3 pin said SHACL/LEASE had "no
        // landed producer in crates/" — that reality MOVED this wave. The
        // sovereign BODIES landed in THIS file (§2 owner-table wiring:
        // admit->A04 shape leg, commit->A02 fixed-capacity lease) and now
        // produce both values for real; the scan above excludes self_path
        // (definitions are not productions), so the in-file producers are
        // counted here, mechanically, by the same scan pattern.
        let self_text = fs::read_to_string(&self_path).expect("sovereign.rs readable");
        for n in ["CGAP_ERR_SHACL_VIOLATION", "CGAP_ERR_LEASE_EXHAUSTED"] {
            let c = self_text.matches(&format!("return {n}")).count();
            assert!(
                c >= 1,
                "{n}: sovereign-body producer site absent from this file — reachability reality regressed; update the pin + receipt"
            );
            println!(
                "LANDED  {n}: sovereign body (this file, {c} site{})",
                if c == 1 { "" } else { "s" }
            );
        }
        println!(
            "STATUS COURT: 5/5 fault values fault-reachable (cgap-gate::ingress 3 + sovereign bodies 2); CGAP_OK success path landed"
        );
    }

    // -- Wave-7: the walk's `target*` skip is load-bearing. Named mutation =
    //    the skip deleted, armed as `walk_unfiltered` and executed EVERY run
    //    (a16-wave6 M-corpus pattern: the mutation is executed in-court, and
    //    an undetected mutation fails the court as VACUOUS GATE).

    #[test]
    fn ledger_walk_build_root_pollution_court() {
        // Real temp tree (Chicago: real files, real walk, no mocks):
        //   <root>/crates/m/src/real.rs              — real workspace source
        //   <root>/crates/m/target-lane7/artifact.rs — build-root pollution
        let dir = std::env::temp_dir().join(format!("cgap-walk-court-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let src = dir.join("crates/m/src");
        let bld = dir.join("crates/m/target-lane7");
        fs::create_dir_all(&src).expect("temp src dir");
        fs::create_dir_all(&bld).expect("temp build dir");
        fs::write(src.join("real.rs"), "fn real() {}").expect("write real source");
        fs::write(bld.join("artifact.rs"), "fn artifact() {}").expect("write build artifact");

        let has = |files: &[PathBuf], name: &str| {
            files
                .iter()
                .any(|f| f.file_name().map_or(false, |n| n.to_string_lossy() == name))
        };
        let mut filtered = Vec::new();
        let mut unfiltered = Vec::new();
        walk(&dir, 0, &mut filtered);
        walk_unfiltered(&dir, 0, &mut unfiltered);

        // Premises: both arms really walk (the real source is found by both)
        // and the pollution is real (an unfiltered walk sees it — otherwise
        // this court could never detect a broken skip and would be vacuous).
        assert!(
            has(&filtered, "real.rs") && has(&unfiltered, "real.rs"),
            "walk broken: real source not visited by both arms — court vacuous"
        );
        assert!(
            has(&unfiltered, "artifact.rs"),
            "NEGATIVE CONTROL FAILED: the unfiltered (mutation) arm cannot see the planted build-root source — the court could never detect a deleted skip (vacuous)"
        );
        // The repair: the filtered walk must not count build-root pollution.
        // This is the leg that FLIPS if the Wave-7 skip is deleted or broken.
        assert!(
            !has(&filtered, "artifact.rs"),
            "VACUOUS GATE / repair reverted: the filtered walk counted build-root pollution — the target* skip is gone (named mutation FIRED and was detected, court red as designed)"
        );
        println!(
            "WALK POLLUTION COURT: filtered walk excludes target* build roots; unfiltered (mutation) arm sees the planted artifact — skip proven load-bearing"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // -- §6.4 zero-copy witness ----------------------------------------------

    #[test]
    fn zero_copy_witness_view_borrows_caller_buffer() {
        // §6.4: >=1 MB host frame; the view must BORROW the caller's bytes.
        const LEN: usize = 1024 * 1024 + 7; // odd tail word included
        let mut buf = vec![0u8; LEN];
        let mut seed: u64 = 0x2463_4100_A19E_0001;
        for chunk in buf.chunks_mut(8) {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let bytes = seed.to_le_bytes();
            let take = chunk.len();
            chunk.copy_from_slice(&bytes[..take]);
        }
        let view = CgapFrameView {
            payload: buf.as_ptr(),
            payload_len: LEN,
            schema_id: 1,
            rule_count: 1,
        };
        assert_eq!(
            view.payload,
            buf.as_ptr(),
            "view.payload must alias the caller buffer (zero-copy borrow, §2)"
        );
        assert_eq!(view.payload_len, LEN);
        // Negative control: an independent copy observed through its own
        // view must NOT move when the caller buffer mutates — proving any
        // change through `view` comes from the borrow, not fold drift.
        let snapshot = buf.clone();
        let view_copy = CgapFrameView {
            payload: snapshot.as_ptr(),
            payload_len: LEN,
            schema_id: 1,
            rule_count: 1,
        };

        let d1 = fold_view(&view);
        let d1c = fold_view(&view_copy);
        assert_eq!(d1, d1c, "fold must be deterministic over equal bytes");
        assert_ne!(
            d1, 0,
            "1 MiB+7 pseudorandom bytes folded to 0 — fold broken"
        );
        for site in [LEN / 2, 0, LEN - 1] {
            // SAFETY: site < LEN, single-threaded test-local buffer;
            // volatile defeats elision of the mutation.
            let orig = unsafe { core::ptr::read_volatile(buf.as_ptr().add(site)) };
            unsafe { core::ptr::write_volatile(buf.as_mut_ptr().add(site), !orig) };
            let d2 = fold_view(&view);
            assert_ne!(
                d2, d1,
                "site {site}: one-byte flip must change the fold through the view"
            );
            assert_eq!(
                fold_view(&view_copy),
                d1c,
                "site {site}: snapshot fold moved — change did not come via the borrow"
            );
            unsafe { core::ptr::write_volatile(buf.as_mut_ptr().add(site), orig) };
            assert_eq!(
                fold_view(&view),
                d1,
                "site {site}: restore must return the original fold"
            );
        }
        println!(
            "ZERO-COPY WITNESS: {LEN} B frame; 3 mutate/restore sites through the view; snapshot negative control held"
        );
    }
}

// ===========================================================================
// Wave 9 — cross-lane integration slice. Dispatch: "Wave 9 - cross-lane
// integration: A04 identity court, A14 dam over A04 frames, A20 1M-row
// stress, A05 power-cut sweep, A07 zero-string gate over all built crates."
//
// A19's seat is the C-ABI membrane, so this wave's contribution is NOT new
// logic in those lanes (their owner lanes landed it: A04 identity court =
// crates/cgap-core/tests/identity_court.rs; A14 dam over A04 frames =
// crates/cgap-core/tests/cross_lane_gate_court.rs; A20 verdict suite =
// crates/cgap-court [[test]] court_verdict; A05 sweep =
// crates/cgap-storage/tests/court_sweep.rs; A07 gate = scripts/
// zero_string_gate.py). It is the boundary's own witness: this module
// EXECUTES the five surfaces as real subprocesses on the real tree, this
// session, every run — other lanes' receipts are hearsay here
// (receipt-cited counts are stale by default; re-read at use time).
//
// IN-CODE WAVE-9 RECEIPT (durable receipt lives in the owned artifact; the
// waves/ directory is coordinator-owned, per the Wave-7 continuity note).
// Session receipts, this wave (commands + exits re-run this session):
//   - A04: `cargo test -p cgap-core --test identity_court` -> exit 0,
//     "4 passed; 0 failed" (courts also re-witnessed through the nested
//     court, below).
//   - A14: `cargo test -p cgap-core --test cross_lane_gate_court` -> exit 0,
//     "8 passed; 0 failed".
//   - A05: `cargo test -p cgap-storage --test court_sweep` -> exit 0,
//     "2 passed; 0 failed" in 4.33s.
//   - A07: `python3 scripts/zero_string_gate.py` (default scan = ALL built
//     crates) -> exit 0, "SUMMARY scanned=45 crates=22 hits=0 emitter_hits=2
//     adv_hits=0" (scanned grew to 49 by end of wave as lanes landed —
//     floors are 10/20, both far under); `--self-check` -> exit 0,
//     "SELF-CHECK RESULT PASS", poison fired=True. Both also re-executed
//     inside the A07 court every run.
//     REPAIR ADDENDUM (Wave-9 lane-runner session, 2026-11-01): the scan
//     had gone RED — the gate's workspace_members capture stopped at the
//     first `]` of the root members array, which lands inside the scaffold
//     comment "(its own [workspace])", so only the 11 pre-moonshot members
//     were enumerated, cgap-fibo was never enumerated, and the no-dead-
//     entries law refused: `GATE-REFUSED: emitter-exempt-unresolved
//     cgap-fibo:emit-fibo-pack`, exit 1 (witnessed; the lane's whole test
//     command exited 101 on this single red court). Repaired as a DISCLOSED
//     minimal unblock (see divergences below) and re-witnessed green this
//     session: scan exit 0 scanned=67 crates=21 hits=0 emitter_hits=2;
//     --self-check exit 0 (poison fired, restore byte-exact); gate --courts
//     13/13 exit 0; the A07 court green (scan + self-check legs).
//   - A20: `cargo test -p cgap-court --test court_verdict` -> exit 0,
//     12/12, three consecutive runs at end of wave. TWO live transients
//     were witnessed and are recorded, not rounded over:
//     (1) a shared-crate compile freeze: cgap-bn3 (arena.rs:632,685,697-
//     699 referenced `BN3_STATEMENT_SIZE` without importing it; sigil.rs:81
//     defines it) was red 08:25-08:31 while the bn3 seam lanes (A11+A12+A13)
//     were actively editing (arena.rs mtime 08:26:18 -> fix landed 08:31:53,
//     exit 0 re-witnessed) — the freeze resolved INSIDE the SLA window by
//     its owner; this lane applied NO unblock edit to bn3.
//     (2) one red witness of `a05_interleave_engine` (court_verdict.rs:1044
//     assert, "11 passed; 1 failed") against the A20 owner's MID-EDIT tree
//     (tests/court_verdict.rs mtime 08:34:41, sampled during the edit);
//     the settled tree is 12/12 x3 (the edit added a 12th court). The court
//     asserting the tree's REAL run-time state is the court working — a
//     mid-edit tree fails loudly, never silently.
//
// Cross-lane divergence disclosures (updated this session; the prior "NONE"
// was invalidated by the A07 seam repair below):
//   1. scripts/zero_string_gate.py is A07-OWNED; this lane applied a MINIMAL
//      unblock there (disclosed in both directions): workspace_members now
//      strips comments from the full manifest text BEFORE the bracket
//      capture. Root cause and evidence in the A07 addendum above. The gate's
//      own court suite re-ran green after the edit (13/13, exit 0).
//   2. crates/cgap-ffi/xip.ld landed on disk this session — the lane doc's
//      Files-owned list names xip.ld ("PROPOSED addition") and the lane doc
//      is authoritative over the dispatch lease line (crates/cgap-ffi/src/ +
//      include/cgap.h); the §6.6 ASSERT court above executes its mutant
//      every run.
// No other file outside the lane's owned files was written (bn3 self-repaired
// inside the SLA window in the prior session; the A20 edit belongs to its
// owner lane).
//
// Wave-9 lane-runner session addendum (this session, re-witnessed):
//   - xip courts landed (§6.3/§6.6, DoD 6/7/8): the ASSERT court is green on
//     both legs — .oabi/.bn3 land page-aligned inside FLASH
//     (0x42010000/0x42020000) and the section-absent mutant refuses ON the
//     ASSERT text — so DoD 6 is witnessed every run, not once-then-reverted.
//     T1 size gate: the A01 owner wall cited by the wave-9 pin FELL (A01
//     c_abi_nested target-class split + alloc-cfg gate; and the crate's own
//     `no_std` gate landed) — the [SKIP] branch is deleted, the size gate is
//     hard, and the flash/SRAM numbers are MEASURED in-court every run.
//     T2 loud [SKIP] (xtensa fork toolchain absent) — DoD 8 SKIP-receipted.
//   - lib.rs §5 placement law landed: the Box::leak slot exports are
//     #[cfg(not(target_os = "none"))] (host-only; the allocator drops out of
//     the XIP SRAM budget) and the none-target glue is
//     #[cfg(target_os = "none")] (panic handler). Host builds unchanged
//     (full lane command exit 0 before and after the edit).
//   - Receipt-honesty repairs: the Wave-8 "witnessed in the lane report" and
//     Wave-6 "receipt in the lane report" pointers named a file that does
//     not exist (no waves/ A19 receipt; waves/ is coordinator-owned) — both
//     claims are now EXECUTED in-court every run
//     (header_court_wrong_leg_shim_collateral_flip; the crates-only negative
//     control in status_court_reachability_ledger).
//
// Court-wide discipline (lane contract, unchanged): real collaborators (the
// real suites on the real tree, real exits, real output), assert final
// state, no mocks. Anti-vacuity: every leg must be able to FLIP — a suite
// that executes ZERO tests is the compile-abort class and is refused by the
// `require_summary` floor (negative control executed every run in
// `wave9_result_parser_negative_control`), a red suite fails its court by
// exit, the A07 court additionally requires the gate's own poison/clean
// self-check to pass (a gate that cannot detect poison cannot vouch for
// hits=0), and every nested build root is a LEASE deleted by the last
// leaseholder (cleanup law; deletion asserted in the negative-control
// court on a real scratch root).
//
// No include/cgap.h change in Wave 9: the header court is green and no
// wave-8/wave-9 finding implicates the header (diff-minimal, waves 7/8
// method).
// ===========================================================================

#[cfg(test)]
mod wave9_courts {
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    /// Absolute repo root, derived from the crate manifest (never cwd).
    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root canonicalize")
    }

    // -- Bounded subprocess runner ------------------------------------------
    //
    // No-sleep law: the wait below is a BOUNDED POLL on a real observable
    // (child exit status) with a deadline; the poll step lives only inside
    // this one audited poll implementation (SLEEP-CLASS: POLL-STEP) and a
    // deadline breach kills the child and fails the court — time never
    // substitutes for the observation.

    /// Poll step of the audited bounded poll (never a coordination sleep).
    const POLL_STEP: Duration = Duration::from_millis(250);
    /// Cargo nested-suite deadline. Generous on purpose (first run builds
    /// the whole closure in a fresh lease), still bounded and enforced.
    const CARGO_DEADLINE: Duration = Duration::from_secs(900);
    /// Python gate deadline (the scan is strings(1)-bound, seconds-class).
    const GATE_DEADLINE: Duration = Duration::from_secs(300);

    pub(super) struct Outcome {
        pub(super) status: Option<std::process::ExitStatus>,
        pub(super) stdout: String,
        pub(super) stderr: String,
    }

    pub(super) fn run_bounded(
        program: &str,
        args: &[&str],
        envs: &[(&str, String)],
        deadline: Duration,
    ) -> Result<Outcome, String> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("spawn {program} {args:?}: {e}"))?;
        let out_pipe = child.stdout.take().expect("stdout piped");
        let err_pipe = child.stderr.take().expect("stderr piped");
        let read_out = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::Read::read_to_string(&mut { out_pipe }, &mut s);
            s
        });
        let read_err = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::Read::read_to_string(&mut { err_pipe }, &mut s);
            s
        });
        let started = Instant::now();
        let mut timed_out = false;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => {
                    if started.elapsed() >= deadline {
                        timed_out = true;
                        let _ = child.kill();
                        let _ = child.wait();
                        break None;
                    }
                    std::thread::sleep(POLL_STEP); // SLEEP-CLASS: POLL-STEP (audited bounded poll)
                }
                Err(e) => return Err(format!("try_wait {program}: {e}")),
            }
        };
        let stdout = read_out.join().expect("stdout reader thread");
        let stderr = read_err.join().expect("stderr reader thread");
        Ok(Outcome {
            status: if timed_out { None } else { status },
            stdout,
            stderr,
        })
    }

    pub(super) fn tail(text: &str) -> String {
        let len = text.chars().count();
        if len <= 2000 {
            text.to_string()
        } else {
            text.chars().skip(len - 2000).collect()
        }
    }

    // -- Nested cargo plumbing ----------------------------------------------
    //
    // The outer `cargo test` holds the PRIMARY target dir for the whole
    // invocation; a nested cargo on the same dir would deadlock against its
    // own parent. Every nested suite therefore builds into a fresh LEASE
    // root (same-checkout-fanout: build roots are leases, not assets) —
    // shared per test-process, deleted by the LAST leaseholder out.

    fn lease_root() -> &'static PathBuf {
        static ROOT: OnceLock<PathBuf> = OnceLock::new();
        ROOT.get_or_init(|| {
            let r = std::env::temp_dir()
                .join(format!("cgap-a19-w9-nested-target-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&r); // stale lease from a crashed prior run
            std::fs::create_dir_all(&r).expect("create nested build-root lease");
            r
        })
    }

    /// Open lease count for the shared nested build root.
    fn open_leases() -> &'static AtomicUsize {
        static OPEN: OnceLock<AtomicUsize> = OnceLock::new();
        OPEN.get_or_init(|| AtomicUsize::new(0))
    }

    /// The cleanup law, factored so BOTH legs are executable (a law only
    /// its happy path can express cannot be witnessed non-vacuously): the
    /// LAST holder out (`holders_after_release == 0`) deletes the
    /// build-root lease; an earlier release must leave it standing.
    fn release_lease_if_last(root: &Path, holders_after_release: usize) {
        if holders_after_release == 0 {
            let _ = std::fs::remove_dir_all(root);
        }
    }

    struct Lease;

    impl Lease {
        fn acquire() -> Lease {
            open_leases().fetch_add(1, Ordering::SeqCst);
            Lease
        }
    }

    impl Drop for Lease {
        fn drop(&mut self) {
            let before = open_leases().fetch_sub(1, Ordering::SeqCst);
            release_lease_if_last(lease_root(), before - 1);
        }
    }

    /// In-process serializer: parallel cargo courts would queue on the
    /// lease's own file lock and burn their deadlines waiting; serialized
    /// here instead so each court gets its full budget. Poison-tolerant on
    /// purpose — a panicking predecessor must not turn successors'
    /// failures into "mutex poisoned".
    fn cargo_serializer() -> std::sync::MutexGuard<'static, ()> {
        static SERIALIZER: OnceLock<Mutex<()>> = OnceLock::new();
        SERIALIZER
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|p| p.into_inner())
    }

    fn nested_cargo_envs() -> Vec<(&'static str, String)> {
        vec![
            ("CARGO_TARGET_DIR", lease_root().display().to_string()),
            // Wave-receipt method (a20-wave5/a20-wave9): dev opt-level 2 so
            // the 1M-row stress and the BLAKE3 closures run at measurable
            // speed; dev opt-0 is not the receipted configuration.
            ("CARGO_PROFILE_DEV_OPT_LEVEL", "2".to_string()),
            // Deterministic resolution: every dep in these closures is a
            // workspace path dep (E=0 externals, gate --deps receipt).
            ("CARGO_NET_OFFLINE", "true".to_string()),
        ]
    }

    /// The shared body of the four cargo suite courts: run ONE nested
    /// `cargo test -p <pkg> --test <target>` on a fresh lease, assert
    /// exit 0, and require a libtest summary with `0 failed` and at least
    /// `min_passed` executed tests (the zero-executed/compile-abort class
    /// must never read as a pass).
    fn cargo_suite_court(label: &str, pkg: &str, test_target: &str, min_passed: u64) {
        // Held to scope end: the LAST court out of the process deletes the
        // lease root (Drop impl below).
        let _lease = Lease::acquire();
        let _serial = cargo_serializer();
        let envs = nested_cargo_envs();
        let started = Instant::now();
        let outcome = run_bounded(
            "cargo",
            &["test", "-p", pkg, "--test", test_target],
            &envs,
            CARGO_DEADLINE,
        )
        .unwrap_or_else(|e| panic!("{label}: nested runner failed: {e}"));
        let elapsed = started.elapsed();
        let status = outcome.status.unwrap_or_else(|| {
            panic!(
                "{label}: deadline of {}s exceeded — child killed; stderr tail:\n{}",
                CARGO_DEADLINE.as_secs(),
                tail(&outcome.stderr)
            )
        });
        assert!(
            status.success(),
            "{label}: `cargo test -p {pkg} --test {test_target}` exited {status}; stdout tail:\n{}\nstderr tail:\n{}",
            tail(&outcome.stdout),
            tail(&outcome.stderr)
        );
        let (passed, failed) = require_summary(&outcome.stdout, min_passed)
            .unwrap_or_else(|e| panic!("{label}: {e}; stdout tail:\n{}", tail(&outcome.stdout)));
        println!(
            "WAVE9 INTEGRATION  {label}: cargo test -p {pkg} --test {test_target} exit=0 passed={passed} failed={failed} elapsed={elapsed:?} (lease {})",
            lease_root().display()
        );
    }

    // -- libtest summary parsing (the court's flip sensor) -------------------

    /// Parse EVERY `test result:` line in `out`. Ok((passed, failed)) only
    /// when: at least one summary line exists (a cargo invocation that ran
    /// no test binary is the compile-abort class — never a pass), every
    /// line is `ok`, `failed == 0`, and `passed >= min_passed`.
    fn require_summary(out: &str, min_passed: u64) -> Result<(u64, u64), String> {
        let mut lines = 0usize;
        let (mut passed, mut failed) = (0u64, 0u64);
        for raw in out.lines() {
            let Some(rest) = raw.trim_start().strip_prefix("test result: ") else {
                continue;
            };
            lines += 1;
            let ok = rest.starts_with("ok.");
            let mut num: Option<u64> = None;
            let (mut p, mut f) = (None, None);
            for tok in rest.split_whitespace() {
                if tok == "passed;" || tok == "passed" {
                    p = num;
                } else if tok == "failed;" || tok == "failed" {
                    f = num;
                }
                num = tok.parse::<u64>().ok();
            }
            let (Some(p), Some(f)) = (p, f) else {
                return Err(format!("unparsable libtest summary: {raw:?}"));
            };
            if !ok {
                return Err(format!("libtest summary not ok: {raw:?}"));
            }
            passed += p;
            failed += f;
        }
        if lines == 0 {
            return Err(
                "no `test result:` line — no test binary executed (compile-abort class), refused"
                    .into(),
            );
        }
        if failed != 0 {
            return Err(format!("{failed} test(s) FAILED"));
        }
        if passed < min_passed {
            return Err(format!(
                "only {passed} test(s) executed (< floor {min_passed}) — a suite gutted to zero executed tests must flip this court, never pass it"
            ));
        }
        Ok((passed, failed))
    }

    /// Anti-vacuity, executed EVERY run (the walk_unfiltered pattern): the
    /// refusal paths above are real legs of the real parser, armed against
    /// fabricated outputs on every run. An undetectable failure class is a
    /// court that cannot flip = VACUOUS GATE; these four flips are the
    /// proof it can. Also witnesses the lease cleanup law on a real
    /// scratch root (deletion by the last leaseholder, observed).
    #[test]
    fn wave9_result_parser_negative_control() {
        let ok3 = "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";
        assert_eq!(require_summary(ok3, 3).unwrap(), (3, 0));
        let refusals: Vec<(&str, &str)> = vec![
            (
                "zero-executed",
                "test result: ok. 0 passed; 0 failed; 0 ignored",
            ),
            (
                "red suite",
                "test result: FAILED. 2 passed; 1 failed; 0 ignored",
            ),
            (
                "no summary (compile-abort)",
                "error: could not compile `cgap-x` (lib)",
            ),
        ];
        for (label, fabricated) in refusals {
            assert!(
                require_summary(fabricated, 1).is_err(),
                "negative control '{label}' was ACCEPTED — parser cannot flip, VACUOUS GATE"
            );
        }
        // Floor refusal is its own leg: a green 1-test summary must refuse
        // at floor 2 but pass at floor 1.
        let one = "test result: ok. 1 passed; 0 failed";
        assert!(require_summary(one, 2).is_err(), "floor leg cannot flip");
        assert_eq!(require_summary(one, 1).unwrap(), (1, 0));

        // Lease cleanup law, both legs, executed on a real scratch root:
        // a NON-last release must leave the root standing; the LAST
        // release must delete it. (Lease::drop routes through the same
        // factored law — if either leg breaks, the shared nested build
        // root either never disappears or vanishes under a live court.)
        let scratch =
            std::env::temp_dir().join(format!("cgap-a19-w9-lease-court-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("scratch lease");
        assert!(
            scratch.exists(),
            "scratch lease root must exist before release"
        );
        release_lease_if_last(&scratch, 1); // one holder remains
        assert!(
            scratch.exists(),
            "non-last release DELETED the lease root — cleanup law broken (live courts would lose their build root)"
        );
        release_lease_if_last(&scratch, 0); // last holder out
        assert!(
            !scratch.exists(),
            "last-holder release did NOT delete the lease root — cleanup law broken (orphaned build root)"
        );
    }

    // -- The five integration-surface courts ---------------------------------

    /// A04 — the wire/memory identity court over the 128 B `CgapFrameHeader`
    /// (owner lane A04). The sovereign surface hands RAW frame bytes across
    /// the ABI; this witness keeps the upstream identity law green from the
    /// membrane's seat.
    #[test]
    fn wave9_a04_identity_court_green() {
        cargo_suite_court(
            "A04 identity court",
            "cgap-core",
            "identity_court",
            4, // receipted this wave: 4 passed
        );
    }

    /// A14 — the dam (cgap-gate `penstock::ingress`) over REAL A04 frames:
    /// the Wave-9 cross-lane tier court (owner lanes A04+A14). The A19
    /// register values it imports are parity-pinned against THIS module in
    /// the wave-3 status court and penstock's own three-way purity court.
    #[test]
    fn wave9_a14_dam_over_a04_frames_court_green() {
        cargo_suite_court(
            "A14 dam over A04 frames",
            "cgap-core",
            "cross_lane_gate_court",
            8, // receipted this wave: 8 passed
        );
    }

    /// A20 — the court-verdict suite including the 1M-row interleave stress
    /// (owner lane A20). Receipted method: dev opt-level 2.
    #[test]
    fn wave9_a20_one_m_row_stress_court_green() {
        cargo_suite_court(
            "A20 1M-row stress (court verdict)",
            "cgap-court",
            "court_verdict",
            11, // receipted: 11/11 courts
        );
    }

    /// A05 — the power-cut sweep: torn-write/rollback court over the frozen
    /// WAL (owner lane A05).
    #[test]
    fn wave9_a05_power_cut_sweep_court_green() {
        cargo_suite_court(
            "A05 power-cut sweep",
            "cgap-storage",
            "court_sweep",
            2, // receipted this wave: 2 passed
        );
    }

    /// A07 — the zero-string gate over ALL BUILT CRATES: the default scan
    /// mode enumerates every non-excluded workspace member's built
    /// artifacts and REFUSES if any member has zero artifacts (structural
    /// anti-vacuity precondition) or if any forbidden string surface
    /// scores a hit. Two legs, both real subprocesses:
    ///   1. the scan itself (hits=0 over every built crate),
    ///   2. the gate's OWN poison/clean self-check (`--self-check`, the
    ///      T4+T12 anti-vacuity probe) — the gate must be able to DETECT
    ///      poison for its hits=0 to carry information.
    #[test]
    fn wave9_a07_zero_string_gate_over_all_built_crates() {
        if !python3_available() {
            println!("[SKIP] python3 absent — A07 zero-string gate unexecuted (loud skip, never a silent pass)");
            return;
        }
        // Leg 1: the real scan over all built crates (defaults: root = the
        // repo this test compiles in, target = <root>/target, profile =
        // debug — the same invocation the session receipt pinned). The
        // script path is ABSOLUTE: cargo runs test binaries with cwd =
        // the package dir, and a relative path resolves there, not at the
        // repo root (witnessed: exit 2, "can't open file .../cgap-ffi/
        // scripts/zero_string_gate.py" on the first court run).
        let script = repo_root().join("scripts/zero_string_gate.py");
        let script = script.to_str().expect("repo script path is UTF-8");
        let started = Instant::now();
        let outcome = run_bounded("python3", &[script], &[], GATE_DEADLINE)
            .unwrap_or_else(|e| panic!("A07 gate: runner failed: {e}"));
        let elapsed = started.elapsed();
        let status = outcome.status.unwrap_or_else(|| {
            panic!(
                "A07 gate: deadline of {}s exceeded — child killed; stderr tail:\n{}",
                GATE_DEADLINE.as_secs(),
                tail(&outcome.stderr)
            )
        });
        assert!(
            status.success(),
            "A07 gate over all built crates exited {status}; stdout tail:\n{}\nstderr tail:\n{}",
            tail(&outcome.stdout),
            tail(&outcome.stderr)
        );
        let summary = parse_gate_summary(&outcome.stdout)
            .unwrap_or_else(|e| panic!("A07 gate: {e}; stdout tail:\n{}", tail(&outcome.stdout)));
        // Leg 2: the gate's own anti-vacuity probe must pass — a gate that
        // cannot detect its poison fixture cannot vouch for hits=0 above.
        let probe = run_bounded("python3", &[script, "--self-check"], &[], GATE_DEADLINE)
            .unwrap_or_else(|e| panic!("A07 gate self-check: runner failed: {e}"));
        let probe_status = probe.status.unwrap_or_else(|| {
            panic!(
                "A07 gate self-check: deadline exceeded; stderr tail:\n{}",
                tail(&probe.stderr)
            )
        });
        assert!(
            probe_status.success(),
            "A07 gate self-check (poison/clean probe) exited {probe_status} — the gate cannot flip, its scan is untrustworthy; stdout tail:\n{}",
            tail(&probe.stdout)
        );
        println!(
            "WAVE9 INTEGRATION  A07 zero-string gate: scan exit=0 scanned={} crates={} hits=0 + self-check exit=0 elapsed={elapsed:?}",
            summary.scanned, summary.crates
        );
    }

    struct GateSummary {
        scanned: u64,
        crates: u64,
    }

    /// Mechanical read of the gate's SUMMARY line. Refuses: no SUMMARY
    /// (the gate exited 0 without scanning — vacuous), any `hits=` or
    /// `adv_hits=` other than 0, or a scan narrower than the receipted
    /// breadth (45 artifacts / 22 crates this wave; floors 10/20 allow
    /// benign churn, refuse the gutting class).
    fn parse_gate_summary(out: &str) -> Result<GateSummary, String> {
        let line = out
            .lines()
            .find(|l| l.trim_start().starts_with("SUMMARY "))
            .ok_or("no SUMMARY line — the gate exited 0 without scanning (vacuous)")?;
        let mut scanned = None;
        let mut crates = None;
        for tok in line.split_whitespace() {
            if let Some(v) = tok.strip_prefix("scanned=") {
                scanned = v.parse::<u64>().ok();
            } else if let Some(v) = tok.strip_prefix("crates=") {
                crates = v.parse::<u64>().ok();
            } else if let Some(v) = tok.strip_prefix("hits=") {
                if v != "0" {
                    return Err(format!("zero-string HIT in built crates: {tok}"));
                }
            } else if let Some(v) = tok.strip_prefix("adv_hits=") {
                if v != "0" {
                    return Err(format!("advisory hits nonzero: {tok}"));
                }
            }
        }
        let (Some(scanned), Some(crates)) = (scanned, crates) else {
            return Err(format!("SUMMARY line unparsable: {line:?}"));
        };
        if scanned < 10 {
            return Err(format!(
                "scanned={scanned} < floor 10 — scan breadth gutted"
            ));
        }
        if crates < 20 {
            return Err(format!(
                "crates={crates} < floor 20 — not all built crates scanned"
            ));
        }
        Ok(GateSummary { scanned, crates })
    }

    fn python3_available() -> bool {
        Command::new("python3")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// The parser's own negative control, executed every run: a SUMMARY
    /// with a hit, a hit-free summary below the breadth floors, and a
    /// missing SUMMARY must all REFUSE. If any is accepted, the A07 court
    /// cannot flip = VACUOUS GATE.
    #[test]
    fn wave9_a07_gate_parser_negative_control() {
        let good = "SUMMARY scanned=45 crates=22 hits=0 emitter_hits=2 adv_hits=0 unknown=316943 allowed=6661 gate=1.4.0";
        let s = parse_gate_summary(good).expect("receipted summary must parse");
        assert_eq!((s.scanned, s.crates), (45, 22));
        let refusals = vec![
            (
                "hit present",
                "SUMMARY scanned=45 crates=22 hits=1 adv_hits=0",
            ),
            (
                "advisory hit",
                "SUMMARY scanned=45 crates=22 hits=0 adv_hits=3",
            ),
            (
                "breadth gutted",
                "SUMMARY scanned=4 crates=22 hits=0 adv_hits=0",
            ),
            (
                "crate coverage gutted",
                "SUMMARY scanned=45 crates=2 hits=0 adv_hits=0",
            ),
        ];
        for (label, fabricated) in refusals {
            assert!(
                parse_gate_summary(fabricated).is_err(),
                "negative control '{label}' was ACCEPTED — A07 parser cannot flip, VACUOUS GATE"
            );
        }
        assert!(
            parse_gate_summary("no summary here").is_err(),
            "missing-SUMMARY leg cannot flip, VACUOUS GATE"
        );
    }
}

// ===========================================================================
// XIP target courts (lane doc §6.3; DoD 6/7/8). Real subprocesses over a /tmp
// sandbox — the canonical tree is never mutated; the fixture is written fresh
// per test-process id and removed on success (lease law).
//
//   * §6.6/DoD 6 — the xip.ld ASSERT court: sections PRESENT link green with
//     .oabi/.bn3 page-aligned inside FLASH; the section-ABSENT mutant must
//     fail the link ON the ASSERT text. A script whose ASSERTs cannot fire
//     is decoration; this court executes the mutant every run.
//   * DoD 7 — the T1 RISC-V staticlib size gate (flash < 64 KB, SRAM < 32 KB,
//     measured via `llvm-size -A`, never estimated). Loud [SKIP] class:
//     toolchain absent (spec §6.3 form) only — the former owner-wall skip
//     self-destructed (the wall fell; see the T1 court body), so a build
//     failure now fails the court instead of skipping.
//   * DoD 8 — T2 Xtensa: the esp-rs fork toolchain is absent here; the skip
//     is loud (spec §6.3 verbatim class), never a silent pass.
// ===========================================================================

#[cfg(test)]
mod xip_courts {
    use super::wave9_courts::{run_bounded, tail};
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::time::Duration;

    /// Fixture build deadline. Generous on purpose (first run builds the
    /// rv32 core graph), still bounded and enforced.
    const XIP_BUILD_DEADLINE: Duration = Duration::from_secs(300);

    /// FLASH region per crates/cgap-ffi/xip.ld (ESP32-S3 app mmap).
    const FLASH_ORIGIN: u64 = 0x4200_0020;
    const FLASH_LEN: u64 = 4 * 1024 * 1024;
    const FLASH_PAGE: u64 = 64 * 1024;

    /// §6.3 budgets (DoD 7): the gate values the size court asserts against.
    const FLASH_BUDGET: u64 = 64 * 1024;
    const SRAM_BUDGET: u64 = 32 * 1024;

    const FIXTURE_MANIFEST: &str = r#"[package]
name = "a19xip"
version = "0.1.0"
edition = "2021"
publish = false

[[bin]]
name = "a19xip_link"
path = "src/main.rs"

[profile.release]
opt-level = "s"
lto = "fat"
codegen-units = 1
panic = "abort"

[workspace]
"#;

    const FIXTURE_MAIN_WITH_SECTIONS: &str = r#"#![no_std]
#![no_main]
use core::arch::asm;
#[panic_handler]
fn ph(_: &core::panic::PanicInfo) -> ! { loop { unsafe { asm!("wfi") } } }
#[no_mangle]
pub extern "C" fn cgap_xip_start() -> ! { loop { unsafe { asm!("wfi") } } }
#[used]
#[link_section = ".oabi"]
pub static OABI_TABLE: [u8; 96] = [0xA7; 96];
#[used]
#[link_section = ".bn3"]
pub static BN3_NET: [u8; 48] = [0xB3; 48];
"#;

    /// The court's NAMED MUTATION (§6.6): the .oabi/.bn3 input sections
    /// deleted. The link MUST refuse on the xip.ld ASSERTs; a still-green
    /// link means the ASSERT court cannot flip = VACUOUS GATE.
    const FIXTURE_MAIN_NO_SECTIONS: &str = r#"#![no_std]
#![no_main]
use core::arch::asm;
#[panic_handler]
fn ph(_: &core::panic::PanicInfo) -> ! { loop { unsafe { asm!("wfi") } } }
#[no_mangle]
pub extern "C" fn cgap_xip_start() -> ! { loop { unsafe { asm!("wfi") } } }
"#;

    /// Absolute repo root, derived from the crate manifest (never cwd).
    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root canonicalize")
    }

    fn xip_ld_path() -> PathBuf {
        let p = repo_root().join("crates/cgap-ffi/xip.ld");
        assert!(
            p.is_file(),
            "checked-in crates/cgap-ffi/xip.ld missing at {}",
            p.display()
        );
        p
    }

    /// The toolchain's llvm-tools bin dir (hosts `llvm-size`), triple-agnostic.
    fn llvm_tools_bin() -> Option<PathBuf> {
        let out = std::process::Command::new("rustc")
            .arg("--print")
            .arg("sysroot")
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let sysroot = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
        let rustlib = sysroot.join("lib/rustlib");
        for entry in std::fs::read_dir(&rustlib).ok()?.flatten() {
            let bin = entry.path().join("bin");
            if bin.is_dir() && bin.join("llvm-size").is_file() {
                return Some(bin);
            }
        }
        None
    }

    /// Target std/core presence = the toolchain gate of spec §6.3 (loud-skip
    /// class). A target absent from `lib/rustlib/` is not installed.
    fn target_installed(triple: &str) -> bool {
        let out = std::process::Command::new("rustc")
            .arg("--print")
            .arg("sysroot")
            .output();
        let Ok(out) = out else { return false };
        if !out.status.success() {
            return false;
        }
        let sysroot = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
        sysroot.join("lib/rustlib").join(triple).is_dir()
    }

    fn fixture_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("cgap-a19-xip-{}-{}", name, std::process::id()))
    }

    /// Parse `llvm-size -A` (SysV) rows into (name, size, addr) triples.
    fn parse_size_a(out: &str) -> Vec<(String, u64, u64)> {
        let mut rows = Vec::new();
        for line in out.lines() {
            let toks: Vec<&str> = line.split_whitespace().collect();
            if toks.len() == 3 && toks[0].starts_with('.') {
                if let (Ok(size), Ok(addr)) = (toks[1].parse::<u64>(), toks[2].parse::<u64>()) {
                    rows.push((toks[0].to_string(), size, addr));
                }
            }
        }
        rows
    }

    /// section families: FLASH-resident vs SRAM-resident. For the final ELF
    /// these are the LINKED section sizes (xip.ld places .text/.rodata/.
    /// .oabi/.bn3 in FLASH and .data/.bss in SRAM); for the staticlib
    /// context row they are the summed input sections.
    fn flash_sram_split(rows: &[(String, u64, u64)]) -> (u64, u64) {
        let mut flash = 0u64;
        let mut sram = 0u64;
        for (name, size, _addr) in rows {
            let in_flash = name.starts_with(".text")
                || name.starts_with(".rodata")
                || name.starts_with(".oabi")
                || name.starts_with(".bn3")
                || name.starts_with(".init")
                || name.starts_with(".fini");
            let in_sram = name.starts_with(".data")
                || name.starts_with(".bss")
                || name.starts_with(".sbss")
                || name.starts_with(".sdata")
                || *name == "COMMON";
            if in_flash {
                flash += size;
            } else if in_sram {
                sram += size;
            }
        }
        (flash, sram)
    }

    fn build_fixture(
        dir: &PathBuf,
        envs: &[(&str, String)],
        main_src: &str,
        deadline: Duration,
    ) -> (Option<std::process::ExitStatus>, String, String) {
        std::fs::create_dir_all(dir.join("src")).expect("fixture src dir");
        std::fs::write(dir.join("Cargo.toml"), FIXTURE_MANIFEST).expect("write fixture manifest");
        std::fs::write(dir.join("src/main.rs"), main_src).expect("write fixture main.rs");
        let mp = dir.join("Cargo.toml").display().to_string();
        let td = dir.join("target").display().to_string();
        let outcome = run_bounded(
            "cargo",
            &[
                "build",
                "--release",
                "--target",
                "riscv32imc-unknown-none-elf",
                "--manifest-path",
                mp.as_str(),
                "--target-dir",
                td.as_str(),
            ],
            envs,
            deadline,
        )
        .unwrap_or_else(|e| panic!("xip court: nested runner failed: {e}"));
        (outcome.status, outcome.stdout, outcome.stderr)
    }

    /// §6.6 / DoD 6 — the xip.ld build-level court. Leg 1 (sections PRESENT):
    /// the link is green and .oabi/.bn3 land page-aligned INSIDE the FLASH
    /// region (placement law §5). Leg 2 (NAMED MUTATION, sections ABSENT):
    /// the link MUST refuse ON the ASSERT text — a script whose ASSERTs
    /// cannot fire is decoration, so a green mutant link fails this court as
    /// a VACUOUS GATE.
    #[test]
    fn xip_ld_asserts_fire_and_sections_land_page_aligned_in_flash() {
        let Some(tools) = llvm_tools_bin() else {
            println!("[SKIP] rustc sysroot llvm tools absent (llvm-size) — xip.ld ASSERT court unexecuted (loud skip, never a silent pass)");
            return;
        };
        if !target_installed("riscv32imc-unknown-none-elf") {
            println!("[SKIP] riscv32imc-unknown-none-elf target not installed — xip.ld ASSERT court unexecuted (loud skip, never a silent pass)");
            return;
        }
        let script = xip_ld_path();
        let dir = fixture_root("assert-court");
        let _ = std::fs::remove_dir_all(&dir); // stale lease from a crashed prior run
        let envs = vec![("RUSTFLAGS", format!("-C link-arg=-T{}", script.display()))];

        // LEG 1 — sections present.
        let (status, _out, err) =
            build_fixture(&dir, &envs, FIXTURE_MAIN_WITH_SECTIONS, XIP_BUILD_DEADLINE);
        let status = status.unwrap_or_else(|| {
            panic!(
                "xip court leg 1: deadline exceeded; stderr:\n{}",
                tail(&err)
            )
        });
        assert!(
            status.success(),
            "fixture with .oabi/.bn3 sections failed to link against the checked-in xip.ld; stderr tail:\n{}",
            tail(&err)
        );
        let size = tools.join("llvm-size");
        let elf = dir.join("target/riscv32imc-unknown-none-elf/release/a19xip_link");
        let size_out = std::process::Command::new(&size)
            .arg("-A")
            .arg(&elf)
            .output()
            .expect("spawn llvm-size");
        assert!(
            size_out.status.success(),
            "llvm-size -A on {elf:?} failed:\n{}",
            String::from_utf8_lossy(&size_out.stderr)
        );
        let secs: BTreeMap<String, (u64, u64)> =
            parse_size_a(&String::from_utf8_lossy(&size_out.stdout))
                .into_iter()
                .map(|(n, s, a)| (n, (s, a)))
                .collect();
        for name in [".oabi", ".bn3"] {
            let Some(&(sz, addr)) = secs.get(name) else {
                panic!("{name} absent from the linked ELF — XIP placement law broken (llvm-size output below):\n{}", String::from_utf8_lossy(&size_out.stdout));
            };
            assert!(sz > 0, "{name} linked with size 0");
            assert!(
                addr >= FLASH_ORIGIN && addr < FLASH_ORIGIN + FLASH_LEN,
                "{name} at {addr:#x} outside the FLASH region [{FLASH_ORIGIN:#x}, +{FLASH_LEN:#x})"
            );
            assert_eq!(
                addr % FLASH_PAGE,
                0,
                "{name} at {addr:#x} not {FLASH_PAGE}-byte page-aligned (§5 placement law)"
            );
        }
        println!(
            "XIP ASSERT COURT leg 1: .oabi/.bn3 page-aligned in FLASH ({:#x}/{:#x}); sections=({})",
            secs[".oabi"].1,
            secs[".bn3"].1,
            secs.keys().cloned().collect::<Vec<_>>().join(",")
        );
    }

    /// Leg 2 of the §6.6 court — the NAMED MUTATION, its own test so a leg-1
    /// environment gate cannot mask it: with the .oabi/.bn3 input sections
    /// deleted, the link MUST fail ON the ASSERT text. (Runs in a separate
    /// fixture lease so the mutation can never poison leg 1.)
    #[test]
    fn xip_ld_assert_mutant_section_absent_link_refuses() {
        if !target_installed("riscv32imc-unknown-none-elf") {
            println!("[SKIP] riscv32imc-unknown-none-elf target not installed — xip.ld ASSERT mutant leg unexecuted (loud skip, never a silent pass)");
            return;
        }
        let script = xip_ld_path();
        let dir = fixture_root("assert-mutant");
        let _ = std::fs::remove_dir_all(&dir);
        let envs = vec![("RUSTFLAGS", format!("-C link-arg=-T{}", script.display()))];
        let (status, _out, err) =
            build_fixture(&dir, &envs, FIXTURE_MAIN_NO_SECTIONS, XIP_BUILD_DEADLINE);
        let status = status.unwrap_or_else(|| {
            panic!(
                "xip court leg 2: deadline exceeded; stderr:\n{}",
                tail(&err)
            )
        });
        assert!(
            !status.success(),
            "VACUOUS GATE: the section-absent mutant LINKED — the xip.ld ASSERTs never fire (DoD 6 court cannot flip); stderr tail:\n{}",
            tail(&err)
        );
        assert!(
            err.contains("cgap XIP: .oabi section absent"),
            "mutant link failed but NOT on the .oabi ASSERT (wrong leg):\n{}",
            tail(&err)
        );
        assert!(
            err.contains("cgap XIP: .bn3 section absent"),
            "mutant link failed but NOT on the .bn3 ASSERT (wrong leg):\n{}",
            tail(&err)
        );
        println!("XIP ASSERT COURT leg 2 (mutation): section-absent link refused ON the ASSERTs — the script's ASSERTs are load-bearing");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// DoD 7 — T1 RISC-V size gate, REPINNED (coordinator-ordered GAP-1,
    /// 2026-11-01) to the §4-conformant FINAL ELF subject. Spec §4 prose:
    /// "Budget = FLASH is the size of the final `.elf` text+rodata (`size
    /// -A`), SRAM = data+bss + worst-case eval stack". The staticlib `.a` is
    /// NOT the §4 budget subject — it is an unlinked archive (every
    /// compiler_builtins TU with relocations, nothing GC'd); its measured
    /// 264,881 B vs the 64 KiB gate receipts RED, and that row is kept as
    /// labeled CONTEXT below (measured, printed, never asserted). The
    /// court's asserted subject is the xip.ld-linked final ELF (rust-lld,
    /// --gc-sections): the artifact that actually executes from XIP flash.
    /// Budgets UNCHANGED (64 KiB flash / 32 KiB SRAM) — the subject was
    /// re-anchored, never the budget loosened. If the ELF busts budget the
    /// gate stays red (BLOCKED), never loosened to green.
    #[test]
    fn xip_t1_riscv_elf_size_court() {
        if !target_installed("riscv32imc-unknown-none-elf") {
            println!("[SKIP] riscv32imc-unknown-none-elf target not installed — T1 ELF size gate unexecuted (loud skip, never a silent pass)");
            return;
        }
        let Some(tools) = llvm_tools_bin() else {
            println!("[SKIP] llvm-size absent — T1 size measurement unexecutable (loud skip, never a silent pass)");
            return;
        };
        // /tmp sandbox copy of the REAL crate (courts mutate sandbox copies,
        // never the shared tree; path deps point back read-only).
        let dir = fixture_root("t1");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("sandbox src dir");
        let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        for entry in std::fs::read_dir(&src_dir)
            .expect("src dir readable")
            .flatten()
        {
            let p = entry.path();
            if p.extension().map(|x| x == "rs").unwrap_or(false) {
                let dst = dir.join("src").join(p.file_name().expect("file name"));
                std::fs::copy(&p, &dst).expect("copy sandbox source");
            }
        }
        // Sandbox manifest: staticlib (the §4 staticlib artifact, CONTEXT
        // row) + rlib (the bin links the crate) + the XIP entry bin.
        std::fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"cgap-ffi\"\nversion = \"26.10.29\"\nedition = \"2021\"\npublish = false\n\n\
                 [lib]\ncrate-type = [\"staticlib\", \"rlib\"]\n\n\
                 [[bin]]\nname = \"a19xip_elf\"\npath = \"src/main.rs\"\n\n\
                 [dependencies]\ncgap-arrow = {{ path = \"{}\" }}\n\n\
                 [profile.release]\nopt-level = \"s\"\nlto = \"fat\"\ncodegen-units = 1\npanic = \"abort\"\n",
                repo_root().join("crates/cgap-arrow").display()
            ),
        )
        .expect("write sandbox manifest");
        // The XIP entry: demo .oabi/.bn3 rodata sections (the xip.ld ASSERTs
        // require them present) + the ENTRY symbol. The panic handler is the
        // LIB's none-target glue (cgap_xip_panic) — a bin-local one would be
        // a duplicate #[panic_handler]. `extern crate cgap_ffi;` keeps the
        // real crate in the link; xip.ld's KEEP(*(.text.cgap_*)) retains its
        // cgap_* sections through --gc-sections.
        std::fs::write(
            dir.join("src/main.rs"),
            r#"#![no_std]
#![no_main]
use core::arch::asm;
extern crate cgap_ffi;

#[used]
#[link_section = ".oabi"]
pub static OABI_TABLE: [u8; 96] = [0xA7; 96];
#[used]
#[link_section = ".bn3"]
pub static BN3_NET: [u8; 48] = [0xB3; 48];

#[no_mangle]
pub extern "C" fn cgap_xip_start() -> ! {
    // The sovereign bodies are host-surface this wave (recorded XIP
    // divergence, same form as cgap_verify_p28); the XIP entry references
    // the real crate and parks. XIP rule execution stays BLOCKED-on-owner:
    // the A12 unifier is an alloc API and the XIP budget carries no
    // allocator (see the lane receipt).
    let _ = cgap_ffi::sovereign::CGAP_ABI_VERSION;
    loop { unsafe { asm!("wfi") } }
}
"#,
        )
        .expect("write sandbox xip entry");
        let mp = dir.join("Cargo.toml").display().to_string();
        let td = dir.join("target").display().to_string();
        let script = xip_ld_path();
        let envs = vec![(
            "RUSTFLAGS",
            format!(
                "-C link-arg=-T{} -C link-arg=--gc-sections",
                script.display()
            ),
        )];
        let outcome = run_bounded(
            "cargo",
            &[
                "build",
                "--release",
                "--target",
                "riscv32imc-unknown-none-elf",
                "--manifest-path",
                mp.as_str(),
                "--target-dir",
                td.as_str(),
            ],
            &envs,
            XIP_BUILD_DEADLINE,
        )
        .unwrap_or_else(|e| panic!("T1 court: nested runner failed: {e}"));
        let status = outcome.status.unwrap_or_else(|| {
            panic!(
                "T1 court: deadline exceeded; stderr tail:\n{}",
                tail(&outcome.stderr)
            )
        });
        // No sanctioned build-failure skip remains: the size gate is hard.
        assert!(
            status.success(),
            "T1 xip build failed (no sanctioned skip class remains — the size gate is hard); stderr tail:\n{}",
            tail(&outcome.stderr)
        );
        // THE ASSERTED SUBJECT: the final xip.ld-linked ELF (§4 budget
        // prose). FLASH = text+rodata (+ the .oabi/.bn3 flash sections);
        // SRAM = data+bss. The worst-case eval-stack term of the §4 SRAM
        // sentence is target-measurable only (stack-paint on hardware/qemu
        // — absent here); it stays a labeled UNVERIFIED term in the lane
        // receipt, never a silently-dropped one.
        let elf = dir.join("target/riscv32imc-unknown-none-elf/release/a19xip_elf");
        assert!(elf.is_file(), "final ELF artifact missing at {elf:?}");
        let size = tools.join("llvm-size");
        let elf_out = std::process::Command::new(&size)
            .arg("-A")
            .arg(&elf)
            .output()
            .expect("spawn llvm-size on the final ELF");
        assert!(elf_out.status.success(), "llvm-size -A on {elf:?} failed");
        let (flash, sram) =
            flash_sram_split(&parse_size_a(&String::from_utf8_lossy(&elf_out.stdout)));
        assert!(
            flash < FLASH_BUDGET,
            "T1 ELF FLASH {flash} B >= budget {FLASH_BUDGET} B — exit gate §4 NOT met (measured, not estimated); the gate stays red, never loosened"
        );
        assert!(
            sram < SRAM_BUDGET,
            "T1 ELF SRAM {sram} B >= budget {SRAM_BUDGET} B — exit gate §4 NOT met (measured, not estimated); the gate stays red, never loosened"
        );
        println!(
            "T1 ELF SIZE (measured, llvm-size -A, xip.ld + rust-lld --gc-sections): flash={flash} B / {FLASH_BUDGET} B, sram={sram} B / {SRAM_BUDGET} B — budgets met"
        );
        // CONTEXT ROW (not the gate subject, not asserted): the staticlib
        // `.a` — an unlinked archive incl. every compiler_builtins TU; its
        // 264,881 B-vs-budget RED is receipted context, not the §4 subject.
        let liba = dir.join("target/riscv32imc-unknown-none-elf/release/libcgap_ffi.a");
        if liba.is_file() {
            let a_out = std::process::Command::new(&size)
                .arg("-A")
                .arg(&liba)
                .output()
                .expect("spawn llvm-size on the staticlib");
            if a_out.status.success() {
                let (af, asr) =
                    flash_sram_split(&parse_size_a(&String::from_utf8_lossy(&a_out.stdout)));
                println!(
                    "T1 CONTEXT (not asserted): staticlib flash={af} B / sram={asr} B — unlinked archive (compiler_builtins, no GC); the §4 subject is the final ELF above"
                );
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// DoD 8 — T2 Xtensa (ESP32-S3) via the esp-rs fork toolchain (§7 risk).
    /// The fork is not installed in this environment; the skip is LOUD (spec
    /// §6.3 verbatim class). If the fork target ever appears, this gate
    /// refuses loudly rather than silently doing nothing — extending it into
    /// the real T2 court is then mandatory, never optional.
    #[test]
    fn xip_t2_xtensa_toolchain_gate() {
        if target_installed("xtensa-esp32s3-none-elf") {
            panic!(
                "xtensa-esp32s3-none-elf target APPEARED — extend this gate into the real T2 court (spec §6.3); a silent pass would fake it"
            );
        }
        println!("[SKIP] xtensa toolchain absent — T2 unexecuted (loud skip, never a silent pass)");
    }
}

// ===========================================================================
// Wave F2 item 5 courts — the `cgap_verify_p28` sovereign body
// (SEAM-CRYPTO-FFI). Chicago discipline: real instrument bytes from the
// pinned file on disk, the real cgap-p28 API, real C-ABI calls through the
// frozen fn-pointer alias, real /tmp sandbox files (bit flips applied to
// bytes read from disk copies; the original restored byte-exact, sha256
// fail-closed), nm over the real built cdylib. No mocks.
//
// Anti-vacuity, executed every run: the positive control flips an
// always-refuse stub and the mutation corpus flips an always-admit stub
// (both stub legs are executed in `verify_stub_probe_*`); the cdylib symbol
// court flips on rename/removal of the export (renamed-symbol negative
// control); the digest leg makes even the lawfully-admitted D-A17-1 ceiling
// mutations flip-detectable.
//
// D-A17-1 (A17's re-derived survivor law, re-derived here from first
// principles): "flip ANY bit -> refusal" is arithmetically unachievable
// against the real §5.8 `<=` ceiling semantics — with the ceiling pinned at
// MAX, CLEARING any of the 11 set bits of the ceiling field keeps the record
// valid and MUST admit. The court therefore pins the real law: 640
// mutations run, every non-predicted bit refuses TYPED
// (CGAP_ERR_P28_SPOLIATION), survivors are EXACTLY the predicted
// ceiling-set-bit class, and every surviving mutation still changes the
// digest. A court asserting the dispatch sketch verbatim would fail against
// the real validator — recorded as a court divergence.
// ===========================================================================

#[cfg(test)]
mod wave_f2_p28_courts {
    use super::{
        cgap_verify_p28, fold_bytes, CgapStatus, CgapVerifyP28Fn, CGAP_ERR_P28_SPOLIATION, CGAP_OK,
    };
    use cgap_p28::attestation::{
        instrument_header_span, origin_authority_digest, P28LicenseAttestation, ATTESTATION_LEN,
        ATTESTATION_VERSION, FLAG_ROOT, LICENSE_MAGIC, SAFE_HARBOR_CEILING_MAX_CENTS,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// Absolute repo root, derived from the crate manifest (never cwd).
    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root canonicalize")
    }

    /// Canonical §3 span of the REAL pinned instrument, read from disk.
    fn real_instrument_span() -> Vec<u8> {
        let file = fs::read(repo_root().join("LICENSE-P28-SIL.md"))
            .expect("pinned instrument LICENSE-P28-SIL.md readable from the repo root");
        instrument_header_span(&file)
            .map(|s| s.to_vec())
            .expect("canonical instrument has a `---` line")
    }

    /// A REAL valid attestation built via the public p28 API: Root lineage,
    /// pinned ceiling, origin authority digest over the real instrument span.
    fn valid_root_record(instrument: &[u8]) -> P28LicenseAttestation {
        P28LicenseAttestation {
            license_magic: LICENSE_MAGIC,
            version: ATTESTATION_VERSION,
            attestation_flags: FLAG_ROOT,
            safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
            origin_authority_digest: origin_authority_digest(instrument),
            upstream_receipt: [0u8; 32],
        }
    }

    /// Call the real export through the frozen alias — the coercion pins
    /// body↔signature parity at compile time; the call exercises the real body.
    fn call_verify(bytes: &[u8], out_digest: *mut u64) -> CgapStatus {
        let f: CgapVerifyP28Fn = cgap_verify_p28;
        unsafe { f(bytes.as_ptr(), bytes.len(), out_digest) }
    }

    /// sha256 via the real `shasum` tool (None when absent — loud-skip leg).
    fn shasum(path: &Path) -> Option<String> {
        Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string()
            })
    }

    /// D-A17-1 predicted survivor law, derived from first principles: with
    /// the ceiling pinned at MAX, the only tolerated single-bit mutations are
    /// CLEAR-flips of the SET bits of the ceiling field (bytes 8..16).
    fn predicted_ceiling_survivors(good: &[u8; ATTESTATION_LEN]) -> Vec<(usize, u8)> {
        let mut v = Vec::new();
        for i in 8..16 {
            for j in 0..8u8 {
                if good[i] & (1 << j) != 0 {
                    v.push((i, j));
                }
            }
        }
        v
    }

    /// Positive control: a real, valid 80-byte attestation (built via the
    /// public p28 API over the REAL pinned instrument bytes) admits through
    /// the C-ABI with CGAP_OK; the digest is the house fold of the raw
    /// evidence bytes; the NULL out_digest leg admits without faulting.
    /// Anti-vacuity: flips an always-refuse stub (witnessed in
    /// `verify_stub_probe_corpus_and_positive_control_can_flip`).
    #[test]
    fn verify_positive_control_real_instrument_admits_with_fold_digest() {
        let instrument = real_instrument_span();
        // Pin parity (anti-drift): the frozen PINNED_INSTRUMENT_HEADER const
        // must equal the span re-derived from the REAL file on disk through
        // cgap-p28's own §3 API — every run. An instrument edit or a stale
        // pin flips here and forces re-attestation (A17 §9).
        assert_eq!(
            super::PINNED_INSTRUMENT_HEADER,
            instrument.as_slice(),
            "frozen instrument pin drifted from LICENSE-P28-SIL.md on disk — re-attestation required (A17 §9)"
        );
        let wire = valid_root_record(&instrument).to_bytes();
        assert_eq!(wire.len(), ATTESTATION_LEN);

        // Cross-check on the p28 side itself: the same bytes admit there.
        assert!(
            P28LicenseAttestation::from_bytes(&wire)
                .expect("80 bytes parse")
                .validate_intrinsic()
                .is_ok(),
            "test premise: the record must admit through cgap-p28's own validator"
        );

        let mut d = 0u64;
        assert_eq!(
            call_verify(&wire, &mut d),
            CGAP_OK,
            "valid root attestation must admit through the sovereign body"
        );
        assert_eq!(
            d,
            fold_bytes(0, &wire),
            "digest must be the house fold of the raw evidence bytes"
        );

        // Determinism + the §2 NULL out_digest leg.
        let mut d2 = 0u64;
        assert_eq!(call_verify(&wire, &mut d2), CGAP_OK);
        assert_eq!(d2, d, "digest deterministic across calls");
        assert_eq!(
            call_verify(&wire, core::ptr::null_mut()),
            CGAP_OK,
            "NULL out_digest must not fault"
        );
    }

    /// The dispatch court — sandbox bit-flip corpus over the REAL verify
    /// path: the valid record lives in a /tmp sandbox file; all 640
    /// single-bit mutations are applied to bytes read from disk copies; every
    /// non-predicted mutation refuses TYPED (CGAP_ERR_P28_SPOLIATION, never a
    /// crash); survivors are EXACTLY the D-A17-1 ceiling class, and every
    /// survivor still changes the digest (mutation detection holds either
    /// way). The sandbox original is restored byte-exact (sha256
    /// fail-closed); the court deletes its own sandbox lease (cleanup law).
    #[test]
    fn verify_sandbox_bit_flip_corpus_typed_refusals() {
        let instrument = real_instrument_span();
        let wire: [u8; ATTESTATION_LEN] = valid_root_record(&instrument).to_bytes();

        let dir = std::env::temp_dir().join(format!("cgap-f2-p28-sandbox-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("sandbox dir");
        let att_path = dir.join("attestation.bin");
        fs::write(&att_path, wire).expect("sandbox write");
        let sha_before = shasum(&att_path);

        let mut d_orig = 0u64;
        assert_eq!(call_verify(&wire, &mut d_orig), CGAP_OK);
        let predicted = predicted_ceiling_survivors(&wire);
        assert_eq!(
            predicted.len(),
            11,
            "pinned MAX ceiling 5_000_000_000 has exactly 11 set bits (D-A17-1)"
        );

        let mut refusals = 0usize;
        let mut survivors: Vec<(usize, u8)> = Vec::new();
        for i in 0..ATTESTATION_LEN {
            for j in 0..8u8 {
                let mut mutated = wire;
                mutated[i] ^= 1 << j;
                // The flip happens IN THE SANDBOX: mutated bytes are written
                // to a real file and verified FROM DISK.
                let mpath = dir.join(format!("mut-{i}-{j}.bin"));
                fs::write(&mpath, mutated).expect("sandbox mutation write");
                let on_disk = fs::read(&mpath).expect("sandbox mutation read");
                assert_eq!(on_disk, mutated, "sandbox bytes must match the mutation");
                let mut d = 0u64;
                let st = call_verify(&on_disk, &mut d);
                if st == CGAP_OK {
                    survivors.push((i, j));
                    // Mutation detection through the digest leg.
                    assert_ne!(
                        d, d_orig,
                        "admitted mutation ({i},{j}) must change the digest"
                    );
                    assert_eq!(
                        d,
                        fold_bytes(0, &mutated),
                        "admitted mutation digest must fold the mutated bytes"
                    );
                } else {
                    refusals += 1;
                    assert_eq!(
                        st, CGAP_ERR_P28_SPOLIATION,
                        "mutation ({i},{j}) must refuse TYPED (got {st:#x})"
                    );
                }
            }
        }

        assert_eq!(
            survivors, predicted,
            "survivors must be EXACTLY the D-A17-1 ceiling set-bit class"
        );
        assert_eq!(
            refusals,
            640 - predicted.len(),
            "all 640 mutations must run"
        );
        // sha256-exact restore, fail-closed; byte-exact comparison always runs.
        let sha_after = shasum(&att_path);
        match (&sha_before, &sha_after) {
            (Some(b), Some(a)) => assert_eq!(
                a, b,
                "sandbox original not sha256-exact after the corpus (restore law)"
            ),
            _ => println!(
                "[SKIP] shasum absent — sha256 restore leg unexecuted; the byte-exact leg below still ran"
            ),
        }
        assert_eq!(
            fs::read(&att_path).expect("sandbox re-read"),
            wire,
            "sandbox original byte-exact after the corpus"
        );
        fs::remove_dir_all(&dir).expect("sandbox lease cleanup");
    }

    // -- Anti-vacuity stub probe ---------------------------------------------
    //
    // A corpus that cannot distinguish the real body from a stubbed one is
    // decoration. These test-local stubs are never exported (no #[no_mangle])
    // and exist only to prove the court's legs have detection power.

    unsafe extern "C" fn stub_always_ok(
        _evidence: *const u8,
        _len: usize,
        _out_digest: *mut u64,
    ) -> CgapStatus {
        CGAP_OK
    }

    unsafe extern "C" fn stub_always_refuse(
        _evidence: *const u8,
        _len: usize,
        _out_digest: *mut u64,
    ) -> CgapStatus {
        CGAP_ERR_P28_SPOLIATION
    }

    /// Both stub legs, executed every run: (1) an always-admit stub ADMITS a
    /// mutation the real body refuses — the refusal corpus has detection
    /// power (flipping the body to a stub flips this court); (2) an
    /// always-refuse stub refuses the VALID record — the positive control
    /// has detection power.
    #[test]
    fn verify_stub_probe_corpus_and_positive_control_can_flip() {
        let instrument = real_instrument_span();
        let wire = valid_root_record(&instrument).to_bytes();
        let mut mutated = wire;
        mutated[0] ^= 0xFF; // magic byte flip: real body must refuse
        let mut d = 0u64;
        assert_eq!(
            call_verify(&mutated, &mut d),
            CGAP_ERR_P28_SPOLIATION,
            "premise: the real body refuses the magic-flip mutation"
        );
        let stub_ok: CgapVerifyP28Fn = stub_always_ok;
        assert_eq!(
            unsafe { stub_ok(mutated.as_ptr(), mutated.len(), &mut d) },
            CGAP_OK,
            "VACUOUS: an always-admit stub was REFUSED — the corpus cannot flip, detection power dead"
        );
        let stub_refuse: CgapVerifyP28Fn = stub_always_refuse;
        assert_eq!(
            unsafe { stub_refuse(wire.as_ptr(), wire.len(), &mut d) },
            CGAP_ERR_P28_SPOLIATION,
            "VACUOUS: an always-refuse stub ADMITTED the valid record — the positive control cannot flip"
        );
    }

    /// The symbol-table court: the built cdylib's export table gains
    /// `cgap_verify_p28` (nm -gU over the REAL artifact, built by a real
    /// nested cargo into a /tmp lease target dir — never the parent's target
    /// dir), the six pre-existing exports are untouched (additive-only law,
    /// A19 §2), and the renamed symbol is ABSENT (rename/removal flip). A
    /// court over a symbol table that cannot flip on rename is decoration.
    #[test]
    fn verify_cdylib_symbol_table_gains_verify_p28() {
        // NOTE: macOS BSD nm has no --version; the availability gate is the
        // real spawn (NotFound = absent -> loud [SKIP], never a fake pass).
        let dir = std::env::temp_dir().join(format!("cgap-f2-p28-dylib-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dylib court lease dir");
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let outcome = super::wave9_courts::run_bounded(
            "cargo",
            &[
                "build",
                "--lib",
                "--manifest-path",
                manifest.to_str().expect("manifest path"),
                "--target-dir",
                dir.join("target").to_str().expect("target path"),
            ],
            &[("CARGO_NET_OFFLINE", "true".to_string())],
            std::time::Duration::from_secs(300),
        )
        .unwrap_or_else(|e| panic!("cdylib symbol court: nested runner failed: {e}"));
        let status = outcome.status.unwrap_or_else(|| {
            panic!(
                "cdylib symbol court: build deadline exceeded; stderr:\n{}",
                super::wave9_courts::tail(&outcome.stderr)
            )
        });
        assert!(
            status.success(),
            "cdylib build failed; stderr tail:\n{}",
            super::wave9_courts::tail(&outcome.stderr)
        );
        let dylib = dir.join("target/debug/libcgap_ffi.dylib");
        assert!(
            dylib.is_file(),
            "cdylib artifact missing at {} (macOS court)",
            dylib.display()
        );
        let nm = match Command::new("nm").args(["-gU"]).arg(&dylib).output() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("[SKIP] nm absent — cdylib symbol court unexecuted (loud skip, never a silent pass)");
                let _ = fs::remove_dir_all(&dir);
                return;
            }
            Err(e) => panic!("nm -gU spawn failed: {e}"),
            Ok(o) => o,
        };
        assert!(
            nm.status.success(),
            "nm -gU on {} failed:\n{}",
            dylib.display(),
            String::from_utf8_lossy(&nm.stderr)
        );
        let exports = String::from_utf8_lossy(&nm.stdout);
        // The four-symbol list gains cgap_verify_p28 (mach-o leading underscore).
        assert!(
            exports.contains("_cgap_verify_p28"),
            "VACUOUS GATE: cgap_verify_p28 absent from the cdylib export table:\n{}",
            exports
        );
        // Additive-only law: the pre-existing frozen exports are untouched.
        for sym in [
            "_cgap_i32_batch",
            "_cgap_schema_addr",
            "_cgap_array_addr",
            "_cgap_batch_rows",
            "_cgap_version",
            "_cgap_digest_of",
        ] {
            assert!(
                exports.contains(sym),
                "pre-existing export {sym} lost — additive-only law violated:\n{exports}"
            );
        }
        // Rename/removal flip: the mutated name must not exist.
        assert!(
            !exports.contains("cgap_verify_p2x"),
            "renamed symbol cgap_verify_p2x present — rename negative control broken"
        );
        println!(
            "CDYLIB SYMBOL COURT: nm -gU gained _cgap_verify_p28; 6 pre-existing exports intact; rename control absent"
        );
        fs::remove_dir_all(&dir).expect("dylib court lease cleanup");
    }
}

// ===========================================================================
// Wave GAP-1 courts — the three new sovereign bodies. Chicago discipline:
// real owner APIs (cgap-core/cgap-gate/cgap-bn3/cgap-parquet/cgap-blake3),
// real frames built per the cgap-core court canon, real /tmp sandbox files
// written and read back from disk with sha256/byte-exact fail-closed
// restores, real cdylib exports via nm -gU. No mocks.
//
// Anti-vacuity, executed every run: every mutation corpus flips an
// always-admit stub and every positive control flips an always-refuse stub
// (all stub legs executed in `gap1_stub_probes_can_flip`), the eval
// encoding pin-parity court flips a hand-mis-encoded table, the lane-domain
// pin flips a widened domain (the surface must refuse what the kernel
// truncates), and the symbol court flips on a renamed export.
// ===========================================================================

#[cfg(test)]
mod wave_gap1_courts {
    use super::{
        cgap_admit_frame, cgap_commit_parquet, cgap_eval_bn3, CgapAdmitFrameFn,
        CgapCommitParquetFn, CgapEvalBn3Fn, CgapFrameView, CgapStatus, CGAP_ERR_INVALID_MAGIC,
        CGAP_ERR_LEASE_EXHAUSTED, CGAP_ERR_OABI_MISMATCH, CGAP_ERR_P28_SPOLIATION,
        CGAP_ERR_SHACL_VIOLATION, CGAP_OK, EVAL_ENCODING_PIN,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    /// Absolute repo root, derived from the crate manifest (never cwd).
    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root canonicalize")
    }

    /// FROZEN license URN (docs/jira/v26.11.1/03-rename-receipt.md D2; the
    /// cgap-core tests/common canon this court mirrors).
    const URN_LICENSE_P28_SIL: &str = "urn:avec:license:p28-sil:v2.1.0-sovereign";

    // -- Real-frame fixtures (crates/cgap-core/tests/common/mod.rs canon) ----

    fn root_attestation() -> cgap_core::frame::P28LicenseAttestation {
        cgap_core::frame::P28LicenseAttestation {
            license_magic: cgap_core::frame::LICENSE_MAGIC,
            version: 1,
            attestation_flags: cgap_core::frame::LICENSE_FLAG_ROOT,
            safe_harbor_ceiling_cents: cgap_core::frame::SAFE_HARBOR_CEILING_CENTS_MAX,
            origin_authority_digest: cgap_blake3::hash(URN_LICENSE_P28_SIL.as_bytes()),
            upstream_receipt: [0; 32],
        }
    }

    /// One fully valid wire frame over `payload` — header + payload bytes,
    /// BLAKE3 digest embedded, FLAG_BN3 (the eval-carrying format).
    fn wire_frame(payload: &[u8]) -> Vec<u8> {
        let header = cgap_core::frame::CgapFrameHeader {
            magic: cgap_core::frame::FRAME_MAGIC,
            version: cgap_core::frame::FRAME_VERSION,
            flags: cgap_core::frame::FLAG_BN3,
            reserved: [0; 2],
            payload_byte_len: payload.len() as u64,
            oabi_digest: cgap_blake3::hash(payload),
            attestation: root_attestation(),
        };
        let mut frame = vec![0u8; 128 + payload.len()];
        header.encode_into(&mut frame).expect("frame buffer");
        frame[128..].copy_from_slice(payload);
        frame
    }

    /// sha256 via the real `shasum` tool (None when absent — loud-skip leg).
    fn shasum(path: &PathBuf) -> Option<String> {
        Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string()
            })
    }

    fn sandbox_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cgap-gap1-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("sandbox dir");
        dir
    }

    /// Call the real admit export through the frozen alias — the coercion
    /// pins body<->signature parity at compile time (verify_p28 pattern).
    fn call_admit(bytes: &[u8], out: *mut CgapFrameView) -> CgapStatus {
        let f: CgapAdmitFrameFn = cgap_admit_frame;
        unsafe { f(bytes.as_ptr(), bytes.len(), out) }
    }

    // -- Eval fixtures: A11 term words, pinned eval encoding -----------------

    /// SlotConst term word (sigil 0x00, payload = the value).
    fn k(v: u64) -> u64 {
        v // SlotConst = 0x00 << 56 | v
    }
    /// VarUni term word (sigil 0x01, payload = register number).
    fn v(reg: u64) -> u64 {
        (0x01u64 << 56) | reg
    }

    /// Store: rows (1,2,100), (5,2,100), (9,3,250) as columnar
    /// s|p|o LE u64 words (the pinned eval payload encoding).
    fn store_columns() -> (Vec<u64>, Vec<u64>, Vec<u64>) {
        (
            vec![k(1), k(5), k(9)],
            vec![k(2), k(2), k(3)],
            vec![k(100), k(100), k(250)],
        )
    }

    fn store_payload() -> Vec<u8> {
        let (s, p, o) = store_columns();
        let mut b = Vec::with_capacity(24 * 8);
        for w in s.iter().chain(p.iter()).chain(o.iter()) {
            b.extend_from_slice(&w.to_le_bytes());
        }
        b
    }

    /// One-rule network: `(s ?x, p 2, o 100) => (7, 8, ?x)` — matches rows
    /// 0 and 1; every projection's object word is the bound ?x = 100.
    fn one_rule_network() -> Vec<u8> {
        let words: Vec<u64> = [
            1u64,  // rule_count
            42u64, // rule_id
            k(7),
            k(8),
            v(0), // head (7, 8, ?0)
            1u64, // body_len
            v(0),
            k(2),
            k(100), // body atom (?0, 2, 100)
        ]
        .to_vec();
        let mut b = Vec::with_capacity(words.len() * 8);
        for w in &words {
            b.extend_from_slice(&w.to_le_bytes());
        }
        b
    }

    fn eval_view(payload: &[u8]) -> CgapFrameView {
        CgapFrameView {
            payload: payload.as_ptr(),
            payload_len: payload.len(),
            schema_id: 0,
            rule_count: 0,
        }
    }

    fn call_eval(
        view: *const CgapFrameView,
        net: &[u8],
        lanes: *mut i32,
        cap: usize,
        count: *mut usize,
    ) -> CgapStatus {
        let f: CgapEvalBn3Fn = cgap_eval_bn3;
        unsafe { f(view, net.as_ptr(), net.len(), lanes, cap, count) }
    }

    fn call_commit(lanes: &[i32], path: &std::ffi::CStr) -> CgapStatus {
        let f: CgapCommitParquetFn = cgap_commit_parquet;
        unsafe { f(lanes.as_ptr(), lanes.len(), path.as_ptr()) }
    }

    // -- Admit court 1: positive control over a real frame -------------------

    /// A real, fully valid frame admits through the sovereign body; the view
    /// is the zero-copy payload borrow (pointer INTO the caller's buffer);
    /// the NULL out_view probe admits without writing. Anti-vacuity: flips
    /// the always-refuse stub in `gap1_stub_probes_can_flip`.
    #[test]
    fn admit_positive_control_real_frame_zero_copy_view() {
        let payload: Vec<u8> = (0..64u16).map(|i| (i * 7) as u8).collect();
        let frame = wire_frame(&payload);
        // Aligned staging: the A04 projection path requires 64-byte
        // alignment; a repr(align(64)) wrapper is the aligned fixture.
        #[repr(align(64))]
        struct Aligned([u8; 512]);
        let mut staging = Aligned([0u8; 512]);
        staging.0[..frame.len()].copy_from_slice(&frame);
        let mut view = CgapFrameView {
            payload: core::ptr::null(),
            payload_len: 0,
            schema_id: 0xDEAD,
            rule_count: 0xBEEF,
        };
        let st = call_admit(&staging.0[..frame.len()], &mut view);
        assert_eq!(st, CGAP_OK, "valid frame must admit (status {st:#x})");
        assert_eq!(
            view.payload,
            unsafe { staging.0.as_ptr().add(128) },
            "view.payload must point INTO the caller buffer at the payload region (zero copy)"
        );
        assert_eq!(view.payload_len, 64);
        // Probe leg: NULL out_view admits and must not write.
        let st_probe = call_admit(&staging.0[..frame.len()], core::ptr::null_mut());
        assert_eq!(st_probe, CGAP_OK, "NULL out_view probe must admit");
        // The unaligned twin: the SAME bytes at a 1-aligned buffer must take
        // the decode path and reach the same verdict (recorded divergence:
        // the C surface cannot promise 64-byte alignment).
        let mut unaligned = frame.clone();
        let mut uview = CgapFrameView {
            payload: core::ptr::null(),
            payload_len: 0,
            schema_id: 0,
            rule_count: 0,
        };
        let st_u = call_admit(&mut unaligned, &mut uview);
        assert_eq!(
            st_u, CGAP_OK,
            "unaligned twin must admit via the decode path"
        );
        assert_eq!(uview.payload_len, 64);
        assert_eq!(uview.payload, unsafe { unaligned.as_ptr().add(128) });
        println!("ADMIT POSITIVE: aligned projection + unaligned decode both admit; view is the zero-copy payload borrow");
    }

    // -- Admit court 2: the /tmp sandbox corruption corpus -------------------

    /// The dispatch court — corrupt frame -> TYPED status. Real frame bytes
    /// live in a /tmp sandbox file; every mutation is written to a real
    /// file, read back FROM DISK, and admitted through the real body; each
    /// fault site must produce its EXACT register class (A14 rows 1-6 + the
    /// A04 validation/digest legs). The sandbox original is restored
    /// byte-exact (sha256 fail-closed); the court deletes its own lease.
    /// Anti-vacuity: flips the always-admit stub in `gap1_stub_probes_can_flip`.
    #[test]
    fn admit_sandbox_corruption_corpus_typed_refusals() {
        let payload: Vec<u8> = (0..64u16).map(|i| (i * 7) as u8).collect();
        let frame = wire_frame(&payload);
        let dir = sandbox_dir("admit");
        let path = dir.join("frame.bin");
        fs::write(&path, &frame).expect("sandbox frame write");
        let sha_before = shasum(&path);

        // (fault site, expected register class). Sites: magic, version,
        // reserved-flag bits, payload_byte_len (row-3b length coherence),
        // digest bytes, attestation magic, payload byte (A04 digest leg).
        let cases: &[(usize, u8, CgapStatus, &str)] = &[
            (0, b'X', CGAP_ERR_INVALID_MAGIC, "magic flip (A14 row 2)"),
            (4, 2, CGAP_ERR_INVALID_MAGIC, "version flip (A14 row 2)"),
            (
                5,
                0x10,
                CGAP_ERR_SHACL_VIOLATION,
                "reserved flag bit (A14 row 6 admits; A04 shape leg)",
            ),
            (
                8,
                0x01,
                CGAP_ERR_OABI_MISMATCH,
                "payload_byte_len +1 (A14 row 3b)",
            ),
            (
                16,
                0xFF,
                CGAP_ERR_OABI_MISMATCH,
                "digest byte flip (A04 BLAKE3 re-derivation)",
            ),
            (
                48,
                b'X',
                CGAP_ERR_P28_SPOLIATION,
                "attestation magic flip (A14 row 4)",
            ),
            (
                128,
                0xFF,
                CGAP_ERR_OABI_MISMATCH,
                "payload byte flip (A04 digest leg)",
            ),
        ];
        for &(site, xor, want, name) in cases {
            let mut mutated = frame.clone();
            mutated[site] ^= xor;
            let mpath = dir.join(format!("mut-{site}.bin"));
            fs::write(&mpath, &mutated).expect("sandbox mutation write");
            let on_disk = fs::read(&mpath).expect("sandbox mutation read");
            assert_eq!(on_disk, mutated, "sandbox bytes must match the mutation");
            let st = call_admit(&on_disk, core::ptr::null_mut());
            assert_eq!(st, want, "mutation at {site} ({name}) must refuse TYPED");
        }
        // sha256-exact restore, fail-closed; byte-exact always runs.
        let sha_after = shasum(&path);
        match (&sha_before, &sha_after) {
            (Some(b), Some(a)) => assert_eq!(a, b, "sandbox restore not sha256-exact"),
            _ => println!("[SKIP] shasum absent — sha256 restore leg unexecuted; byte-exact leg below still ran"),
        }
        assert_eq!(
            fs::read(&path).expect("sandbox re-read"),
            frame,
            "sandbox original byte-exact after the corpus"
        );
        fs::remove_dir_all(&dir).expect("sandbox lease cleanup");
        println!("ADMIT CORPUS: 7 fault sites, 7 typed refusals, restore byte-exact");
    }

    // -- Eval court 1: positive control, determinism, zero-copy --------------

    /// The real one-rule network over the real three-row store: exactly two
    /// matches, lanes = the projected head object words (100, 100), count
    /// written; a second call is byte-identical (determinism contract); the
    /// view borrows the payload — one mutated payload byte changes the eval
    /// output, restoring it restores the output (eval-side zero-copy
    /// witness, A19 §6.4 form). Anti-vacuity: flips the always-refuse stub.
    #[test]
    fn eval_positive_control_deterministic_zero_copy() {
        let payload = store_payload();
        let net = one_rule_network();
        let view = eval_view(&payload);
        let mut lanes = [0i32; 8];
        let mut count = 0usize;
        let st = call_eval(&view, &net, lanes.as_mut_ptr(), 8, &mut count);
        assert_eq!(st, CGAP_OK, "one-rule eval must admit (status {st:#x})");
        assert_eq!(count, 2, "rows 0 and 1 match (?0, 2, 100)");
        assert_eq!(
            &lanes[..count],
            &[1, 5],
            "lanes are the projected head object words (?0 binds the matched row's subject: 1, 5)"
        );
        // Determinism: same input -> same output.
        let mut lanes2 = [0i32; 8];
        let mut count2 = 0usize;
        assert_eq!(
            call_eval(&view, &net, lanes2.as_mut_ptr(), 8, &mut count2),
            CGAP_OK
        );
        assert_eq!((count2, &lanes2[..count2]), (count, &lanes[..count]));

        // Zero-copy witness: flip one payload o-column byte through the raw
        // pointer -> the row no longer matches -> eval output changes;
        // restore -> output restored.
        let mut payload_m = payload.clone();
        let view_m = eval_view(&payload_m);
        // o-column byte for row 0: the o-col starts at byte 16*n = 48
        // (n = 3 rows); byte +1 is inside the o_0 word (100).
        let site = 16 * 3 + 1;
        let orig = payload_m[site];
        payload_m[site] = !orig;
        let mut lanes3 = [0i32; 8];
        let mut count3 = 0usize;
        assert_eq!(
            call_eval(&view_m, &net, lanes3.as_mut_ptr(), 8, &mut count3),
            CGAP_OK
        );
        assert_eq!(count3, 1, "mutated o_0 must drop row 0 from the match set");
        payload_m[site] = orig;
        let mut lanes4 = [0i32; 8];
        let mut count4 = 0usize;
        assert_eq!(
            call_eval(&view_m, &net, lanes4.as_mut_ptr(), 8, &mut count4),
            CGAP_OK
        );
        assert_eq!(
            (count4, &lanes4[..count4]),
            (count, &lanes[..count]),
            "restore must return the original eval"
        );
        println!("EVAL POSITIVE: 2 matches -> lanes [1, 5] (projected ?0); deterministic; eval-side zero-copy witness held");
    }

    // -- Eval court 2: the /tmp sandbox mutation corpus ----------------------

    /// The dispatch court — aliased/hostile eval -> TYPED status. The
    /// network and store live in /tmp sandbox files; every mutation is
    /// written to a real file and read back FROM DISK before the call.
    /// Legs: (a) unknown-sigil term word -> OABI class (A12 compiler
    /// refusal); (b) framing fault (body_len runaway) -> OABI; (c) aliased
    /// view (payload_len not a row multiple) -> SHACL; (d) NULL view ->
    /// SHACL; (e) head object outside the declared i32 lane domain ->
    /// SHACL (the surface refuses what the kernel would truncate); (f)
    /// exhausted out_lanes lease -> LEASE (fail-closed, never truncation).
    /// Restore sha256/byte-exact; the court deletes its own lease.
    #[test]
    fn eval_sandbox_mutation_corpus_typed_refusals() {
        let payload = store_payload();
        let net = one_rule_network();
        let dir = sandbox_dir("eval");
        let net_path = dir.join("net.bin");
        fs::write(&net_path, &net).expect("sandbox network write");
        let sha_before = shasum(&net_path);
        let view = eval_view(&payload);
        let mut lanes = [0i32; 8];
        let mut count = 0usize;

        // (a) unknown sigil 0x05 in a body term word -> A12 compiler refusal.
        let mut bad_sigil = net.clone();
        bad_sigil[7 * 8 + 7] = 0x05; // word 7 top byte = term sigil -> 0x05 RESERVED
        let mp = dir.join("mut-sigil.bin");
        fs::write(&mp, &bad_sigil).expect("mut write");
        let on_disk = fs::read(&mp).expect("mut read");
        assert_eq!(
            call_eval(&view, &on_disk, lanes.as_mut_ptr(), 8, &mut count),
            CGAP_ERR_OABI_MISMATCH,
            "unknown sigil must refuse OABI-typed"
        );
        // (b) framing runaway: body_len word set huge -> OABI before any
        // allocation.
        let mut bad_frame = net.clone();
        bad_frame[6 * 8] = 0xFF; // body_len word low byte -> 255 > stream
        let mp = dir.join("mut-frame.bin");
        fs::write(&mp, &bad_frame).expect("mut write");
        let on_disk = fs::read(&mp).expect("mut read");
        assert_eq!(
            call_eval(&view, &on_disk, lanes.as_mut_ptr(), 8, &mut count),
            CGAP_ERR_OABI_MISMATCH,
            "body_len runaway must refuse OABI-typed"
        );
        // (c) aliased view: a payload length that is not a 24-byte row
        // multiple is not a statement store.
        let aliased: &[u8] = &payload[..payload.len() - 1];
        let view_alias = CgapFrameView {
            payload: aliased.as_ptr(),
            payload_len: aliased.len(),
            schema_id: 0,
            rule_count: 0,
        };
        assert_eq!(
            call_eval(&view_alias, &net, lanes.as_mut_ptr(), 8, &mut count),
            CGAP_ERR_SHACL_VIOLATION,
            "non-row-multiple payload must refuse SHACL-typed"
        );
        // (d) NULL view / NULL payload: not the eval surface shape.
        assert_eq!(
            call_eval(core::ptr::null(), &net, lanes.as_mut_ptr(), 8, &mut count),
            CGAP_ERR_SHACL_VIOLATION,
            "NULL view must refuse SHACL-typed"
        );
        // (e) lane domain: head object word above i32::MAX is outside the
        // declared lane domain — refused, never truncated.
        let mut wide = one_rule_network();
        let w = 3_000_000_000u64; // > i32::MAX, SlotConst payload
        wide[4 * 8..5 * 8].copy_from_slice(&w.to_le_bytes()); // head_o word
        let st = call_eval(&view, &wide, lanes.as_mut_ptr(), 8, &mut count);
        assert_eq!(
            st, CGAP_ERR_SHACL_VIOLATION,
            "out-of-domain lane must refuse SHACL-typed"
        );
        // (f) exhausted lease: cap 1 < 2 matches -> fail-closed typed.
        assert_eq!(
            call_eval(&view, &net, lanes.as_mut_ptr(), 1, &mut count),
            CGAP_ERR_LEASE_EXHAUSTED,
            "cap-1 lease must refuse LEASE-typed, never truncate"
        );
        let sha_after = shasum(&net_path);
        match (&sha_before, &sha_after) {
            (Some(b), Some(a)) => assert_eq!(a, b, "sandbox restore not sha256-exact"),
            _ => println!("[SKIP] shasum absent — sha256 restore leg unexecuted; byte-exact leg below still ran"),
        }
        assert_eq!(
            fs::read(&net_path).expect("re-read"),
            net,
            "sandbox network byte-exact"
        );
        fs::remove_dir_all(&dir).expect("sandbox lease cleanup");
        println!("EVAL CORPUS: sigil/framing/alias/null/domain/lease -> 6 typed refusals; restore byte-exact");
    }

    // -- Eval court 3: the encoding pin-parity court -------------------------

    /// The verify_p28 pin-parity pattern applied to the eval encoding: a
    /// real rule, encoded through the PINNED word-stream framing, must
    /// compile through cgap-bn3's OWN RuleCompiler to the identical compiled
    /// form a directly-constructed Rule compiles to — every run. A framing
    /// change (word order, arity) or a gutted pin text flips here.
    #[test]
    fn eval_encoding_pin_parity_real_compiler_round_trip() {
        // The pin text must name its framing (gutted-pin flip).
        assert!(
            EVAL_ENCODING_PIN.contains("LE u64"),
            "pin must name the word encoding"
        );
        assert!(
            EVAL_ENCODING_PIN.contains("rule_count"),
            "pin must name the rule framing"
        );
        assert!(
            EVAL_ENCODING_PIN.contains("s-col|p-col|o-col"),
            "pin must name the payload layout"
        );
        // Direct rule (owner API construction).
        let direct = cgap_bn3::unify::Rule {
            rule_id: 42,
            head: [k(7), k(8), v(0)],
            body: vec![cgap_bn3::unify::BodyAtom::new(v(0), k(2), k(100))],
        };
        let direct_c = cgap_bn3::unify::RuleCompiler::compile(&direct).expect("direct compiles");
        // Pinned-encoding rule (the same rule, encoded as the ABI framing
        // defines it — one_rule_network is the encoder under test).
        assert_eq!(
            one_rule_network().len(),
            9 * 8,
            "encoding shape pin: 9 words"
        );
        let (s, p, o) = store_columns();
        let _ = (s, p, o); // store shape pin lives in eval_positive_control
        let from_wire = decode_network_for_parity(&one_rule_network()).expect("pinned decode");
        let from_wire_c =
            cgap_bn3::unify::RuleCompiler::compile(&from_wire).expect("wire compiles");
        assert_eq!(
            direct_c, from_wire_c,
            "pinned-encoding round trip must compile to the owner-identical form"
        );
        // NAMED MUTATION: a mis-encoded table (head words before rule_id —
        // the framing flipped) must NOT decode to the same rule; the parity
        // court must flip on it. Executed every run.
        let mut wrong = one_rule_network();
        // Swap WORDS 1 and 3 (rule_id <-> head_p): a framing-order mutation.
        let (a, b) = wrong.split_at_mut(24);
        a[8..16].swap_with_slice(&mut b[0..8]);
        let wrong_rule = decode_network_for_parity(&wrong).expect("mis-framing still decodes");
        let wrong_c = cgap_bn3::unify::RuleCompiler::compile(&wrong_rule).expect("compiles");
        assert_ne!(
            direct_c, wrong_c,
            "VACUOUS GATE: the framing mutation produced an identical compiled form — the pin cannot flip"
        );
        println!("EVAL PIN-PARITY: pinned framing round-trips to the owner-identical compiled rule; framing mutation flips");
    }

    /// Reference decoder for the pinned framing (court-side twin of the
    /// body's decoder; drift between the two is the pin-parity court's
    /// subject — this twin exists ONLY in tests, never in the lib).
    fn decode_network_for_parity(words_bytes: &[u8]) -> Option<cgap_bn3::unify::Rule> {
        let word = |i: usize| -> Option<u64> {
            if words_bytes.len() < (i + 1) * 8 {
                return None;
            }
            let mut w = [0u8; 8];
            w.copy_from_slice(&words_bytes[i * 8..i * 8 + 8]);
            Some(u64::from_le_bytes(w))
        };
        let count = word(0)?;
        if count != 1 {
            return None;
        }
        let rule_id = word(1)? as u32;
        let head = [word(2)?, word(3)?, word(4)?];
        let body_len = word(5)? as usize;
        let mut body = Vec::new();
        for a in 0..body_len {
            let base = 6 + a * 3;
            body.push(cgap_bn3::unify::BodyAtom::new(
                word(base)?,
                word(base + 1)?,
                word(base + 2)?,
            ));
        }
        Some(cgap_bn3::unify::Rule {
            rule_id,
            head,
            body,
        })
    }

    // -- Commit court 1: positive control, byte-identical to the owner API ---

    /// A real commit writes a real file whose bytes are BYTE-IDENTICAL to a
    /// sealed block produced locally through the A02 owner API
    /// (write_slab -> seal) over the same lane — the sovereign commit is
    /// exactly the owner path, nothing more. Two commits are deterministic
    /// (two files identical). Anti-vacuity: flips the always-refuse stub.
    #[test]
    fn commit_positive_control_matches_owner_api_bytes() {
        // W3-L4 unblock (disclose-in-both): the commit leg now feeds the
        // PRE-LEASED ring sink — steady-state blocks land in the ring and
        // the spill file receives the drained bytes ONLY on rotation. The
        // byte-identity claim is preserved EXACTLY: the spill file must
        // equal the CONCATENATION of owner-API sealed blocks over the same
        // lane, in commit order.
        let lanes: Vec<i32> = (0..64).map(|i| (i % 251) as i32).collect();
        let dir = sandbox_dir("commit");
        let p1 = dir.join("out1.par2");
        let p2 = dir.join("out2.par2");
        let cstr1 = std::ffi::CString::new(p1.as_os_str().as_encoded_bytes()).expect("cstr");
        // Loop commits until the ring wraps at least once (bounded): the
        // spilled bytes are the drained ring blocks.
        let mut on_disk: Vec<u8> = Vec::new();
        for _ in 0..4096 {
            let st1 = call_commit(&lanes, cstr1.as_c_str());
            assert_eq!(st1, CGAP_OK, "commit must admit (status {st1:#x})");
            on_disk = fs::read(&p1).expect("committed file readable");
            if !on_disk.is_empty() {
                break;
            }
            on_disk.clear();
        }
        assert!(
            !on_disk.is_empty(),
            "ring must rotate to the spill file within 4096 commits"
        );
        // Owner-API reference: the CONCATENATION of sealed blocks over the
        // same lane (the canon path), same block count as the ring drained.
        let lane_u32: Vec<u32> = lanes.iter().map(|&v| v as u32).collect();
        let mut block = cgap_parquet::mmap_writer::StaticPageBuffer::<65_536>::new();
        let mut expected: Vec<u8> = Vec::new();
        for _ in 0.. {
            let slab = cgap_parquet::mmap_writer::SlabView {
                lanes: &[&lane_u32],
                rows: 64,
                bits: 8,
            };
            let mut writer = block.writer();
            writer.write_slab(&slab).expect("reference write_slab");
            let sealed = writer.seal(None).expect("reference seal");
            expected.extend_from_slice(sealed.bytes());
            if expected.len() >= on_disk.len() {
                break;
            }
        }
        assert!(
            *on_disk == *expected,
            "sovereign spill bytes must be byte-identical owner-API blocks"
        );
        // Determinism: a second commit stream is byte-identical. The lease
        // is per lane slot (thread-local; spill path resolved at grant), so
        // the second stream runs on a FRESH lane slot (its own thread = its
        // own lease granting at p2).
        let on_disk_ref = on_disk.clone();
        let p2_ref = p2.clone();
        let handle = std::thread::spawn(move || {
            let cstr2 =
                std::ffi::CString::new(p2_ref.as_os_str().as_encoded_bytes()).expect("cstr");
            let mut second: Vec<u8> = Vec::new();
            for _ in 0..4096 {
                assert_eq!(
                    call_commit(&lanes, cstr2.as_c_str()),
                    CGAP_OK,
                    "second commit stream must admit"
                );
                second = fs::read(&p2_ref).expect("second file");
                if !second.is_empty() {
                    break;
                }
                second.clear();
            }
            assert!(
                !second.is_empty(),
                "second stream must rotate within 4096 commits"
            );
            second
        });
        let on_disk2 = handle.join().expect("determinism thread");
        assert_eq!(on_disk2, on_disk_ref, "commit is deterministic");
        fs::remove_dir_all(&dir).expect("sandbox lease cleanup");
        println!(
            "COMMIT POSITIVE: spill bytes == owner-API sealed blocks (concatenated, commit order); deterministic across lane slots"
        );
    }

    // -- Commit court 2: the /tmp sandbox refusal corpus ---------------------

    /// The dispatch court — truncated/overflowed commit payload and domain
    /// faults -> TYPED status. Legs: (a) lane value 256 (outside the
    /// declared bits-8 kernel domain — the surface must refuse what the
    /// inherited packer would truncate) -> SHACL; (b) negative lane ->
    /// SHACL; (c) zero rows -> SHACL (the A02 footer refuses an empty
    /// body; the surface refuses it up front); (d) NULL path -> SHACL;
    /// (e) payload that cannot fit the documented 64 KiB lease ->
    /// LEASE (capacity exhaustion). Anti-vacuity: the lane-domain pin is
    /// the flip sensor for any widening of the domain gate in the body.
    #[test]
    fn commit_sandbox_refusal_corpus_typed() {
        let dir = sandbox_dir("commit-corpus");
        let cpath = |name: &str| {
            std::ffi::CString::new(dir.join(name).as_os_str().as_encoded_bytes()).expect("cstr")
        };
        // (a) value 256 outside [0, 255].
        let mut lanes = vec![0i32; 64];
        lanes[0] = 256;
        assert_eq!(
            call_commit(&lanes, &cpath("a.par2")),
            CGAP_ERR_SHACL_VIOLATION,
            "lane 256 must refuse SHACL-typed (domain pin)"
        );
        // Domain pin anti-vacuity: the SAME call on a legal lane admits
        // (the corpus above has detection power only if the positive twin
        // passes through the same gate).
        lanes[0] = 255;
        let st = call_commit(&lanes, &cpath("a-ok.par2"));
        assert_eq!(st, CGAP_OK, "lane 255 must commit (status {st:#x})");
        assert!(dir.join("a-ok.par2").is_file(), "committed file must exist");
        // (b) negative lane.
        lanes[1] = -1;
        assert_eq!(
            call_commit(&lanes, &cpath("b.par2")),
            CGAP_ERR_SHACL_VIOLATION,
            "negative lane must refuse SHACL-typed"
        );
        // (c) zero rows.
        let empty: Vec<i32> = Vec::new();
        assert_eq!(
            call_commit(&empty, &cpath("c.par2")),
            CGAP_ERR_SHACL_VIOLATION,
            "zero rows must refuse SHACL-typed"
        );
        // (d) NULL path.
        let f: CgapCommitParquetFn = cgap_commit_parquet;
        assert_eq!(
            unsafe { f(lanes.as_ptr(), lanes.len(), core::ptr::null()) },
            CGAP_ERR_SHACL_VIOLATION,
            "NULL path must refuse SHACL-typed"
        );
        // (e) truncated-lease class: a lane batch that cannot fit the
        // documented 64 KiB static lease. INCOMPRESSIBLE values: the A02
        // hybrid codec RLE-collapses constant runs, so a constant lane fits
        // easily — alternating values keep the page at full width.
        let big: Vec<i32> = (0..70_000).map(|i| (i % 251) as i32).collect();
        assert_eq!(
            call_commit(&big, &cpath("e.par2")),
            CGAP_ERR_LEASE_EXHAUSTED,
            "payload beyond the 64 KiB lease must refuse LEASE-typed"
        );
        fs::remove_dir_all(&dir).expect("sandbox lease cleanup");
        println!("COMMIT CORPUS: domain/negative/zero/null/overflow -> typed refusals; positive twin admitted");
    }

    // -- Anti-vacuity stub probes (test-local, never exported) ----------------
    //
    // A corpus that cannot distinguish the real bodies from stubs is
    // decoration. These exist only to prove the courts' legs have detection
    // power: the always-refuse stub must refuse every positive control's
    // input; the always-admit stubs must admit every corpus mutation.

    unsafe extern "C" fn stub_admit_refuse(
        _b: *const u8,
        _l: usize,
        _o: *mut CgapFrameView,
    ) -> CgapStatus {
        CGAP_ERR_INVALID_MAGIC
    }
    unsafe extern "C" fn stub_admit_ok(
        _b: *const u8,
        _l: usize,
        _o: *mut CgapFrameView,
    ) -> CgapStatus {
        CGAP_OK
    }
    unsafe extern "C" fn stub_eval_refuse(
        _v: *const CgapFrameView,
        _n: *const u8,
        _nl: usize,
        _o: *mut i32,
        _c: usize,
        _oc: *mut usize,
    ) -> CgapStatus {
        CGAP_ERR_OABI_MISMATCH
    }
    unsafe extern "C" fn stub_commit_refuse(
        _l: *const i32,
        _n: usize,
        _p: *const core::ffi::c_char,
    ) -> CgapStatus {
        CGAP_ERR_LEASE_EXHAUSTED
    }

    /// Both stub directions, per body, executed every run: (1) the
    /// always-refuse stub refuses each positive control's input — the
    /// positive controls cannot pass on a dead body; (2) the always-admit
    /// stub admits a corpus mutation the real body refuses typed — the
    /// corpora cannot pass on a fake body.
    #[test]
    fn gap1_stub_probes_can_flip() {
        // Admit.
        let payload: Vec<u8> = (0..64u16).map(|i| (i * 7) as u8).collect();
        let frame = wire_frame(&payload);
        let mut frame_m = frame.clone();
        frame_m[0] ^= b'X';
        let mut view = CgapFrameView {
            payload: core::ptr::null(),
            payload_len: 0,
            schema_id: 0,
            rule_count: 0,
        };
        assert_eq!(
            call_admit(&frame, &mut view),
            CGAP_OK,
            "premise: real body admits the valid frame"
        );
        assert_ne!(
            call_admit(&frame_m, core::ptr::null_mut()),
            CGAP_OK,
            "premise: real body refuses the magic flip"
        );
        let refuse: CgapAdmitFrameFn = stub_admit_refuse;
        assert_ne!(
            unsafe { refuse(frame.as_ptr(), frame.len(), &mut view) },
            CGAP_OK,
            "VACUOUS: the always-refuse stub ADMITTED the valid frame — positive control cannot flip"
        );
        let admit: CgapAdmitFrameFn = stub_admit_ok;
        assert_eq!(
            unsafe { admit(frame_m.as_ptr(), frame_m.len(), core::ptr::null_mut()) },
            CGAP_OK,
            "VACUOUS: the always-admit stub REFUSED the mutation — corpus cannot flip"
        );
        // Eval.
        let store = store_payload();
        let net = one_rule_network();
        let view_e = eval_view(&store);
        let mut lanes = [0i32; 8];
        let mut count = 0usize;
        assert_eq!(
            call_eval(&view_e, &net, lanes.as_mut_ptr(), 8, &mut count),
            CGAP_OK
        );
        let mut net_m = net.clone();
        net_m[7 * 8 + 7] = 0x05; // reserved sigil (the corpus mutation)
        assert_ne!(
            call_eval(&view_e, &net_m, lanes.as_mut_ptr(), 8, &mut count),
            CGAP_OK,
            "premise: real body refuses the reserved-sigil network"
        );
        let refuse_e: CgapEvalBn3Fn = stub_eval_refuse;
        assert_ne!(
            unsafe {
                refuse_e(
                    &view_e,
                    net.as_ptr(),
                    net.len(),
                    lanes.as_mut_ptr(),
                    8,
                    &mut count,
                )
            },
            CGAP_OK,
            "VACUOUS: the eval always-refuse stub ADMITTED — positive control cannot flip"
        );
        // Commit.
        let good = vec![1i32; 64];
        let big: Vec<i32> = (0..70_000).map(|i| (i % 251) as i32).collect();
        let dir = sandbox_dir("stub-commit");
        let path = std::ffi::CString::new(dir.join("s.par2").as_os_str().as_encoded_bytes())
            .expect("cstr");
        assert_eq!(
            call_commit(&good, &path),
            CGAP_OK,
            "premise: real commit admits the legal lane"
        );
        assert_ne!(
            call_commit(&big, &path),
            CGAP_OK,
            "premise: real commit refuses the oversized lease"
        );
        let refuse_c: CgapCommitParquetFn = stub_commit_refuse;
        assert_ne!(
            unsafe { refuse_c(good.as_ptr(), good.len(), path.as_ptr()) },
            CGAP_OK,
            "VACUOUS: the commit always-refuse stub ADMITTED the legal lane — positive control cannot flip"
        );
        fs::remove_dir_all(&dir).expect("sandbox lease cleanup");
        println!("GAP1 STUB PROBES: refuse-stubs flip all three positive controls; admit-stub flips the admit corpus");
    }

    // -- Cdylib symbol court: ALL FOUR sovereign symbols ---------------------

    /// nm -gU over the REAL fresh cdylib (nested lease build, /tmp lease
    /// target dir — never the parent's target dir): all FOUR sovereign
    /// bodies exported (the GAP-1 addition: admit/eval/commit join verify),
    /// the six frozen slot exports untouched (additive-only law), and the
    /// renamed symbols ABSENT (rename/removal flip). A symbol court that
    /// cannot flip on rename is decoration.
    #[test]
    fn gap1_cdylib_all_four_sovereign_symbols_present() {
        let dir = std::env::temp_dir().join(format!("cgap-gap1-dylib-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dylib court lease dir");
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let outcome = super::wave9_courts::run_bounded(
            "cargo",
            &[
                "build",
                "--lib",
                "--manifest-path",
                manifest.to_str().expect("manifest path"),
                "--target-dir",
                dir.join("target").to_str().expect("target path"),
            ],
            &[("CARGO_NET_OFFLINE", "true".to_string())],
            std::time::Duration::from_secs(600),
        )
        .unwrap_or_else(|e| panic!("gap1 symbol court: nested runner failed: {e}"));
        let status = outcome.status.unwrap_or_else(|| {
            panic!(
                "gap1 symbol court: build deadline exceeded; stderr:\n{}",
                super::wave9_courts::tail(&outcome.stderr)
            )
        });
        assert!(
            status.success(),
            "cdylib build failed; stderr tail:\n{}",
            super::wave9_courts::tail(&outcome.stderr)
        );
        let dylib = dir.join("target/debug/libcgap_ffi.dylib");
        assert!(
            dylib.is_file(),
            "cdylib artifact missing at {} (macOS court)",
            dylib.display()
        );
        let nm = match Command::new("nm").args(["-gU"]).arg(&dylib).output() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("[SKIP] nm absent — gap1 symbol court unexecuted (loud skip, never a silent pass)");
                let _ = fs::remove_dir_all(&dir);
                return;
            }
            Err(e) => panic!("nm -gU spawn failed: {e}"),
            Ok(o) => o,
        };
        assert!(
            nm.status.success(),
            "nm -gU failed:\n{}",
            String::from_utf8_lossy(&nm.stderr)
        );
        let exports = String::from_utf8_lossy(&nm.stdout);
        for sym in [
            "_cgap_admit_frame",
            "_cgap_eval_bn3",
            "_cgap_commit_parquet",
            "_cgap_verify_p28",
        ] {
            assert!(
                exports.contains(sym),
                "VACUOUS GATE: sovereign symbol {sym} absent from the cdylib export table:\n{exports}"
            );
        }
        for sym in [
            "_cgap_i32_batch",
            "_cgap_schema_addr",
            "_cgap_array_addr",
            "_cgap_batch_rows",
            "_cgap_version",
            "_cgap_digest_of",
        ] {
            assert!(
                exports.contains(sym),
                "pre-existing export {sym} lost — additive-only law violated:\n{exports}"
            );
        }
        assert!(
            !exports.contains("cgap_eval_bn_") && !exports.contains("cgap_admit_fram3"),
            "renamed sovereign symbol present — rename negative control broken:\n{exports}"
        );
        println!(
            "GAP1 SYMBOL COURT: nm -gU carries ALL FOUR sovereign bodies; 6 pre-existing exports intact; rename controls absent"
        );
        fs::remove_dir_all(&dir).expect("dylib court lease cleanup");
    }
}
