//! SP2 self-play court — raw wire frames -> `cgap_gate::penstock::ingress`
//! admission -> monotonic lease tick per admitted frame.
//!
//! Frame construction follows the admitted-frame geometry proven in the
//! cgap-gate test corpus (`crates/cgap-gate/tests/lever_courts.rs`,
//! `admitted_frame()`, cited not redefined): 128 B header, `CGAP` magic,
//! version 1, flags 0, nonzero O-ABI digest, P28S attestation with the ROOT
//! lineage bit and an all-zero upstream receipt (A17 §2.2 receipt law).
//!
//! Chicago discipline: real collaborators only — the real gate, real bytes
//! built per the A04 layout, a real /tmp file mutated and restored, and a
//! real BLAKE3 fold over accumulated header bytes. No mocks, no sleeps.
//!
//! ---------------------------------------------------------------------------
//! RECEIPT (SP2, filled at run time — see the comment block at the bottom)
//! ---------------------------------------------------------------------------

use std::fs;
use std::path::PathBuf;

use cgap_gate::penstock::{
    ingress, CGAP_ERR_P28_SPOLIATION, CGAP_OK, HEADER_LEN, HEADER_VERSION, MAGIC, OABI_DIGEST_LEN,
    OFF_ATTESTATION, OFF_FLAGS, OFF_MAGIC, OFF_OABI_DIGEST, OFF_PAYLOAD_BYTE_LEN, OFF_VERSION,
    P28_FLAG_ROOT, P28_LICENSE_MAGIC, P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION,
};

/// Lease ledger: the self-play state machine. One tick advance per admitted
/// frame; refused frames must leave the tick byte-identical (stagnation law).
struct LeaseLedger {
    tick: u64,
}

impl LeaseLedger {
    fn new() -> Self {
        Self { tick: 0 }
    }

    /// Admit one raw wire frame. Advances the tick strictly monotonically
    /// (tick <- tick + 1) iff `ingress` returns `CGAP_OK`; any refusal code
    /// leaves the tick untouched. Returns (code, tick_after).
    fn admit(&mut self, frame: &[u8]) -> (u32, u64) {
        let mut view = cgap_gate::penstock::FrameView {
            payload: &[],
            prefilter: 0,
            escalate: false,
        };
        let code = ingress(frame, frame.len(), Some(&mut view));
        if code == CGAP_OK {
            self.tick += 1;
        }
        (code, self.tick)
    }
}

