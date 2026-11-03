//! Whole-media-loss DR court (RED-16 finding A2 / FIX-14 R-A2.1-R-A2.3).
//!
//! Remedies the red-16 hole: journal recovery existed at module level
//! (`atomic_commit.rs:262-359`), but no court exercised a REAL whole-site
//! loss: primary store directory destroyed, restore from an off-site
//! snapshot (base root + journal copy) through the real recovery path.
//!
//! Scenario map (R-A2.1/R-A2.2):
//!   C1  valid off-site snapshot        -> root re-derived, Clean
//!   C2a journal copy truncated mid-top-record -> last intact record
//!       applied, lost transition explicitly surfaced via slot verdicts
//!   C2b journal copy truncated to a torn first slot -> base root +
//!       typed RecoveryDegraded, recovered_seq None (nothing fabricated)
//!   C3  base+journal both lost / base short -> typed restore refusal,
//!       fail-closed, no fabricated root (DATA_LOSS posture)
//!   C4  single-byte corruption of the off-site journal copy -> typed
//!       RecoveryDegraded (loud), never a Clean verdict (anti-vacuity)
//!   C5  M3 trailer-digest mutation flips on the C4 corruption class
//!
//! Anti-vacuity law: C4 asserts the typed degraded result on a corrupted
//! copy; a recovery that accepted the corruption would fail this court.
//! All mutations land on /tmp sandbox copies only; the shared tree is
//! read-only to this lane except this new test file.

#![forbid(unsafe_code)]

mod common;

use cgap_storage::atomic_commit::*;
use common::{base_root, commit, new_payload, prior_payload, FileMedium};

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static UNIQUE: AtomicU64 = AtomicU64::new(0);

fn unique_root() -> PathBuf {
    let n = UNIQUE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("cgap-dr-court-{}-{n}", std::process::id()))
}

/// Typed refusal of the restore harness itself (fail-closed, court-side;
/// the recovery module never sees fabricated media bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestoreRefusal {
    /// Off-site copy lacks a full 48-byte base-root superblock.
    BaseRootMissingOrShort,
    /// Off-site copy lacks the journal artifact entirely (whole-site loss:
    /// R-A2.2 typed DATA_LOSS posture — refuse, never fabricate a root).
    JournalMissing,
}

/// A journal medium restored from an off-site snapshot directory holding
/// `base.bin` (48-byte superblock descriptor) and `journal.bin` (raw slot
/// bytes, exactly as snapshotted — including truncation and bit damage).
/// Real `std::fs` reads; no in-memory shortcuts.
#[derive(Debug)]
struct RestoredMedium {
    dir: PathBuf,
}

impl RestoredMedium {
    /// Open the restored site. Typed refusal when the off-site copy cannot
    /// yield a lawful recovery substrate (fail-closed: the caller must never
    /// fall back to a guessed root).
    fn open(dir: &Path) -> Result<Self, RestoreRefusal> {
        let base = dir.join("base.bin");
        let journal = dir.join("journal.bin");
        let base_len = std::fs::metadata(&base)
            .map(|m| m.len())
            .map_err(|_| RestoreRefusal::BaseRootMissingOrShort)?;
        if base_len != PAYLOAD_SIZE as u64 {
            return Err(RestoreRefusal::BaseRootMissingOrShort);
        }
        if !journal.is_file() {
            return Err(RestoreRefusal::JournalMissing);
        }
        Ok(Self {
            dir: dir.to_path_buf(),
        })
    }

    /// Raw snapshot bytes of the journal file (may be short/damaged).
    fn journal_bytes(&self) -> Vec<u8> {
        std::fs::read(self.dir.join("journal.bin")).expect("read restored journal")
    }

    /// S-04b lenient open: skip harness refusals entirely and let the
    /// KERNEL classifier (`recover_loss_classified`) decide — this is the
    /// whole point of the kernel-side DATA_LOSS arm. Refuses nothing;
    /// never fabricates media bytes either.
    fn open_lossy(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }
}

impl Medium for RestoredMedium {
    fn read_base_root(&self) -> NewRootDescriptor {
        let bytes = std::fs::read(self.dir.join("base.bin")).expect("read restored superblock");
        let mut b = [0u8; PAYLOAD_SIZE];
        b.copy_from_slice(&bytes);
        NewRootDescriptor::from_bytes(&b)
    }

