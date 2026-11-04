// Layout courts (AB-6): the landed ontology is the spec.
//
// 1. Struct court: size/align/offset pins of AoqHeader against the
//    generated layout (compile-time pins live in header.rs; these are
//    the runtime re-asserts so a court failure names the field).
// 2. Drift court: parse the LANDED ontology TTL and assert its field
//    table equals the generated layout byte-for-byte. The graph is the
//    spec; a drift between graph and code is a failing court, and the
//    compile-time court then refuses the struct. Missing ontology is a
//    loud FAIL, never a silent pass.

use cgap_aoq::generated_aoq_header_layout as gen;
use cgap_aoq::header::{AoqHeader, DIGEST_SPAN, MAGIC, SIZE_BYTES, VERSION};
use std::mem::{align_of, offset_of, size_of};

/// Canonical landed ontology path (absolute; the graph is the spec).
const ONTOLOGY_PATH: &str = "/Users/sac/ggen-marketplace/packs/aoq-pack/ontology.ttl";

#[test]
fn struct_pins_match_generated_layout() {
    assert_eq!(size_of::<AoqHeader>(), SIZE_BYTES, "AoqHeader size");
    assert_eq!(size_of::<AoqHeader>(), 64, "AOQ1 envelope is 64 bytes");
    assert_eq!(align_of::<AoqHeader>(), 8, "AoqHeader align (u64 key_id)");
    assert_eq!(offset_of!(AoqHeader, magic), gen::AOQ1_MAGIC_OFFSET);
    assert_eq!(offset_of!(AoqHeader, version), gen::AOQ1_VERSION_OFFSET);
    assert_eq!(offset_of!(AoqHeader, epoch), gen::AOQ1_EPOCH_OFFSET);
    assert_eq!(offset_of!(AoqHeader, key_id), gen::AOQ1_KEY_ID_OFFSET);
    assert_eq!(offset_of!(AoqHeader, frame_ordinal), gen::AOQ1_FRAME_ORDINAL_OFFSET);
    assert_eq!(offset_of!(AoqHeader, seq), gen::AOQ1_SEQ_OFFSET);
    assert_eq!(offset_of!(AoqHeader, reserved), gen::AOQ1_RESERVED_OFFSET);
    assert_eq!(offset_of!(AoqHeader, header_digest), gen::AOQ1_HEADER_DIGEST_OFFSET);
}

#[test]
fn digest_span_is_half_open_0_3c() {
    assert_eq!(DIGEST_SPAN, (0x00, 0x3C));
    assert_eq!(gen::AOQ1_DIGEST_SPAN_START, 0x00);
    assert_eq!(gen::AOQ1_DIGEST_SPAN_END, 0x3C);
}

#[test]
fn magic_and_version_pins() {
    assert_eq!(MAGIC, 0x414F_5131);
    assert_eq!(VERSION, 2);
    assert_eq!(gen::AOQ1_MAGIC, 0x414F_5131);
    assert_eq!(gen::AOQ1_VERSION, 2);
}

// ------------------------------------------------------- drift court
// Minimal deterministic Turtle projection reader: extracts the
// aoq:HeaderField individuals (offset / sizeBytes / wireKind) in document
// order. No RDF library: the landed graph is a frozen, canonical file and
// the court reads the exact landed bytes.

#[derive(Debug, Clone, PartialEq, Eq)]
struct OntoField {
    offset: usize,
    size_bytes: usize,
    wire_kind: String,
}

fn parse_header_fields(ttl: &str) -> Vec<OntoField> {
    let mut fields = Vec::new();
    for block in ttl.split("a aoq:HeaderField ;") {
        if !block.contains("aoq:fieldOf aoq:header-aq1") {
            continue; // only real HeaderField blocks link aoq:fieldOf
        }
        let body = &block[..block.find("prov:wasDerivedFrom").unwrap_or(block.len())];
        let grab = |key: &str| -> Option<usize> {
            let idx = body.find(key)?;
            let tail = &body[idx + key.len()..];
            let digits: String = tail
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            digits.parse::<usize>().ok()
        };
        let offset = grab("aoq:offset").expect("HeaderField block without aoq:offset");
        let size_bytes = grab("aoq:sizeBytes").expect("HeaderField block without aoq:sizeBytes");
        let wk_idx = body.find("aoq:wireKind \"").expect("block without wireKind");
        let tail = &body[wk_idx + "aoq:wireKind \"".len()..];
        let wire_kind =
            tail[..tail.find('"').expect("unterminated wireKind string")].to_string();
        fields.push(OntoField { offset, size_bytes, wire_kind });
    }
    fields.sort_by_key(|f| f.offset);
    fields
}

#[test]
fn ontology_field_table_matches_generated_layout() {
    let ttl = std::fs::read_to_string(ONTOLOGY_PATH)
        .expect("landed ontology MISSING — the graph is the spec; a skip is not lawful");
    let onto_fields = parse_header_fields(&ttl);
    assert_eq!(
        onto_fields.len(),
        gen::HEADER_FIELDS.len(),
        "field count: ontology vs generated layout",
    );
    for (onto, genf) in onto_fields.iter().zip(gen::HEADER_FIELDS.iter()) {
        assert_eq!(onto.offset, genf.offset, "offset drift at {}", genf.rust_name);
        assert_eq!(
            onto.size_bytes, genf.size_bytes,
            "size drift at {}",
            genf.rust_name,
        );
        let want = match genf.wire_kind {
            gen::WireKind::U32 => "u32",
            gen::WireKind::U16 => "u16",
            gen::WireKind::U64 => "u64",
            gen::WireKind::ReservedZero => "reserved-zero",
            gen::WireKind::Fnv1a32 => "fnv1a32",
        };
        assert_eq!(onto.wire_kind, want, "wireKind drift at {}", genf.rust_name);
    }
}

#[test]
fn ontology_contiguity_chain_first_0_last_end_64() {
    let ttl = std::fs::read_to_string(ONTOLOGY_PATH)
        .expect("landed ontology MISSING — the graph is the spec; a skip is not lawful");
    let fields = parse_header_fields(&ttl);
    assert_eq!(fields.first().expect("no fields").offset, 0);
    let last = fields.last().expect("no fields");
    assert_eq!(last.offset + last.size_bytes, 64);
    for w in fields.windows(2) {
        assert_eq!(
            w[0].offset + w[0].size_bytes,
            w[1].offset,
            "contiguity break between offsets {} and {}",
            w[0].offset,
            w[1].offset,
        );
    }
}

// Non-vacuity: the drift court must be able to FAIL. We prove the parser
// and the comparator detect a real mutation by injecting one into an
// in-memory TTL copy (never the landed file).
#[test]
fn drift_court_flips_on_injected_mutation() {
    let ttl = std::fs::read_to_string(ONTOLOGY_PATH)
        .expect("landed ontology MISSING");
    // Mutation: seq offset 20 -> 21. The comparator must see it.
    let mutated = ttl.replace("aoq:offset 20 ;", "aoq:offset 21 ;");
    assert!(mutated != ttl, "mutation did not apply (vacuous self-check)");
    let fields = parse_header_fields(&mutated);
    let seq = fields.iter().find(|f| f.offset == 21).expect("mutated seq field");
    assert_eq!(seq.size_bytes, 4);
    let gen_seq = gen::HEADER_FIELDS.iter().find(|f| f.rust_name == "SEQ").unwrap();
    assert_ne!(
        seq.offset, gen_seq.offset,
        "drift court accepted a mutated offset — court is vacuous",
    );
}
