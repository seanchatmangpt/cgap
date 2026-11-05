//! E-h finding-5 courts, cgap-local (arcus-side court NOT warranted — scope
//! evidence below). The arcus Go flight plane consumes ONLY
//! `cgap-core/src/frame.rs` (already pinned by arcus's
//! `aoq_core_frame_drift_test.go`); `grep -rn "dma_ring|isomorphism"
//! --include="*.go"` over the arcus tree returns ZERO hits, so there is no
//! arcus drift surface for these two modules. The real consumers are internal:
//! cgap-git's ingress/egress rings re-implement the dma_ring law by documented
//! reuse (§7.2 cache-line, §7.1 publish order, §8.3 read law), cgap-bn3's
//! edge-closure court depends on lib.rs exporting both modules, and
//! cgap-core's own identity court consumes `identity_assert`.
//!
//! Source-parse idiom (mirrors arcus `aoq_core_frame_drift_test.go`): the REAL
//! subject bytes enter the court via `include_str!` (exact bytes of the src
//! file at compile time); `pins()` evaluates every pinned invariant against a
//! source string and returns drift findings; the mutation legs run the
//! identical pin set against in-memory mutated copies and assert the court
//! FLIPS (a court that cannot flip is FAIL; mutation-free green = VACUOUS).

/// The real subjects: exact bytes at test-compile time.
const DMA_RING_SRC: &str = include_str!("../src/dma_ring.rs");
const ISOMORPHISM_SRC: &str = include_str!("../src/isomorphism.rs");
const LIB_SRC: &str = include_str!("../src/lib.rs");
/// Consumer-side anchor surface (documented law-reuse contracts).
const INGRESS_RING_SRC: &str = include_str!("../../cgap-git/src/ingress_ring.rs");

fn missing(findings: &mut Vec<String>, what: &str, pat: &str) {
    findings.push(format!("DRIFT {what}: pattern {pat:?} not found"));
}

fn require(findings: &mut Vec<String>, haystack: &str, what: &str, pat: &str) {
    if !haystack.contains(pat) {
        missing(findings, what, pat);
    }
}

// ---------------------------------------------------------------------
// dma_ring pins — the laws the internal consumers depend on
// ---------------------------------------------------------------------

fn dma_ring_pins(rs: &str, lib: &str, ingress: &str) -> Vec<String> {
    let mut f: Vec<String> = Vec::new();

    // §7.2 cache-line discipline (cgap-git ingress/egress reuse this law).
    require(
        &mut f,
        rs,
        "CACHE_LINE_BYTES=64",
        "pub const CACHE_LINE_BYTES: usize = 64",
    );
    require(
        &mut f,
        rs,
        "CachePaddedIndex align(64)",
        "#[repr(align(64))]",
    );
    require(
        &mut f,
        rs,
        "cache-line size const assert",
        "assert!(core::mem::size_of::<CachePaddedIndex>() == CACHE_LINE_BYTES)",
    );
    require(
        &mut f,
        rs,
        "cache-line align const assert",
        "assert!(core::mem::align_of::<CachePaddedIndex>() == CACHE_LINE_BYTES)",
    );

    // §7.1 ring layout: power-of-two capacity, mask indexing, head/tail atomics.
    require(
        &mut f,
        rs,
        "power-of-two capacity",
        "let capacity = 1usize << slot_pow2",
    );
    require(
        &mut f,
        rs,
        "producer mask indexing",
        "let idx = head & (self.capacity - 1); // §7.1 mask-indexed",
    );
    require(
        &mut f,
        rs,
        "consumer mask indexing",
        "let idx = tail & (self.capacity - 1);",
    );
    require(&mut f, rs, "head index atomic", "head: CachePaddedIndex");
    require(&mut f, rs, "tail index atomic", "tail: CachePaddedIndex");

    // §10 claim zero-fill law (no stale bytes at publish).
    require(
        &mut f,
        rs,
        "claim zero-fill",
        "slot.fill(0); // §10 padding law",
    );

    // §7.3 publish law: typed refusals + payload-length agreement.
    require(
        &mut f,
        rs,
        "Overflow refusal",
        "return Err(RingError::Overflow)",
    );
    require(&mut f, rs, "BadFrame refusal", "RingError::BadFrame");
    require(
        &mut f,
        rs,
        "publish payload-len gate",
        "if header.payload_byte_len != (len - SLOT_HEADER_LEN) as u64",
    );
    require(
        &mut f,
        rs,
        "publish decode gate",
        "crate::frame::CgapFrameHeader::decode(slot).map_err(|_| RingError::BadFrame)",
    );
    // §7.1 AcqRel publish order: bytes before index advance (Release).
    require(
        &mut f,
        rs,
        "publish Release ordering",
        "self.head.store(head.wrapping_add(1), Ordering::Release);",
    );

    // §8.3 read law: fail-closed, never garbage; widened u64 bound (A04 fix).
    require(
        &mut f,
        rs,
        "read empty -> None",
        "if head == tail {\n            return None;",
    );
    require(
        &mut f,
        rs,
        "read widened u64 payload bound",
        "if header.payload_byte_len > (self.slot_size - SLOT_HEADER_LEN) as u64",
    );

    // §7.3 slot layout: header span is the 128 B frame header.
    require(
        &mut f,
        rs,
        "slot header = frame header",
        "pub const SLOT_HEADER_LEN: usize = crate::frame::FRAME_HEADER_LEN;",
    );

    // ReleaseOnEmpty typed refusal.
    require(
        &mut f,
        rs,
        "ReleaseOnEmpty refusal",
        "RingError::ReleaseOnEmpty",
    );

    // Export surface (cgap-bn3 edge-closure court consumption: lib.rs exports
    // dma_ring / frame / isomorphism; tracking_alloc is #[path]-included).
    require(&mut f, lib, "lib.rs exports dma_ring", "pub mod dma_ring;");
    require(
        &mut f,
        lib,
        "lib.rs exports isomorphism",
        "pub mod isomorphism;",
    );

    // Consumer self-court: cgap-git ingress_ring must still declare its
    // documented dma_ring law-reuse contract.
    for (hay, what, pat) in [
        (
            ingress,
            "ingress single-accessor law",
            "single-accessor by construction (dma_ring law)",
        ),
        (ingress, "ingress §7.2 reuse", "dma_ring §7.2 reuse"),
        (
            ingress,
            "ingress §7.1 publish reuse",
            "Publish law (dma_ring §7.1 reuse)",
        ),
        (
            ingress,
            "ingress §8.3 read law reuse",
            "dma_ring §8.3 read law",
        ),
    ] {
        require(&mut f, hay, what, pat);
    }

    f
}