    fn read_slot(&self, slot: usize) -> [u8; RECORD_SIZE] {
        assert!(slot < SLOT_COUNT, "slot index out of range");
        let raw = self.journal_bytes();
        let start = slot * RECORD_SIZE;
        let mut buf = [0u8; RECORD_SIZE];
        // Truncated tail: real snapshot bytes first, then 0x00 fill. Zero
        // fill (NOT 0xFF) so a truncated slot classifies Damaged, never the
        // erased-state carve-out that would read as healthy.
        let avail = raw.len().saturating_sub(start).min(RECORD_SIZE);
        if avail > 0 {
            buf[..avail].copy_from_slice(&raw[start..start + avail]);
        }
        buf
    }

    fn artifacts_fenced(&self) -> bool {
        true
    }

    // S-04b additive kernel-seam overrides: the restored site reports which
    // artifacts genuinely exist so the kernel-side classifier can run.
    fn journal_present(&self) -> bool {
        self.dir.join("journal.bin").is_file()
    }

    fn base_root_present(&self) -> bool {
        match std::fs::metadata(self.dir.join("base.bin")) {
            Ok(m) => m.len() == PAYLOAD_SIZE as u64,
            Err(_) => false,
        }
    }
}

impl Drop for RestoredMedium {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Build a real primary store: two fenced commits (seq 1 = `prior_payload`,
/// seq 2 = `new_payload`) over a real-file `FileMedium`. Returns the site
/// directory and the on-disk journal byte length (224 when both records are
/// whole).
fn build_primary_store() -> (FileMedium, PathBuf) {
    let mut m = FileMedium::new(base_root());
    commit(&mut m, 1, &prior_payload());
    commit(&mut m, 2, &new_payload());
    let dir = m.dir().to_path_buf();
    let journal_len = std::fs::metadata(dir.join("journal.bin"))
        .expect("journal file exists")
        .len();
    assert_eq!(
        journal_len,
        (2 * RECORD_SIZE) as u64,
        "two whole records on disk"
    );
    (m, dir)
}

/// Off-site snapshot: copy base root + journal bytes out of the primary
/// site (the realistic DR posture: only these two artifacts survive).
fn snapshot_offsite(site: &Path, offsite: &Path, journal_len: u64) {
    std::fs::create_dir_all(offsite).expect("create offsite dir");
    std::fs::copy(site.join("base.bin"), offsite.join("base.bin")).expect("snapshot base root");
    let src = std::fs::read(site.join("journal.bin")).expect("read journal for snapshot");
    let n = (journal_len as usize).min(src.len());
    std::fs::write(offsite.join("journal.bin"), &src[..n]).expect("snapshot journal copy");
}

/// C1: total media loss of the primary site; restore from a COMPLETE
/// off-site snapshot re-derives the committed root, Clean.
#[test]
fn c1_restore_from_whole_snapshot_rederives_root() {
    let (_m, site) = build_primary_store();
    let offsite = unique_root();
    snapshot_offsite(&site, &offsite, 2 * RECORD_SIZE as u64);
    std::fs::remove_dir_all(&site).expect("destroy primary site");
    assert!(
        !site.exists(),
        "whole-media-loss precondition: primary site gone"
    );
    let restored_dir = unique_root();
    std::fs::create_dir_all(&restored_dir).expect("create restored site");
    for f in ["base.bin", "journal.bin"] {
        std::fs::copy(offsite.join(f), restored_dir.join(f)).expect("restore from offsite copy");
    }
    let restored = RestoredMedium::open(&restored_dir).expect("off-site copy is lawful");
    let out = recover(&restored).expect("whole valid snapshot recovers Clean");
    assert_eq!(
        out.root,
        new_payload(),
        "committed root re-derived from journal"
    );
    assert_eq!(out.report, RecoveryReport::Clean);
    assert_eq!(out.recovered_seq, Some(2));
    assert_eq!(out.slot_verdicts, [SlotVerdict::Valid; SLOT_COUNT]);
}

/// C2a: journal snapshot truncated mid-top-record (seq 2 torn). The last
/// INTACT record (seq 1) is applied; the lost transition seq 2 is surfaced
/// explicitly via the slot verdicts — never a silently-partial healthy
/// read of the torn record.
#[test]
fn c2a_truncated_tail_applies_last_intact_and_names_the_loss() {
    let (_m, site) = build_primary_store();
    let offsite = unique_root();
    // 112 + 40 bytes: full seq-1 record + torn prefix of the seq-2 record.
    snapshot_offsite(&site, &offsite, RECORD_SIZE as u64 + 40);
    std::fs::remove_dir_all(&site).expect("destroy primary site");
    let restored_dir = unique_root();
    std::fs::create_dir_all(&restored_dir).expect("create restored site");
    for f in ["base.bin", "journal.bin"] {
        std::fs::copy(offsite.join(f), restored_dir.join(f)).expect("restore from offsite copy");
    }
    let restored = RestoredMedium::open(&restored_dir).expect("off-site copy is lawful");
    let out = recover(&restored).expect("intact record still recovers");
    assert_eq!(
        out.root,
        prior_payload(),
        "last INTACT record applied, not the torn one"
    );
    assert_eq!(out.recovered_seq, Some(1));
    assert_eq!(out.slot_verdicts[0], SlotVerdict::Valid);
    assert_eq!(
        out.slot_verdicts[1],
        SlotVerdict::Damaged,
        "lost seq-2 named"
    );
}

/// C2b: journal snapshot truncated inside the FIRST slot (40 bytes). No
/// valid candidate anywhere: base root stands, typed
/// [`RecoveryReport::RecoveryDegraded`], recovered_seq None — the fully
/// degraded verdict, loud, with both transitions lost.
#[test]
fn c2b_truncated_first_slot_degrades_to_base_root() {
    let (_m, site) = build_primary_store();
    let offsite = unique_root();
    snapshot_offsite(&site, &offsite, 40);
    std::fs::remove_dir_all(&site).expect("destroy primary site");
    let restored_dir = unique_root();
    std::fs::create_dir_all(&restored_dir).expect("create restored site");
    std::fs::copy(offsite.join("base.bin"), restored_dir.join("base.bin"))
        .expect("restore base root");
    std::fs::copy(
        offsite.join("journal.bin"),
        restored_dir.join("journal.bin"),
    )
    .expect("restore journal copy");
    let restored = RestoredMedium::open(&restored_dir).expect("off-site copy is lawful");
    let out = recover(&restored).expect("degraded recovery is still a typed Ok");
    assert_eq!(
        out.root,
        base_root(),
        "base root stands, nothing fabricated"
    );
    assert_eq!(out.report, RecoveryReport::RecoveryDegraded);
    assert_eq!(out.recovered_seq, None, "both transitions lost, explicitly");
    assert_eq!(out.slot_verdicts, [SlotVerdict::Damaged; SLOT_COUNT]);
}

/// C3: whole-site loss where the off-site copy itself is incomplete.
/// Restoring from a copy missing the journal artifact entirely must REFUSE
/// typed (R-A2.2 DATA_LOSS posture), and a short/missing base root must
/// refuse too — no fabricated root is ever handed to recovery.
#[test]
fn c3_incomplete_offsite_copy_refuses_typed() {
    let (m, site) = build_primary_store();
    let offsite = unique_root();
    snapshot_offsite(&site, &offsite, 2 * RECORD_SIZE as u64);
    let _ = m;

    // (a) journal artifact absent: typed refusal, no recovery substrate.
    let no_journal = unique_root();
    std::fs::create_dir_all(&no_journal).expect("create dir");
    std::fs::copy(offsite.join("base.bin"), no_journal.join("base.bin")).expect("base root only");
    assert!(
        matches!(
            RestoredMedium::open(&no_journal),
            Err(RestoreRefusal::JournalMissing)
        ),
        "journal lost entirely: typed DATA_LOSS refusal, fail-closed"
    );

    // (b) base root absent.
    let no_base = unique_root();
    std::fs::create_dir_all(&no_base).expect("create dir");
    std::fs::copy(offsite.join("journal.bin"), no_base.join("journal.bin")).expect("journal only");
    assert!(
        matches!(
            RestoredMedium::open(&no_base),
            Err(RestoreRefusal::BaseRootMissingOrShort)
        ),
        "base root lost: typed refusal, no fabricated root"
    );

    // (c) base root SHORT (truncated superblock copy): typed refusal.
    let short_base = unique_root();
    std::fs::create_dir_all(&short_base).expect("create dir");
    std::fs::copy(offsite.join("journal.bin"), short_base.join("journal.bin"))
        .expect("journal only");
    std::fs::write(short_base.join("base.bin"), [0u8; 16]).expect("short superblock");
    assert!(
        matches!(
            RestoredMedium::open(&short_base),
            Err(RestoreRefusal::BaseRootMissingOrShort)
        ),
        "short base copy refuses typed, never a guessed root"
    );
    for d in [&no_journal, &no_base, &short_base, &offsite] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// C4 (contract anti-vacuity): ONE byte of the off-site journal copy is
/// corrupted in a fresh tempdir. Recovery must degrade loudly — typed
/// [`RecoveryReport::RecoveryDegraded`] with the corrupted slot named —
/// never a Clean verdict on damaged media. A recovery that accepts the
/// corruption is FAIL.
#[test]
fn c4_single_byte_offsite_corruption_degrades_loudly() {
    let (_m, site) = build_primary_store();
    let offsite = unique_root();
    snapshot_offsite(&site, &offsite, RECORD_SIZE as u64); // seq-1 record only
    std::fs::remove_dir_all(&site).expect("destroy primary site");

    // Corrupt ONE byte of the record's payload region (CRC+BLAKE3-covered).
    let jpath = offsite.join("journal.bin");
    let mut bytes = std::fs::read(&jpath).expect("read offsite journal");
    let corrupt_at = HEADER_SIZE + OFF_PARQUET_ROOT; // payload byte 20..28
    bytes[corrupt_at] ^= 0x01;
    std::fs::write(&jpath, &bytes).expect("write corrupted copy");

    let restored_dir = unique_root();
    std::fs::create_dir_all(&restored_dir).expect("create restored site");
    std::fs::copy(offsite.join("base.bin"), restored_dir.join("base.bin")).expect("restore base");
    std::fs::copy(&jpath, restored_dir.join("journal.bin")).expect("restore corrupted journal");
    let restored = RestoredMedium::open(&restored_dir).expect("off-site copy is lawful");
    let out = recover(&restored).expect("damage is typed Ok (degraded), never a panic");
    assert_eq!(
        out.report,
        RecoveryReport::RecoveryDegraded,
        "corrupted off-site copy must degrade LOUDLY, never a Clean verdict"
    );
    assert_eq!(
        out.root,
        base_root(),
        "base root stands, corruption not applied"
    );
    assert_eq!(out.recovered_seq, None);
    assert_eq!(out.slot_verdicts[0], SlotVerdict::Damaged);
    for d in [&offsite, &restored_dir] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// C5 (validator-chain mutation flip, R-A2.3): corrupt a byte in the
/// BLAKE3 TRAILER half only (bytes 100..112 — the region ONLY the trailer
/// digest check covers). Under Strict the copy degrades loudly; under the
/// M3 mutation (`M3TrailerDigestSkipped`) the SAME corrupted copy decodes
/// Valid and recovers Clean — the court flips on the named single fault.
#[test]
fn c5_trailer_digest_mutation_flips_on_corrupted_copy() {
    let (_m, site) = build_primary_store();
    let offsite = unique_root();
    snapshot_offsite(&site, &offsite, RECORD_SIZE as u64);
    std::fs::remove_dir_all(&site).expect("destroy primary site");

    let jpath = offsite.join("journal.bin");
    let mut bytes = std::fs::read(&jpath).expect("read offsite journal");
    let corrupt_at = OFF_BLAKE3_TRAILER_ABS; // trailer digest half, byte 100
    bytes[corrupt_at] ^= 0x80;
    std::fs::write(&jpath, &bytes).expect("write corrupted copy");

    let restored_dir = unique_root();
    std::fs::create_dir_all(&restored_dir).expect("create restored site");
    std::fs::copy(offsite.join("base.bin"), restored_dir.join("base.bin")).expect("restore base");
    std::fs::copy(&jpath, restored_dir.join("journal.bin")).expect("restore corrupted journal");
    let restored = RestoredMedium::open(&restored_dir).expect("lawful off-site copy");

    // Strict (production chain): the trailer-only-corruption must degrade.
    let strict = recover(&restored).expect("damage is a typed degraded Ok");
    assert_eq!(strict.report, RecoveryReport::RecoveryDegraded);
    assert_eq!(strict.root, base_root());
    assert_eq!(strict.slot_verdicts[0], SlotVerdict::Damaged);

    // M3 mutation (trailer digest dropped): same bytes decode Valid and the
    // corrupted record ships as the recovered root — the court FLIPS.
    let mutated = recover_mode(&restored, MutationMode::M3TrailerDigestSkipped)
        .expect("mutated chain still returns");
    assert_eq!(mutated.report, RecoveryReport::Clean);
    assert_eq!(
        mutated.root,
        prior_payload(),
        "corrupted record applied under M3"
    );
    assert_eq!(mutated.recovered_seq, Some(1));
    for d in [&offsite, &restored_dir] {
        let _ = std::fs::remove_dir_all(d);
    }
}

// ---------------------------------------------------------- S-04b additions --

/// C6 (S-04b, R-A2.2 kernel-side arm): whole-site loss where the off-site
/// copy ITSELF lost the journal artifact. The kernel classifier returns the
/// typed DATA_LOSS arm with full evidence — no fabricated root, fail-closed.
#[test]
fn c6_journal_artifact_lost_kernel_side_data_loss() {
    let (m, site) = build_primary_store();
    let offsite = unique_root();
    snapshot_offsite(&site, &offsite, 2 * RECORD_SIZE as u64);
    let _ = m;

    // Off-site copy: base root survived, journal artifact gone entirely.
    let lost = unique_root();
    std::fs::create_dir_all(&lost).expect("create dir");
    std::fs::copy(offsite.join("base.bin"), lost.join("base.bin")).expect("base only");
    let restored = RestoredMedium::open_lossy(&lost);
    let verdict = recover_loss_classified(&restored).expect("DataLoss is typed Ok");
    match verdict {
        RecoveryVerdict::Recovered(_) => {
            panic!("journal artifact gone: kernel must NOT return a recovery outcome")
        }
        RecoveryVerdict::DataLoss(ev) => {
            assert_eq!(ev.class(), DataLossClass::JournalMissing);
            assert_eq!(
                ev.base_root(),
                Some(base_root()),
                "surviving base root reported, not fabricated further"
            );
            assert!(
                ev.slot_verdicts().is_none(),
                "no journal bytes: no slot evidence to fabricate"
            );
            assert_eq!(ev.top_journal_seq(), None);
            assert_eq!(
                ev.lost_seq_floor(),
                FIRST_SEQ,
                "every journal transition from seq 1 is unprovable"
            );
        }
    }
    for d in [&lost, &offsite] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// C7 (S-04b): base-root superblock lost, journal bytes survive. Typed
/// DATA_LOSS (the rollback floor is unprovable — SeqRegression could never
/// be checked), but the JOURNAL evidence is preserved in the verdict.
#[test]
fn c7_base_root_lost_kernel_side_data_loss_with_journal_evidence() {
    let (_m, site) = build_primary_store();
    let offsite = unique_root();
    snapshot_offsite(&site, &offsite, 2 * RECORD_SIZE as u64);
    let _ = offsite;
    let lost = unique_root();
    std::fs::create_dir_all(&lost).expect("create dir");
    std::fs::copy(site.join("journal.bin"), lost.join("journal.bin"))
        .expect("journal bytes survive");
    let restored = RestoredMedium::open_lossy(&lost);
    let verdict = recover_loss_classified(&restored).expect("DataLoss is typed Ok");
    match verdict {
        RecoveryVerdict::Recovered(_) => {
            panic!("base root gone: kernel must NOT return a recovery outcome")
        }
        RecoveryVerdict::DataLoss(ev) => {
            assert_eq!(ev.class(), DataLossClass::BaseRootMissing);
            assert_eq!(
                ev.base_root(),
                None,
                "lost superblock must not be fabricated"
            );
            assert_eq!(
                ev.slot_verdicts(),
                Some([SlotVerdict::Valid; SLOT_COUNT]),
                "journal bytes survive and decode"
            );
            assert_eq!(
                ev.top_journal_seq(),
                Some(2),
                "highest decodable seq named in the evidence"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&lost);
}

/// C8 (S-04b sharpness law): a PARTIAL truncation through the kernel-side
/// classifier must stay in the `Recovered` arm with
/// [`RecoveryReport::RecoveryDegraded`] — NOT DATA_LOSS. DATA_LOSS =
/// unrecoverable without an off-site copy; Degraded = recovered-with-losses.
#[test]
fn c8_partial_truncation_is_degraded_not_data_loss() {
    let (_m, site) = build_primary_store();
    let offsite = unique_root();
    // Truncation inside the first slot: damage, not absence.
    snapshot_offsite(&site, &offsite, 40);
    std::fs::remove_dir_all(&site).expect("destroy primary site");
    let restored_dir = unique_root();
    std::fs::create_dir_all(&restored_dir).expect("create restored site");
    std::fs::copy(offsite.join("base.bin"), restored_dir.join("base.bin")).expect("restore base");
    std::fs::copy(
        offsite.join("journal.bin"),
        restored_dir.join("journal.bin"),
    )
    .expect("restore truncated journal");
    let restored = RestoredMedium::open_lossy(&restored_dir);
    let verdict = recover_loss_classified(&restored).expect("degraded is a typed Ok");
    match verdict {
        RecoveryVerdict::DataLoss(ev) => {
            panic!(
                "truncation must never classify DATA_LOSS (got {ev:?}); Degraded/DATA_LOSS distinction is not sharp"
            )
        }
        RecoveryVerdict::Recovered(out) => {
            assert_eq!(out.root, base_root());
            assert_eq!(out.report, RecoveryReport::RecoveryDegraded);
            assert_eq!(out.recovered_seq, None);
            assert_eq!(out.slot_verdicts, [SlotVerdict::Damaged; SLOT_COUNT]);
        }
    }
    for d in [&offsite, &restored_dir] {
        let _ = std::fs::remove_dir_all(d);
    }
}

// End of media-loss DR court.
