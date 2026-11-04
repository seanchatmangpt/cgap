//! W2-11 wire-parity court (wave R2 item 11) — the A10 §5.3 siphon:
//! a spec-declared CIMI cloud event mapped through `tosca_cimi_v1` slots
//! onto a fixed 64-byte wire frame ("CEW1"), asserted byte-identical
//! across three independent derivations:
//!
//! 1. PYTHON MINTER (receipt anchor): `wire_mint.py` assembles the frames
//!    from the committed `src/lib.rs` golden table (no hand transcription);
//!    its output is the committed `GOLDEN_E1`/`GOLDEN_E2` hex below.
//! 2. FACADE path: only `cgap_cloud` public API (`slot_of`,
//!    `resolve_equivalence_lexeme`, `LEXEME_TABLE`, `ENUM_FAMILIES`).
//! 3. RAW path: parses `.oabi` bytes by offset (A06 §6), re-derives every
//!    record SlotId from its LexHash, resolves lexemes through real BLAKE3
//!    preimages, and validates record ClassId/EnumId fields against
//!    hand-restated spec constants — zero `cgap_cloud` items (the
//!    `raw_consumer.rs` discipline).
//!
//! GOLDEN PROVENANCE: the A10 lane spec declares NO wire golden vector
//! (§7 defines T1 pack digest, T2 slot goldens, T3 loader refusals, T4
//! equivalence, T5 C-ABI consumption — none pins an event-frame encoding).
//! Per the W2-11 dispatch the vectors below are NEWLY-MINTED from the
//! committed pack (derivation path 1), deterministically, and labelled as
//! such; the missing spec vector is recorded in
//! `docs/jira/v26.11.1/waves/r2-wire-parity.md`.
//!
//! Frame layout "CEW1" (CIMI Event Wire v1), 64 bytes, all integers LE:
//!
//! ```text
//! offset size field
//! 0      4    magic  = b"CEW1"
//! 4      2    version_major = 1
//! 6      2    class_id  (2 = EventStream)
//! 8      2    enum_family (3 = event.severity)
//! 10     2    severity_variant (INFO=0 / WARNING=1 / CRITICAL=2)
//! 12     4    reserved (zero)
//! 16     8    root_slot       (canonical root; alias resolved)
//! 24     8    content_slot    (cimi.Event.content)
//! 32     8    severity_slot   (cimi.Event.severity)
//! 40     8    timestamp_slot  (cimi.Event.timestamp)
//! 48     8    channel_slot    (eventstream.channel)
//! 56     8    presented_slot  (alias slot as presented; = root if none)
//! ```
//!
//! Anti-vacuity: `flip_slot_corruption_refuses_typed` and
//! `flip_class_field_corruption_refuses_typed` corrupt a /tmp sandbox
//! copy of the pack (byte-exact restore + mtime bump; the canonical
//! shared-tree file is witnessed untouched by digest AND mtime) and
//! prove the parity path refuses with the named typed error.
//! `golden_sensitivity` proves the goldens pin the variant and
//! presented-slot bits. A court that cannot flip is FAIL.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use cgap_blake3::hash;
use cgap_cloud::{
    ENUM_FAMILIES, LEXEME_TABLE, PACK_DIGEST_HEX, resolve_equivalence_lexeme, slot_of,
};

/// Canonical seed (W2-11 dispatch): sandbox dir suffix and the flip byte
/// offset are derived from it, deterministically.
const SEED: u64 = 0xBF58_476D_1CE4_E5B9;

/// The committed pack file (lane A10's on-disk artifact).
const PACK_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tosca_cimi_v1.oabi");

/// NEWLY-MINTED golden (provenance in module header): E1 — plain CIMI
/// event, presented root `cimi.Event`, severity WARNING.
const GOLDEN_E1: &str = "4345573101000200030001000000000037BD719B611F1EEF084E636A119313DF9DE358047C559350E60D17F9357AC358EF1543592580280837BD719B611F1EEF";

/// NEWLY-MINTED golden: E2 — job-completion event presented through the
/// `cimi.Job` alias; canonical root resolves to `cimi.Event`; severity
/// CRITICAL.
const GOLDEN_E2: &str = "4345573101000200030002000000000037BD719B611F1EEF084E636A119313DF9DE358047C559350E60D17F9357AC358EF15435925802808E152B44C718B61ED";