// ---------------------------------------------------------------------
// isomorphism pins — the mapping laws the identity court depends on
// ---------------------------------------------------------------------

fn isomorphism_pins(rs: &str) -> Vec<String> {
    let mut f: Vec<String> = Vec::new();

    // §8.1 tier surface: exactly four tiers, with the §8.2 fail-line tokens.
    require(&mut f, rs, "Tier::Wire variant", "pub enum Tier");
    for (what, pat) in [
        ("Wire variant", "    Wire,"),
        ("Memory variant", "    Memory,"),
        ("Arrow variant", "    Arrow,"),
        ("Parquet variant", "    Parquet,"),
    ] {
        require(&mut f, rs, what, pat);
    }
    for (tier, tok) in [
        ("wire", "\"wire\""),
        ("memory", "\"memory\""),
        ("arrow", "\"arrow\""),
        ("parquet", "\"parquet\""),
    ] {
        require(&mut f, rs, &format!("tier token {tier}"), tok);
    }

    // §8.2 fail-loud line, verbatim shape.
    require(
        &mut f,
        rs,
        "fail-line format",
        "IDENTITY_COURT_FAIL tier={} offset={} wire=0x{:02x} mem=0x{:02x} disk=0x{:02x}",
    );
    require(
        &mut f,
        rs,
        "TierMismatch report struct",
        "pub struct TierMismatch",
    );

    // §8.1 identity law: full-region value identity, addr-independent.
    require(&mut f, rs, "identity_assert entry", "pub fn identity_assert(wire: &[u8], tier: Tier, projection: &[u8]) -> Result<(), TierMismatch>");
    require(
        &mut f,
        rs,
        "length divergence gate",
        "if wire.len() != projection.len()",
    );
    require(
        &mut f,
        rs,
        "first-divergent-byte scan",
        "for (i, (w, p)) in wire.iter().zip(projection.iter()).enumerate()",
    );

    f
}

// ---------------------------------------------------------------------
// Courts + non-vacuity mutation legs
// ---------------------------------------------------------------------

#[test]
fn dma_ring_source_drift_court() {
    let findings = dma_ring_pins(DMA_RING_SRC, LIB_SRC, INGRESS_RING_SRC);
    assert!(
        findings.is_empty(),
        "cgap-core dma_ring ABI drift ({} findings):\n{}",
        findings.len(),
        findings.join("\n")
    );
}