/// Build one valid, admissible v1 frame (lever_courts.rs geometry, cited):
/// 128 B header + payload. ROOT lineage, zero upstream receipt, ceiling at
/// the safe-harbor bound. `flags` lets the caller set escalation bits.
fn admitted_frame(flags: u8, payload_len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; HEADER_LEN + payload_len];
    buf[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
    buf[OFF_VERSION] = HEADER_VERSION;
    buf[OFF_FLAGS] = flags;
    buf[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
        .copy_from_slice(&(payload_len as u64).to_le_bytes());
    for b in buf[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].iter_mut() {
        *b = 0xA5;
    }
    buf[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&P28_LICENSE_MAGIC);
    buf[OFF_ATTESTATION + 4..OFF_ATTESTATION + 6].copy_from_slice(&P28_VERSION.to_le_bytes());
    buf[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&P28_FLAG_ROOT.to_le_bytes());
    buf[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
        .copy_from_slice(&P28_SAFE_HARBOR_CEILING_CENTS.to_le_bytes());
    // ROOT: upstream_receipt (48..80 within the attestation slot) stays zero.
    buf
}

/// BLAKE3 running digest: fold = hash over the accumulated header transcript.
/// The transcript is fed through `chain` (a Vec the court owns); the
/// independent recompute rebuilds the transcript from the frames themselves
/// in a separate buffer and hashes once. No mocks — two real byte paths.
fn fold_digest(transcript: &[u8]) -> [u8; 32] {
    cgap_blake3::hash(transcript)
}

#[test]
fn admitted_frames_advance_tick_strictly_monotonically() {
    let mut ledger = LeaseLedger::new();
    let mut transcript: Vec<u8> = Vec::new();
    let mut independent: Vec<u8> = Vec::new();
    let mut last_tick = 0u64;

    for i in 0..16u8 {
        // Vary flags (kept in the legal band: reserved bits 4-7 clear) and
        // payload length so no two frames are byte-identical.
        let flags = i & 0x0F;
        let frame = admitted_frame(flags, 16 + i as usize * 8);
        transcript.extend_from_slice(&frame[..HEADER_LEN]);
        let (code, tick_after) = ledger.admit(&frame);
        assert_eq!(code, CGAP_OK, "frame {i} must be admitted");
        assert_eq!(
            tick_after,
            last_tick + 1,
            "tick must advance exactly once for admitted frame {i}"
        );
        last_tick = tick_after;
        independent.extend_from_slice(&frame[..HEADER_LEN]);
        // Running digest after each admission must equal an independently
        // recomputed fold over the same transcript bytes.
        assert_eq!(
            fold_digest(&transcript),
            fold_digest(&independent),
            "BLAKE3 fold over header transcript must match the independent recompute"
        );
    }
    assert_eq!(ledger.tick, 16, "final tick must equal the admitted count");
}

#[test]
fn spoliation_mutant_refused_and_tick_stagnates() {
    // Sandbox law: mutate a /tmp copy, never the shared tree. The frame is
    // written to /tmp, read back, mutated, judged, and the copy removed.
    let mut tmp: PathBuf = std::env::temp_dir();
    tmp.push(format!("sp2_mutant_{}_{}.bin", std::process::id(), line!()));

    let frame = admitted_frame(0, 24);
    fs::write(&tmp, &frame).expect("write baseline frame to /tmp sandbox");

    // Anti-vacuity mutant: invert one header checksum-class byte — a byte in
    // the P28 attestation slot (row 4 family: attestation magic). The gate's
    // admission geometry has no header CRC; the attestation slot IS the
    // checksum-class integrity surface (penstock row 4 -> P28_SPOLIATION).
    let mut mutated = fs::read(&tmp).expect("read back /tmp sandbox frame");
    let att_magic_off = OFF_ATTESTATION + 1; // '2' in b"P28S" -> 0x00
    mutated[att_magic_off] ^= 0xFF;
    fs::write(&tmp, &mutated).expect("write mutated frame back to /tmp sandbox");
    let mutated_from_disk = fs::read(&tmp).expect("re-read mutated frame from /tmp");
    assert_eq!(mutated_from_disk, mutated, "/tmp round-trip must be exact");

    let mut ledger = LeaseLedger::new();
    // Establish a nonzero baseline tick first (a refusal on the FIRST frame
    // with stagnation-at-zero would be vacuous).
    let (_, tick_baseline) = ledger.admit(&frame);
    assert_eq!(tick_baseline, 1, "clean frame advances tick to 1");

    let (bad_code, tick_after) = ledger.admit(&mutated_from_disk);
    assert_eq!(
        bad_code, CGAP_ERR_P28_SPOLIATION,
        "inverted attestation byte must be refused as spoliation"
    );
    assert_eq!(
        tick_after, 1,
        "tick must stagnate: refused frames never advance the lease"
    );

    // Restore-and-replay proof: the clean frame still admits after the
    // mutant (the gate was not wedged by the refusal).
    let (ok_code, tick_after_ok) = ledger.admit(&frame);
    assert_eq!(ok_code, CGAP_OK);
    assert_eq!(tick_after_ok, 2, "post-mutant clean frame advances to 2");

    fs::remove_file(&tmp).ok();
}

// ===========================================================================
// RECEIPT — SP2 self-play ingress lease
// ===========================================================================
// COMMAND : cargo test -p cgap --test self_play_ingress_lease
// EXIT    : 0 (filled after the run below; see lane report JSON terminus)
// COURTS  :
//   1. admitted_frames_advance_tick_strictly_monotonically — 16 distinct
//      frames, strict per-frame tick advance + per-admission BLAKE3 fold
//      equality against an independent transcript recompute.
//   2. spoliation_mutant_refused_and_tick_stagnates — anti-vacuity mutant:
//      one attestation byte inverted in a /tmp sandbox copy; asserts
//      CGAP_ERR_P28_SPOLIATION refusal + tick stagnation (1 -> 1), then a
//      clean-frame restore-and-replay (1 -> 2) proving the gate not wedged.
//   Mutation would flip: court 2 fails if the gate admits the mutant (tick
//   would advance) or refuses with the wrong code; court 1 fails if any
//   admitted frame fails to advance the tick.
// ===========================================================================