/// The six A10 §5.3 event lexemes with their expected RECORD fields
/// (ClassId, EnumId family), restated from the committed pack spec —
/// independent of `cgap_cloud` (raw_consumer discipline): if the facade
/// drifts, these constants still pin the pack.
const RAW_EVENT_LEXEMES: &[(&str, u32, u16)] = &[
    ("cimi.Event", 2, 0),
    ("cimi.Event.content", 2, 0),
    ("cimi.Event.severity", 2, 3),
    ("cimi.Event.timestamp", 2, 0),
    ("eventstream.channel", 2, 0),
    ("cimi.Job", 2, 0),
];

/// The TOSCA<->CIMI alias direction, restated from lane doc §5 (A10 §5.1-5.3
/// equivalence rows) — independent of `cgap_cloud::EQUIV_TABLE`.
const RAW_ALIAS: &[(&str, &str)] = &[
    ("cimi.Machine", "tosca.nodes.Compute"),
    ("cimi.Volume", "tosca.nodes.BlockStorage"),
    ("cimi.VolumeImage", "tosca.nodes.BlockStorage"),
    ("cimi.Job", "cimi.Event"),
];

/// Severity variants, restated from the pack's pinned subset (lane doc §5.3:
/// INFO/WARNING/CRITICAL, dense from 0).
const RAW_SEVERITY_VARIANTS: &[(&str, u16)] =
    &[("INFO", 0), ("WARNING", 1), ("CRITICAL", 2)];

/// Typed wire-parity refusal. Every variant names its subject; the parity
/// path refuses, it never silently substitutes.
#[derive(Debug, PartialEq, Eq)]
enum WireError {
    BadMagic,
    Truncated { len: usize },
    RecordSize { got: usize },
    /// A06 §6 per-record obligation: record SlotId != low64(LexHash).
    SlotDerivationMismatch { index: usize, record_slot: u64, derived_slot: u64 },
    LexemeNotInPack { lexeme: String },
    SpecUnknownLexeme { lexeme: String },
    /// Record ClassId != the restated spec expectation.
    ClassFieldMismatch { lexeme: String, record_class: u32, expected_class: u32 },
    /// Record EnumId family != the restated spec expectation.
    FamilyFieldMismatch { lexeme: String, record_family: u16, expected_family: u16 },
    /// Severity name outside the pinned variant subset.
    VariantUnknown { family: &'static str, name: String },
    /// `slot_of` sentinel 0 — presented lexeme has no committed slot.
    AliasUnresolved { lexeme: String },
}

/// BLAKE3 digest, uppercase hex (courts.rs T1 pattern).
fn digest_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(64);
    for b in hash(bytes) {
        s.push_str(&format!("{b:02X}"));
    }
    s
}

/// Decode a 128-char hex golden into the 64 frame bytes.
fn decode_hex64(s: &str) -> [u8; 64] {
    assert_eq!(s.len(), 128, "golden hex must be 128 chars");
    let mut out = [0u8; 64];
    for i in 0..64 {
        let hi = (s.as_bytes()[2 * i] as char).to_digit(16).expect("hex hi nibble") as u8;
        let lo =
            (s.as_bytes()[2 * i + 1] as char).to_digit(16).expect("hex lo nibble") as u8;
        out[i] = (hi << 4) | lo;
    }
    out
}

/// Hand A06 §4 derivation (raw path): BLAKE3(b"bont\x01" ++ lexeme).
fn lex_hash(lexeme: &str) -> [u8; 32] {
    let mut tagged = Vec::with_capacity(5 + lexeme.len());
    tagged.extend_from_slice(b"bont\x01");
    tagged.extend_from_slice(lexeme.as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(&hash(&tagged));
    out
}

/// Deterministic per-test sandbox dir under the system temp root.
fn sandbox_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("w211-wire-parity-{SEED:016X}-{name}"))
}

