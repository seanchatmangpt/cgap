//! Host-only C-ABI session surface over [`cgap_aoq::session`] (W2-A,
//! wave-2-membrane-plan §1 Lane W2-A).
//!
//! The XIP (`target_os = "none"`) staticlib must stay allocator-free, so
//! this entire module — and its `cgap-aoq`/`cgap-crypto` host-gated manifest
//! deps — is compiled out there; the module declaration in `lib.rs` carries
//! the same `not(target_os = "none")` gate as the slot exports.
//!
//! Laws enacted here:
//! - **Handle table, not raw pointers.** Session state lives behind `u64`
//!   handles in a mutex-guarded table; handles are monotone and never
//!   reused, and a freed handle is tombstoned so a stale handle and a
//!   never-existed handle are refused identically (no existence oracle).
//! - **Closed-set status words, never -1.** Every export returns a word
//!   from the `aoq/status_maps.rs` frozen band; `OK` on success. The band
//!   is narrower than the state machine's taxonomy, so the collisions are
//!   pinned in one mapping (`status_of`), not guessed at call sites.
//! - **No panics across the boundary.** Every export body runs under
//!   `catch_unwind`; a caught unwind returns `AOQ_STATUS_BUSY` (the pinned
//!   internal-fault word) and never propagates.
//! - **KemSeam = the real ML-KEM adapter (AB-5).** The host seam wraps
//!   `cgap_crypto::kem` (FIPS 203 final) with a Shake256 KDF — the seam
//!   shape the corridor join court proved (AF1-KD-1 closed). Deterministic
//!   encapsulation seed derived from the peer ek (no RNG at the boundary).
//! - **No mutation on refusal (session law 4).** Guaranteed by the engine;
//!   the FFI court asserts it across the boundary via
//!   `cgap_aoq_session_fingerprint`.

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use cgap_aoq::error::AoqError;
use cgap_aoq::session::{
    self as sess, KemSeam, Session, SessionError, CT_BYTES, EK_BYTES, SS_BYTES,
};
use cgap_crypto::fips202::Shake256;

use crate::aoq::status_maps::{
    AOQ_STATUS_BUSY, AOQ_STATUS_DIGEST_MISMATCH, AOQ_STATUS_EPOCH_MISMATCH,
    AOQ_STATUS_EPOCH_RETIRED, AOQ_STATUS_INCOMPLETE_FRAME, AOQ_STATUS_KEY_UNKNOWN, AOQ_STATUS_OK,
};

// ---------------------------------------------------------------------------
// Host KemSeam: the real ML-KEM-768 adapter (AB-5), corridor-join shape
// ---------------------------------------------------------------------------

/// Order-sensitive 32-byte Shake256 squeeze over concatenated parts.
fn shake32(parts: &[&[u8]]) -> [u8; 32] {
    let mut sh = Shake256::instance();
    for p in parts {
        sh.absorb(p);
    }
    let mut out = [0u8; 32];
    sh.squeeze_into(&mut out);
    out
}

/// The real ML-KEM-768 host seam (AB-5 rebound; the corridor's TestKem
/// fallback retired). Zero-sized, so a `&'static` reference is free.
struct HostKem;

impl KemSeam for HostKem {
    fn encapsulate(
        &self,
        ek: &[u8; EK_BYTES],
    ) -> Result<([u8; CT_BYTES], [u8; SS_BYTES]), SessionError> {
        let m = shake32(&[b"w2a/mlkem/m", ek]);
        let mut ct = [0u8; CT_BYTES];
        let mut ss = [0u8; SS_BYTES];
        cgap_crypto::kem::encaps(ek, &m, &mut ct, &mut ss);
        Ok((ct, ss))
    }

    fn decapsulate(&self, dk: &[u8], ct: &[u8; CT_BYTES]) -> Result<[u8; SS_BYTES], SessionError> {
        let dk: &[u8; 2400] = dk
            .try_into()
            .map_err(|_| SessionError::KemRefused { op: 1 })?;
        Ok(cgap_crypto::kem::decaps(dk, ct))
    }

    fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
        shake32(parts)
    }
}

/// The process-lifetime seam reference the sessions borrow.
static HOST_KEM: HostKem = HostKem;

// ---------------------------------------------------------------------------
// Status mapping: SessionError -> frozen band (pinned collision disclosure)
// ---------------------------------------------------------------------------