#[test]
fn dma_ring_drift_court_mutation_flips() {
    let baseline = dma_ring_pins(DMA_RING_SRC, LIB_SRC, INGRESS_RING_SRC);
    assert!(
        baseline.is_empty(),
        "mutation leg precondition: unmutated source must pin clean, got:\n{}",
        baseline.join("\n")
    );
    // Each case mutates exactly one subject; the other two stay REAL so the
    // flip is attributable to the targeted pin, not to cross-subject noise.
    enum Subject {
        Ring,
        Lib,
    }
    let cases: [(&str, Subject, String); 6] = [
        (
            "CACHE_LINE_BYTES 64->32",
            Subject::Ring,
            DMA_RING_SRC.replacen(
                "pub const CACHE_LINE_BYTES: usize = 64",
                "pub const CACHE_LINE_BYTES: usize = 32",
                1,
            ),
        ),
        (
            "producer mask index -> modulo",
            Subject::Ring,
            DMA_RING_SRC.replacen(
                "let idx = head & (self.capacity - 1); // §7.1 mask-indexed",
                "let idx = head % self.capacity;",
                1,
            ),
        ),
        (
            "claim zero-fill deleted",
            Subject::Ring,
            DMA_RING_SRC.replacen("slot.fill(0); // §10 padding law", "", 1),
        ),
        (
            "publish Release -> Relaxed",
            Subject::Ring,
            DMA_RING_SRC.replacen(
                "self.head.store(head.wrapping_add(1), Ordering::Release);",
                "self.head.store(head.wrapping_add(1), Ordering::Relaxed);",
                1,
            ),
        ),
        (
            "read widened u64 bound -> truncated usize",
            Subject::Ring,
            DMA_RING_SRC.replacen(
                "if header.payload_byte_len > (self.slot_size - SLOT_HEADER_LEN) as u64",
                "if (header.payload_byte_len as usize) > self.slot_size - SLOT_HEADER_LEN",
                1,
            ),
        ),
        (
            "lib.rs export removed",
            Subject::Lib,
            LIB_SRC.replacen("pub mod dma_ring;", "mod dma_ring_inner;", 1),
        ),
    ];
    for (name, subject, mutated) in cases {
        let (ring, lib) = match subject {
            Subject::Ring => (mutated.as_str(), LIB_SRC),
            Subject::Lib => (DMA_RING_SRC, mutated.as_str()),
        };
        let findings = dma_ring_pins(ring, lib, INGRESS_RING_SRC);
        assert!(
            !findings.is_empty(),
            "VACUOUS COURT: mutation {name:?} did not flip any pin"
        );
    }
}

#[test]
fn isomorphism_source_drift_court() {
    let findings = isomorphism_pins(ISOMORPHISM_SRC);
    assert!(
        findings.is_empty(),
        "cgap-core isomorphism ABI drift ({} findings):\n{}",
        findings.len(),
        findings.join("\n")
    );
}

#[test]
fn isomorphism_drift_court_mutation_flips() {
    let baseline = isomorphism_pins(ISOMORPHISM_SRC);
    assert!(
        baseline.is_empty(),
        "mutation leg precondition: unmutated source must pin clean, got:\n{}",
        baseline.join("\n")
    );
    let cases: [(&str, String); 5] = [
        (
            "Memory tier removed",
            ISOMORPHISM_SRC.replacen("    Memory,\n", "", 1),
        ),
        (
            "tier token renamed",
            ISOMORPHISM_SRC.replacen("Tier::Memory => \"memory\"", "Tier::Memory => \"ram\"", 1),
        ),
        (
            "fail-line offset field dropped",
            ISOMORPHISM_SRC.replacen(
                "IDENTITY_COURT_FAIL tier={} offset={} wire=0x{:02x} mem=0x{:02x} disk=0x{:02x}",
                "IDENTITY_COURT_FAIL tier={} wire=0x{:02x} mem=0x{:02x} disk=0x{:02x}",
                1,
            ),
        ),
        (
            "length gate deleted",
            ISOMORPHISM_SRC.replacen("if wire.len() != projection.len()", "if false", 1),
        ),
        (
            "identity scan deleted",
            ISOMORPHISM_SRC.replacen(
                "for (i, (w, p)) in wire.iter().zip(projection.iter()).enumerate()",
                "for (i, (w, p)) in [].iter().zip([].iter()).enumerate()",
                1,
            ),
        ),
    ];
    for (name, mutated) in cases {
        let findings = isomorphism_pins(&mutated);
        assert!(
            !findings.is_empty(),
            "VACUOUS COURT: mutation {name:?} did not flip any pin"
        );
    }
}