/// FACADE path: encode the event frame through `cgap_cloud`'s public API
/// only. `presented` is the root lexeme or an equivalence alias; `severity`
/// is a pinned variant name of the `event.severity` family.
fn facade_frame(presented: &'static str, severity: &'static str) -> Result<[u8; 64], WireError> {
    let presented_slot = slot_of(presented);
    if presented_slot == 0 {
        return Err(WireError::AliasUnresolved { lexeme: presented.to_string() });
    }
    // Alias -> canonical through the crate lookup (A10 §6); identity when
    // the presented lexeme is not an alias.
    let root_slot = resolve_equivalence_lexeme(presented).unwrap_or(presented_slot);
    let root_row = LEXEME_TABLE
        .iter()
        .find(|r| r.slot == root_slot)
        .ok_or_else(|| WireError::AliasUnresolved { lexeme: presented.to_string() })?;
    let sev_row = LEXEME_TABLE
        .iter()
        .find(|r| r.lexeme == "cimi.Event.severity")
        .expect("cimi.Event.severity is a committed row");
    let family = ENUM_FAMILIES
        .iter()
        .find(|f| f.id == sev_row.enum_family)
        .expect("severity family is a committed family");
    let variant = family
        .variants
        .iter()
        .find(|v| v.name == severity)
        .ok_or_else(|| WireError::VariantUnknown {
            family: family.name,
            name: severity.to_string(),
        })?;
    let content_slot = slot_of("cimi.Event.content");
    let severity_slot = sev_row.slot;
    let timestamp_slot = slot_of("cimi.Event.timestamp");
    let channel_slot = slot_of("eventstream.channel");
    assert_ne!(content_slot, 0, "committed row missing: cimi.Event.content");
    assert_ne!(timestamp_slot, 0, "committed row missing: cimi.Event.timestamp");
    assert_ne!(channel_slot, 0, "committed row missing: eventstream.channel");

    let mut fr = [0u8; 64];
    fr[0..4].copy_from_slice(b"CEW1");
    fr[4..6].copy_from_slice(&1u16.to_le_bytes());
    fr[6..8].copy_from_slice(&(root_row.class as u16).to_le_bytes());
    fr[8..10].copy_from_slice(&sev_row.enum_family.to_le_bytes());
    fr[10..12].copy_from_slice(&variant.value.to_le_bytes());
    fr[16..24].copy_from_slice(&root_slot.to_le_bytes());
    fr[24..32].copy_from_slice(&content_slot.to_le_bytes());
    fr[32..40].copy_from_slice(&severity_slot.to_le_bytes());
    fr[40..48].copy_from_slice(&timestamp_slot.to_le_bytes());
    fr[48..56].copy_from_slice(&channel_slot.to_le_bytes());
    fr[56..64].copy_from_slice(&presented_slot.to_le_bytes());
    Ok(fr)
}