/// Map a typed session refusal to one word of the frozen 8-word band.
///
/// The band (8 words) is narrower than the state machine's taxonomy; the
/// collisions are pinned here and asserted by the court, never guessed at
/// call sites. The full typed taxonomy remains reachable in Rust
/// (`cgap_aoq::session::SessionError`); the C-ABI band is the wire-visible
/// projection. `PENDING` is reserved (never returned by this surface).
const fn status_of(err: &SessionError) -> u32 {
    match err {
        // Truncated / out-of-order frame law.
        SessionError::LengthMismatch { .. } => AOQ_STATUS_INCOMPLETE_FRAME,
        // Frame-integrity law (magic/version/digest/kind tag/ct-echo bind).
        // KD-1 (docs/jira/v26.11.4/KNOWN_DIVERGENCES.md): UnsupportedVersion
        // collides onto the digest word because the frozen band has no
        // version-specific status; closes at the next band bump.
        SessionError::Wire(AoqError::BadMagic { .. })
        | SessionError::Wire(AoqError::UnsupportedVersion { .. })
        | SessionError::Wire(AoqError::DigestMismatch { .. })
        // AoqError::AeadMismatch appended mid-wave by lane CG8 (0x00_0D):
        // authenticity failure -> same frozen digest-mismatch status.
        | SessionError::Wire(AoqError::AeadMismatch)
        | SessionError::Wire(AoqError::ReservedNotZero)
        | SessionError::UnknownFrameKind { .. }
        | SessionError::TranscriptMismatch => AOQ_STATUS_DIGEST_MISMATCH,
        // Epoch / transition-table law.
        SessionError::Wire(AoqError::UnknownEpochWire { .. })
        | SessionError::EpochMismatch { .. }
        | SessionError::UnexpectedFrame { .. } => AOQ_STATUS_EPOCH_MISMATCH,
        // KEM / table identity law (also serves stale/unknown handles).
        SessionError::Wire(AoqError::KemKernelNotAdmitted { .. })
        | SessionError::Wire(AoqError::AeadKernelNotAdmitted { .. })
        | SessionError::KemRefused { .. }
        // AEAD open refused: an authenticity failure (P2 M1 typed landing).
        | SessionError::AeadMismatch
        // Confirm-tag law (v3): a close-frame confirm tag that fails
        // verification is an authenticity failure like any other.
        | SessionError::ConfirmMismatch => AOQ_STATUS_DIGEST_MISMATCH,
        // Counted budget exhausted; a rekey is required first.
        SessionError::RekeyRequired { .. } => AOQ_STATUS_BUSY,
        // Terminal session; nothing is lawful (retired = read-only).
        SessionError::SessionClosed => AOQ_STATUS_EPOCH_RETIRED,
        // Wire cursor law (truncated envelope, regressed ordinal/seq).
        SessionError::Wire(AoqError::HeaderTooShort { .. })
        | SessionError::Wire(AoqError::BufferTooShort { .. })
        | SessionError::Wire(AoqError::SlabFull { .. })
        | SessionError::Wire(AoqError::FrameOrdinalRegression { .. })
        | SessionError::Wire(AoqError::SeqGap { .. })
        | SessionError::OrdinalMismatch { .. } => AOQ_STATUS_INCOMPLETE_FRAME,
    }
}

// ---------------------------------------------------------------------------
// Handle table
// -------------------------------------------------------------------

/// One live session entry.
struct SessionEntry {
    engine: Session<'static, HostKem>,
}

/// The guarded handle table. Handles are monotone (never reused); freed
/// handles are tombstoned so stale and never-existed refuse identically.
struct SessionTable {
    live: BTreeMap<u64, SessionEntry>,
    tombstones: BTreeSet<u64>,
    next: u64,
}

static SESSION_TABLE: Mutex<SessionTable> = Mutex::new(SessionTable {
    live: BTreeMap::new(),
    tombstones: BTreeSet::new(),
    next: 1,
});

/// Run `f` with the table lock held. Lock poisoning cannot launder a broken
/// invariant past the boundary: the guarded mutations are plain heap-node
/// insert/remove (structurally intact even mid-unwind), so the poison is
/// unwrapped and the next caller observes a consistent table.
fn with_table<T>(f: impl FnOnce(&mut SessionTable) -> T) -> T {
    let mut guard = match SESSION_TABLE.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    f(&mut guard)
}

