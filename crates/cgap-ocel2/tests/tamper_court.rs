//! TAMPER COURT (the lane falsifier): tamper-one-link -> projection
//! refuses. Chicago: real chain, real sealed blocks, real license,
//! real BLAKE3; assert final state; no mocks; no sleeps.
//!
//! Anti-vacuity witness (this session, /tmp sandbox): removing the
//! verify gate in `Ocel2Projector::project` makes `tamper_refusal`
//! FAIL (projection accepts corrupted input) — see the lane receipt.
use cgap_blake3::hash;
use cgap_ocel2::{
    FileIdentity, LeaseWitness, Ocel2Projector, Ocel2Refusal,
};
use cgap_p28::attestation::{
    P28LicenseAttestation, ATTESTATION_VERSION, FLAG_ROOT, LICENSE_MAGIC,
};
use cgap_parquet::footer::SealedBlock;
use cgap_parquet::mmap_writer::{SlabView, StaticPageBuffer};

fn root_license() -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: 1_000_000_000,
        origin_authority_digest: [7u8; 32],
        upstream_receipt: [0u8; 32],
    }
}

/// n REAL sealed blocks + a real chain appending each.
fn fixture(
    n: usize,
) -> (
    cgap_ffi::receipt_chain::RingReceiptChain,
    Vec<&'static SealedBlock<'static>>,
) {
    use cgap_ffi::receipt_chain::RingReceiptChain;
    let mut chain = RingReceiptChain::new();
    let mut blocks: Vec<&'static SealedBlock<'static>> = Vec::new();
    for i in 0..n {
        let buf: &'static mut StaticPageBuffer<65_536> =
            Box::leak(Box::new(StaticPageBuffer::<65_536>::new()));
        let lane_data: [u32; 4] = [
            i as u32, ((i + 1) % 256) as u32, ((i + 2) % 256) as u32,
            ((i + 3) % 256) as u32,
        ];
        let mut writer = buf.writer();
        writer
            .write_slab(&SlabView { lanes: &[&lane_data[..]], rows: 4, bits: 8 })
            .expect("fixture slab writes");
        let sealed: &'static SealedBlock<'static> =
            Box::leak(Box::new(writer.seal(None).expect("fixture seal")));
        chain.append(sealed, &root_license()).expect("fixture append");
        blocks.push(sealed);
    }
    (chain, blocks)
}

fn witnesses(n: usize) -> Vec<LeaseWitness> {
    (0..n)
        .map(|i| LeaseWitness {
            lease_id: 1000 + i as u64,
            file: if i == 0 {
                Some(FileIdentity { dev: 0x1000_0002, ino: 42 })
            } else {
                None
            },
        })
        .collect()
}

/// BASELINE (non-vacuity): the PRISTINE chain projects; every table
/// has the expected shape. An always-refusing projection is a VACUOUS
/// GATE, not a pass.
#[test]
fn pristine_projection_baseline() {
    let (chain, blocks) = fixture(3);
    let ws = witnesses(3);
    let t = Ocel2Projector::project(&chain, &blocks, &root_license(), &ws)
        .expect("pristine chain must project");
    assert_eq!(t.events.row_count(), 3);
    assert_eq!(t.objects.row_count(), 4); // 3 block objects + 1 file object
    assert_eq!(t.e2o.row_count(), 4); // 3 committed_block + 1 witness edge
    // Ordinal law visible in the events table: 0,1,2 zero-gap.
    if let Some(cgap_arrow::batch::Column::UInt64(ids)) =
        t.events.get("event_id")
    {
        assert_eq!(
            ids.values(),
            &[0u64, 1, 2],
            "event ids are the chain ordinals, zero-gap"
        );
    } else {
        panic!("event_id column missing or wrong type");
    }
}

/// THE FALSIFIER: flip one committed block byte -> the projection
/// emits the typed Spoliation refusal citing the chain invariant, and
/// emits NO tables. Mutations run on in-memory fixture state only.
#[test]
fn tamper_one_link_is_typed_refusal() {
    let (chain, blocks) = fixture(3);
    let ws = witnesses(3);
    let license = root_license();

    // Negative control: the pristine fixture still projects.
    assert!(
        Ocel2Projector::project(&chain, &blocks, &license, &ws).is_ok(),
        "pristine fixture must project (always-refusing = vacuous gate)"
    );

    // MUTATION: flip one byte of block 1's committed body.
    let ptr = blocks[1].bytes().as_ptr() as *mut u8;
    unsafe {
        *ptr.add(2) ^= 0x01;
    }

    // AFTER: typed Spoliation, no tables.
    assert_eq!(
        Ocel2Projector::project(&chain, &blocks, &license, &ws).err(),
        Some(Ocel2Refusal::Spoliation),
        "tampered block byte must refuse the projection (chain invariant)"
    );

    // Stored-receipt-field tamper also refuses (copies only).
    let mut m = chain.clone();
    m.receipts_mut()[2].hash[0] ^= 0xFF;
    assert_eq!(
        Ocel2Projector::project(&m, &blocks, &license, &ws).err(),
        Some(Ocel2Refusal::Spoliation),
        "mutated stored receipt hash must refuse the projection"
    );
}

/// Shape verdicts: empty chain, witness arity mismatch — typed, never
/// Spoliation-mislabeled.
#[test]
fn shape_verdicts_are_typed() {
    use cgap_ffi::receipt_chain::RingReceiptChain;
    let chain = RingReceiptChain::new();
    let (_, blocks) = fixture(1);
    assert_eq!(
        Ocel2Projector::project(&chain, &blocks, &root_license(), &[]).err(),
        Some(Ocel2Refusal::EmptyChain),
    );
    let (chain, blocks) = fixture(2);
    let ws = witnesses(3);
    assert_eq!(
        Ocel2Projector::project(&chain, &blocks, &root_license(), &ws).err(),
        Some(Ocel2Refusal::WitnessCountMismatch),
    );
}

/// Post-hoc license tamper: a valid record with different bytes than
/// the chain was minted under refuses the projection.
#[test]
fn license_tamper_refuses_projection() {
    let (chain, blocks) = fixture(2);
    let ws = witnesses(2);
    let mut t = root_license();
    t.origin_authority_digest[0] ^= 0x01;
    assert_eq!(
        Ocel2Projector::project(&chain, &blocks, &t, &ws).err(),
        Some(Ocel2Refusal::Spoliation),
    );
}

/// Round-trip court lives in roundtrip_court.rs; this leg asserts the
/// digest tie between projected columns and the real bytes: recompute
/// BLAKE3 over each block's bytes and match the projected hex.
#[test]
fn projected_block_digests_match_live_bytes() {
    let (chain, blocks) = fixture(4);
    let ws = witnesses(4);
    let t = Ocel2Projector::project(&chain, &blocks, &root_license(), &ws)
        .expect("pristine projects");
    let Some(cgap_arrow::batch::Column::Utf8(dig)) = t.events.get("block_digest") else {
        panic!("block_digest column missing");
    };
    for (i, b) in blocks.iter().enumerate() {
        let want = hex32(&hash(b.bytes()));
        assert_eq!(dig.row(i), want.as_bytes(), "block {i} digest tie");
    }
}

/// Local hex32 mirror (test-side) used to build expected values.
fn hex32(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}