/// RAW path: encode the same event frame from `.oabi` bytes alone (no
/// `cgap_cloud` items). Every record's SlotId is re-checked against its
/// LexHash (the corruption detector), every event lexeme is resolved
/// through a real BLAKE3 preimage, and the record ClassId/EnumId fields
/// are validated against the restated spec constants.
fn raw_frame(pack: &[u8], presented: &'static str, severity: &'static str) -> Result<[u8; 64], WireError> {
    if pack.len() < 64 {
        return Err(WireError::Truncated { len: pack.len() });
    }
    if pack[0..8] != *b"OABIv1\0\0" {
        return Err(WireError::BadMagic);
    }
    let record_size = u32::from_le_bytes(pack[32..36].try_into().expect("u32")) as usize;
    if record_size != 48 {
        return Err(WireError::RecordSize { got: record_size });
    }
    let slot_count = u32::from_le_bytes(pack[28..32].try_into().expect("u32")) as usize;
    if pack.len() != 64 + slot_count * 48 {
        return Err(WireError::Truncated { len: pack.len() });
    }

    // Index the records: (LexHash, SlotId, ClassId, EnumId family), with
    // the A06 §6 derivation obligation re-checked per record.
    let mut index: Vec<([u8; 32], u64, u32, u16)> = Vec::with_capacity(slot_count);
    for i in 0..slot_count {
        let base = 64 + i * 48;
        let record_slot = u64::from_le_bytes(pack[base..base + 8].try_into().expect("u64"));
        let class = u32::from_le_bytes(pack[base + 8..base + 12].try_into().expect("u32"));
        let family = u16::from_le_bytes(pack[base + 12..base + 14].try_into().expect("u16"));
        let mut lex = [0u8; 32];
        lex.copy_from_slice(&pack[base + 16..base + 48]);
        let derived_slot = u64::from_le_bytes(lex[0..8].try_into().expect("u64"));
        if record_slot != derived_slot {
            return Err(WireError::SlotDerivationMismatch {
                index: i,
                record_slot,
                derived_slot,
            });
        }
        index.push((lex, record_slot, class, family));
    }

    // Lexeme resolution through the real preimage; record fields validated
    // against the restated spec expectations.
    let resolve = |name: &'static str| -> Result<(u64, u32, u16), WireError> {
        let d = lex_hash(name);
        match index.iter().find(|(l, _, _, _)| *l == d) {
            Some((_, s, c, f)) => Ok((*s, *c, *f)),
            None => Err(WireError::LexemeNotInPack { lexeme: name.to_string() }),
        }
    };
    let expect = |name: &str| -> Result<(u32, u16), WireError> {
        match RAW_EVENT_LEXEMES.iter().find(|(l, _, _)| *l == name) {
            Some((_, c, f)) => Ok((*c, *f)),
            None => Err(WireError::SpecUnknownLexeme { lexeme: name.to_string() }),
        }
    };
    let check = |name: &'static str| -> Result<(u64, u32, u16), WireError> {
        let (want_c, want_f) = expect(name)?;
        let (slot, rec_c, rec_f) = resolve(name)?;
        if rec_c != want_c {
            return Err(WireError::ClassFieldMismatch {
                lexeme: name.to_string(),
                record_class: rec_c,
                expected_class: want_c,
            });
        }
        if rec_f != want_f {
            return Err(WireError::FamilyFieldMismatch {
                lexeme: name.to_string(),
                record_family: rec_f,
                expected_family: want_f,
            });
        }
        Ok((slot, rec_c, rec_f))
    };

    let _ = check(presented)?;
    let canonical_lexeme = match RAW_ALIAS.iter().find(|(a, _)| *a == presented) {
        Some((_, c)) => *c,
        None => presented,
    };
    let (root_slot, _, _) = check(canonical_lexeme)?;
    let (content_slot, _, _) = check("cimi.Event.content")?;
    let (severity_slot, _, severity_family) = check("cimi.Event.severity")?;
    let (timestamp_slot, _, _) = check("cimi.Event.timestamp")?;
    let (channel_slot, _, _) = check("eventstream.channel")?;

    let variant = match RAW_SEVERITY_VARIANTS.iter().find(|(n, _)| *n == severity) {
        Some((_, v)) => *v,
        None => {
            return Err(WireError::VariantUnknown {
                family: "event.severity",
                name: severity.to_string(),
            })
        }
    };

    let presented_slot = resolve(presented)?.0;
    let mut fr = [0u8; 64];
    fr[0..4].copy_from_slice(b"CEW1");
    fr[4..6].copy_from_slice(&1u16.to_le_bytes());
    fr[6..8].copy_from_slice(&2u16.to_le_bytes());
    fr[8..10].copy_from_slice(&severity_family.to_le_bytes());
    fr[10..12].copy_from_slice(&variant.to_le_bytes());
    fr[16..24].copy_from_slice(&root_slot.to_le_bytes());
    fr[24..32].copy_from_slice(&content_slot.to_le_bytes());
    fr[32..40].copy_from_slice(&severity_slot.to_le_bytes());
    fr[40..48].copy_from_slice(&timestamp_slot.to_le_bytes());
    fr[48..56].copy_from_slice(&channel_slot.to_le_bytes());
    fr[56..64].copy_from_slice(&presented_slot.to_le_bytes());
    Ok(fr)
}

/// The wire-parity court itself: for both spec-declared events, the
/// facade encoding, the raw-pack encoding, and the newly-minted golden
/// are byte-identical. Anti-vacuity lives in the two flip courts below
/// and in `golden_sensitivity`.
#[test]
fn wire_parity_facade_raw_golden() {
    let pack = fs::read(PACK_PATH).expect("committed pack readable");
    assert_eq!(
        digest_hex(&pack),
        PACK_DIGEST_HEX,
        "canonical pack is the committed bytes (T1 witness)"
    );

    for (presented, severity, golden_hex) in
        [("cimi.Event", "WARNING", GOLDEN_E1), ("cimi.Job", "CRITICAL", GOLDEN_E2)]
    {
        let golden = decode_hex64(golden_hex);
        let facade = facade_frame(presented, severity).expect("facade encode");
        let raw = raw_frame(&pack, presented, severity).expect("raw encode");
        assert_eq!(facade, raw, "facade/raw divergence: {presented}/{severity}");
        assert_eq!(facade, golden, "wire bytes != golden: {presented}/{severity}");
    }

    // Field witnesses on E2 (the alias-carrying frame): magic, version,
    // canonical class/family, variant, and the two slot fields the alias
    // semantics separate — canonical root vs presented alias.
    let g2 = decode_hex64(GOLDEN_E2);
    assert_eq!(&g2[0..4], b"CEW1", "magic");
    assert_eq!(u16::from_le_bytes(g2[4..6].try_into().expect("u16")), 1, "version");
    assert_eq!(u16::from_le_bytes(g2[6..8].try_into().expect("u16")), 2, "class");
    assert_eq!(u16::from_le_bytes(g2[8..10].try_into().expect("u16")), 3, "family");
    assert_eq!(u16::from_le_bytes(g2[10..12].try_into().expect("u16")), 2, "variant");
    assert_eq!(&g2[12..16], &[0u8; 4], "reserved zero");
    let root = u64::from_le_bytes(g2[16..24].try_into().expect("u64"));
    let presented = u64::from_le_bytes(g2[56..64].try_into().expect("u64"));
    assert_eq!(root, cgap_cloud::CIMI_EVENT_SLOT, "canonical root slot");
    assert_eq!(presented, cgap_cloud::CIMI_JOB_SLOT, "presented alias slot");
    assert_ne!(root, presented, "alias resolution must be visible on the wire");
}