/// Which refusal a missing handle maps to. Stale and never-existed are
/// deliberately indistinguishable across the boundary (no existence oracle):
/// both return `KEY_UNKNOWN` — the identity is absent from the active table.
const STALE_HANDLE_STATUS: u32 = AOQ_STATUS_KEY_UNKNOWN;

// ---------------------------------------------------------------------------
// Thread-local last-error word (stats.rs poll law: the caller pulls, the
// kernel never sleeps)
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERROR: std::cell::Cell<u32> = const { std::cell::Cell::new(AOQ_STATUS_OK) };
}

fn set_last_error(word: u32) {
    LAST_ERROR.with(|c| c.set(word));
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

/// Create a session in `HandshakeInit` at (epoch, key_id, ordinal).
///
/// Returns the new monotone handle (never 0), or 0 on internal fault
/// (caught unwind; last-error carries `BUSY`).
#[no_mangle]
pub extern "C" fn cgap_aoq_session_new(epoch: u16, key_id: u64, ordinal: u32) -> u64 {
    let r = catch_unwind(AssertUnwindSafe(|| {
        let engine = Session::new(&HOST_KEM, epoch, key_id, ordinal);
        with_table(|t| {
            let h = t.next;
            t.next = t.next.wrapping_add(1);
            t.live.insert(h, SessionEntry { engine });
            t.tombstones.remove(&h);
            h
        })
    }));
    match r {
        Ok(h) => {
            set_last_error(AOQ_STATUS_OK);
            h
        }
        Err(_) => {
            set_last_error(AOQ_STATUS_BUSY);
            0
        }
    }
}

/// Advance the session by one wire frame ([`sess::parse_frame`] +
/// [`sess::Session::step`]).
///
/// # Safety
/// `frame_ptr` must point to `frame_len` readable bytes, or be null with
/// `frame_len == 0`.
#[no_mangle]
pub unsafe extern "C" fn cgap_aoq_session_step(
    handle: u64,
    frame_ptr: *const u8,
    frame_len: usize,
) -> u32 {
    let r = catch_unwind(AssertUnwindSafe(|| -> u32 {
        // Null/zero guards BEFORE any slice formation.
        if frame_len == 0 || frame_ptr.is_null() {
            let w = AOQ_STATUS_INCOMPLETE_FRAME;
            set_last_error(w);
            return w;
        }
        let bytes = unsafe { std::slice::from_raw_parts(frame_ptr, frame_len) };
        with_table(|t| {
            if !t.live.contains_key(&handle) {
                let w = STALE_HANDLE_STATUS;
                set_last_error(w);
                return w;
            }
            let parsed = match sess::parse_frame(bytes) {
                Ok(f) => f,
                Err(err) => {
                    let w = status_of(&err);
                    set_last_error(w);
                    return w;
                }
            };
            match t
                .live
                .get_mut(&handle)
                .expect("checked above")
                .engine
                .step(&parsed)
            {
                Ok(()) => {
                    let w = AOQ_STATUS_OK;
                    set_last_error(w);
                    w
                }
                Err(err) => {
                    let w = status_of(&err);
                    set_last_error(w);
                    w
                }
            }
        })
    }));
    match r {
        Ok(w) => w,
        Err(_) => {
            let w = AOQ_STATUS_BUSY;
            set_last_error(w);
            w
        }
    }
}

/// Emit a verified application frame (Established sessions only): the
/// engine writes `[AOQ1 header 64B][kind 2][seq BE 4B][payload][tag 4B]`
/// into `out` and `out_len` receives the written length.
///
/// Deviation from the plan sketch (disclosed in the W2-A receipt): `seq` is
/// an explicit parameter — the engine's `emit_verified` law requires a
/// caller-supplied sequence and the plan's 6-arg sketch left it implicit.
///
/// # Safety
/// `body_ptr/body_len` must describe readable bytes (may be null/0 for an
/// empty payload); `out_ptr/out_cap` a writable region; `out_len` a valid
/// `usize` cell.
#[no_mangle]
pub unsafe extern "C" fn cgap_aoq_session_emit_verified(
    handle: u64,
    seq: u32,
    body_ptr: *const u8,
    body_len: usize,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
) -> u32 {
    let r = catch_unwind(AssertUnwindSafe(|| -> u32 {
        if body_len > 0 && body_ptr.is_null() {
            let w = AOQ_STATUS_INCOMPLETE_FRAME;
            set_last_error(w);
            return w;
        }
        if out_ptr.is_null() || out_len.is_null() {
            let w = AOQ_STATUS_INCOMPLETE_FRAME;
            set_last_error(w);
            return w;
        }
        let payload = if body_len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(body_ptr, body_len) }
        };
        let out = unsafe { std::slice::from_raw_parts_mut(out_ptr, out_cap) };
        with_table(|t| {
            let Some(entry) = t.live.get_mut(&handle) else {
                let w = STALE_HANDLE_STATUS;
                set_last_error(w);
                return w;
            };
            match entry.engine.emit_verified(seq, payload, out) {
                Ok(written) => {
                    unsafe { *out_len = written };
                    let w = AOQ_STATUS_OK;
                    set_last_error(w);
                    w
                }
                Err(err) => {
                    let w = status_of(&err);
                    set_last_error(w);
                    w
                }
            }
        })
    }));
    match r {
        Ok(w) => w,
        Err(_) => {
            let w = AOQ_STATUS_BUSY;
            set_last_error(w);
            w
        }
    }
}