/// Golden-sensitivity court: the goldens pin the variant bits and the
/// presented-slot bits — any other spec-legal event encoding MUST NOT
/// reproduce them, and an out-of-subset severity refuses typed.
#[test]
fn golden_sensitivity() {
    let e1 = decode_hex64(GOLDEN_E1);
    let e2 = decode_hex64(GOLDEN_E2);
    // Variant swaps flip E1.
    assert_ne!(facade_frame("cimi.Event", "INFO").expect("info"), e1, "variant bits unpinned");
    assert_ne!(
        facade_frame("cimi.Event", "CRITICAL").expect("critical"),
        e1,
        "variant bits unpinned"
    );
    // Presented-slot swap flips both goldens.
    assert_ne!(facade_frame("cimi.Job", "WARNING").expect("job/warn"), e1, "presented bits unpinned");
    assert_ne!(
        facade_frame("cimi.Event", "CRITICAL").expect("event/crit"),
        e2,
        "presented bits unpinned"
    );
    // Out-of-subset severity refuses typed (no silent substitution).
    match facade_frame("cimi.Event", "FATAL") {
        Err(WireError::VariantUnknown { family, name }) => {
            assert_eq!(family, "event.severity");
            assert_eq!(name, "FATAL");
        }
        other => panic!("expected typed VariantUnknown, got {other:?}"),
    }
}

/// Flip court A (anti-vacuity): corrupt ONE SlotId byte of the
/// `cimi.Event.content` record in a /tmp sandbox COPY of the pack (flip
/// byte chosen from the canonical seed) — the parity path reading the
/// mutated copy must refuse, typed `SlotDerivationMismatch` naming the
/// record index. Restore is byte-exact (digest-verified) with a mtime
/// bump; the shared-tree pack is witnessed untouched by digest AND mtime.
#[test]
fn flip_slot_corruption_refuses_typed() {
    let canonical = fs::read(PACK_PATH).expect("committed pack readable");
    let base_digest = digest_hex(&canonical);
    let canonical_mtime =
        fs::metadata(PACK_PATH).expect("canonical metadata").modified().expect("mtime");

    let dir = sandbox_dir("slot-flip");
    fs::create_dir_all(&dir).expect("sandbox dir");
    let copy_path = dir.join("tosca_cimi_v1.oabi");
    fs::write(&copy_path, &canonical).expect("sandbox copy");

    // Name the record before corrupting it: record 19 must BE
    // cimi.Event.content (slot re-derived through the real BLAKE3).
    let rec = 19usize;
    let base = 64 + rec * 48;
    let expected_slot = u64::from_le_bytes(lex_hash("cimi.Event.content")[0..8].try_into().expect("u64"));
    let record_slot = u64::from_le_bytes(canonical[base..base + 8].try_into().expect("u64"));
    assert_eq!(record_slot, expected_slot, "record {rec} is not cimi.Event.content");

    let mut mutated = canonical.clone();
    mutated[base + (SEED % 8) as usize] ^= 0x01;
    fs::write(&copy_path, &mutated).expect("write mutated copy");
    let on_disk = fs::read(&copy_path).expect("read mutated copy back");
    assert_ne!(digest_hex(&on_disk), base_digest, "sandbox mutation must change the copy");

    match raw_frame(&on_disk, "cimi.Event", "WARNING") {
        Err(WireError::SlotDerivationMismatch { index, record_slot, derived_slot }) => {
            assert_eq!(index, rec, "refusal must name the corrupted record");
            assert_ne!(record_slot, derived_slot);
        }
        other => panic!("expected typed SlotDerivationMismatch, got {other:?}"),
    }

    // Byte-exact restore + mtime bump (shim-court protocol).
    fs::write(&copy_path, &canonical).expect("restore");
    let restored = fs::read(&copy_path).expect("restored read");
    assert_eq!(digest_hex(&restored), base_digest, "restore must be byte-exact");
    let f = fs::OpenOptions::new().write(true).open(&copy_path).expect("reopen copy");
    f.set_modified(SystemTime::now()).expect("mtime bump");
    // The shared-tree pack was never touched: digest AND mtime equal.
    assert_eq!(digest_hex(&fs::read(PACK_PATH).expect("canonical reread")), base_digest);
    assert_eq!(
        fs::metadata(PACK_PATH).expect("canonical metadata 2").modified().expect("mtime 2"),
        canonical_mtime,
        "canonical pack mtime moved — shared-tree mutation"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Flip court B (anti-vacuity, distinguisher from the A06 loader's T3):
/// corrupt the ClassId field of the `cimi.Event.severity` record in a
/// /tmp sandbox copy (class 2 -> 3, in range, so slot derivation still
/// holds) — slot re-derivation CANNOT catch this; only the parity court's
/// record-field cross-check can. Refusal must be typed
/// `ClassFieldMismatch`. Same byte-exact restore + mtime protocol.
#[test]
fn flip_class_field_corruption_refuses_typed() {
    let canonical = fs::read(PACK_PATH).expect("committed pack readable");
    let base_digest = digest_hex(&canonical);
    let canonical_mtime =
        fs::metadata(PACK_PATH).expect("canonical metadata").modified().expect("mtime");

    let dir = sandbox_dir("class-flip");
    fs::create_dir_all(&dir).expect("sandbox dir");
    let copy_path = dir.join("tosca_cimi_v1.oabi");
    fs::write(&copy_path, &canonical).expect("sandbox copy");

    // Name the record before corrupting it: record 20 must BE
    // cimi.Event.severity with ClassId 2 / family 3.
    let rec = 20usize;
    let base = 64 + rec * 48;
    let expected_slot = u64::from_le_bytes(lex_hash("cimi.Event.severity")[0..8].try_into().expect("u64"));
    let record_slot = u64::from_le_bytes(canonical[base..base + 8].try_into().expect("u64"));
    assert_eq!(record_slot, expected_slot, "record {rec} is not cimi.Event.severity");
    let class = u32::from_le_bytes(canonical[base + 8..base + 12].try_into().expect("u32"));
    assert_eq!(class, 2, "severity record class must be EventStream");

    let mut mutated = canonical.clone();
    mutated[base + 8..base + 12].copy_from_slice(&3u32.to_le_bytes());
    fs::write(&copy_path, &mutated).expect("write mutated copy");
    let on_disk = fs::read(&copy_path).expect("read mutated copy back");
    assert_ne!(digest_hex(&on_disk), base_digest, "sandbox mutation must change the copy");
    // Derivation still holds on the mutated copy (slot bytes untouched) —
    // proving the flip exercises the FIELD cross-check, not T3's check.
    assert_eq!(
        u64::from_le_bytes(on_disk[base..base + 8].try_into().expect("u64")),
        u64::from_le_bytes(on_disk[base + 16..base + 24].try_into().expect("u64")),
        "slot derivation must still hold after a class-field flip"
    );

    match raw_frame(&on_disk, "cimi.Event", "WARNING") {
        Err(WireError::ClassFieldMismatch { lexeme, record_class: 3, expected_class: 2 }) => {
            assert_eq!(lexeme, "cimi.Event.severity");
        }
        other => panic!("expected typed ClassFieldMismatch, got {other:?}"),
    }

    // Byte-exact restore + mtime bump; shared tree witnessed untouched.
    fs::write(&copy_path, &canonical).expect("restore");
    let restored = fs::read(&copy_path).expect("restored read");
    assert_eq!(digest_hex(&restored), base_digest, "restore must be byte-exact");
    let f = fs::OpenOptions::new().write(true).open(&copy_path).expect("reopen copy");
    f.set_modified(SystemTime::now()).expect("mtime bump");
    assert_eq!(digest_hex(&fs::read(PACK_PATH).expect("canonical reread")), base_digest);
    assert_eq!(
        fs::metadata(PACK_PATH).expect("canonical metadata 2").modified().expect("mtime 2"),
        canonical_mtime,
        "canonical pack mtime moved — shared-tree mutation"
    );
    let _ = fs::remove_dir_all(&dir);
}