/// Write the session's canonical 32-byte fingerprint into `out32` — the
/// no-mutation-on-refusal witness read across the boundary (session law 4).
///
/// # Safety
/// `out32` must point to 32 writable bytes.
#[no_mangle]
pub unsafe extern "C" fn cgap_aoq_session_fingerprint(handle: u64, out32: *mut u8) -> u32 {
    let r = catch_unwind(AssertUnwindSafe(|| -> u32 {
        if out32.is_null() {
            let w = AOQ_STATUS_INCOMPLETE_FRAME;
            set_last_error(w);
            return w;
        }
        let out = unsafe { std::slice::from_raw_parts_mut(out32, 32) };
        with_table(|t| {
            let Some(entry) = t.live.get(&handle) else {
                let w = STALE_HANDLE_STATUS;
                set_last_error(w);
                return w;
            };
            out.copy_from_slice(&entry.engine.fingerprint());
            let w = AOQ_STATUS_OK;
            set_last_error(w);
            w
        })
    }));
    match r {
        Ok(w) => w,
        Err(_) => {
            let w = AOQ_STATUS_BUSY;
            set_last_error(w);
            w
        }
    }
}

/// Free a session. Stale (already-freed or never-existed) handles refuse
/// with [`STALE_HANDLE_STATUS`] and leave the table untouched.
///
/// # Safety
/// `handle` must not be used after this call returns.
#[no_mangle]
pub unsafe extern "C" fn cgap_aoq_session_free(handle: u64) -> u32 {
    let r = catch_unwind(AssertUnwindSafe(|| -> u32 {
        with_table(|t| {
            if t.live.remove(&handle).is_some() {
                t.tombstones.insert(handle);
                let w = AOQ_STATUS_OK;
                set_last_error(w);
                w
            } else {
                let w = STALE_HANDLE_STATUS;
                set_last_error(w);
                w
            }
        })
    }));
    match r {
        Ok(w) => w,
        Err(_) => {
            let w = AOQ_STATUS_BUSY;
            set_last_error(w);
            w
        }
    }
}

/// Last status word recorded on this thread (pull-based; the kernel never
/// sleeps). Fresh threads read `OK` until the first export runs.
#[no_mangle]
pub extern "C" fn cgap_aoq_last_error() -> u32 {
    LAST_ERROR.with(|c| c.get())
}

// ---------------------------------------------------------------------------
// Courts (in-module): status-mapping closed-set law
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aoq::status_maps::{aoq_status_name, AOQ_STATUS_COUNT};

    const BAND: [u32; AOQ_STATUS_COUNT] = [
        AOQ_STATUS_OK,
        AOQ_STATUS_BUSY,
        AOQ_STATUS_DIGEST_MISMATCH,
        AOQ_STATUS_EPOCH_MISMATCH,
        AOQ_STATUS_EPOCH_RETIRED,
        AOQ_STATUS_INCOMPLETE_FRAME,
        AOQ_STATUS_KEY_UNKNOWN,
        crate::aoq::status_maps::AOQ_STATUS_PENDING,
    ];

    #[test]
    fn every_mapped_status_is_in_the_frozen_band() {
        for w in BAND {
            assert!(aoq_status_name(w).is_some(), "{w:#x} outside band");
        }
        assert_eq!(BAND.len(), AOQ_STATUS_COUNT);
    }

    #[test]
    fn stale_and_missing_handles_refuse_identically() {
        assert_eq!(STALE_HANDLE_STATUS, AOQ_STATUS_KEY_UNKNOWN);
    }
}
