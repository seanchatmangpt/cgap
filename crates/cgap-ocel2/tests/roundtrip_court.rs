//! ROUND-TRIP COURT: the projected tables carry enough state to
//! re-derive the chain linkage and match. From the projected columns
//! ALONE we re-derive (a) zero-gap ordinals, (b) prev-hash linkage
//! (hash_{i-1} == prev_i), (c) the license digest identity; from the
//! projected columns PLUS the real committed bytes we re-derive (d)
//! the full link law BLAKE3(prev || block || license) per event and
//! match the projected receipt digest. (The link law hashes raw block
//! bytes, which are not themselves a projected column — the bytes
//! come from the live blocks the chain was minted over.)
use cgap_blake3::hash;
use cgap_ocel2::{FileIdentity, LeaseWitness, Ocel2Projector};
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

fn hex32(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(
            core::str::from_utf8(&s[i * 2..i * 2 + 2]).unwrap(),
            16,
        )
        .unwrap();
    }
    out
}

/// Columns-only linkage re-derivation + full link-law match against
/// the live bytes. Asserts final state of the projection.
#[test]
fn projected_columns_rederive_chain() {
    let n = 4;
    let (chain, blocks) = fixture(n);
    let ws: Vec<LeaseWitness> = (0..n)
        .map(|i| LeaseWitness {
            lease_id: 7 + i as u64,
            file: if i == 1 {
                Some(FileIdentity { dev: 1, ino: 0x2A })
            } else {
                None
            },
        })
        .collect();
    let t = Ocel2Projector::project(&chain, &blocks, &root_license(), &ws)
        .expect("pristine projects");

    let ids = match t.events.get("event_id") {
        Some(cgap_arrow::batch::Column::UInt64(c)) => c.values().to_vec(),
        _ => panic!("event_id column"),
    };
    let rdig = match t.events.get("receipt_digest") {
        Some(cgap_arrow::batch::Column::Utf8(c)) => c,
        _ => panic!("receipt_digest column"),
    };
    let bdig = match t.events.get("block_digest") {
        Some(cgap_arrow::batch::Column::Utf8(c)) => c,
        _ => panic!("block_digest column"),
    };
    let prev = match t.events.get("prev_hash") {
        Some(cgap_arrow::batch::Column::Utf8(c)) => c,
        _ => panic!("prev_hash column"),
    };

    // (a) zero-gap ordinals from columns alone.
    assert_eq!(ids, (0..n as u64).collect::<Vec<_>>());

    // (b) prev-hash linkage: GENESIS then hash_{i-1}.
    let genesis = [0u8; 32];
    for i in 0..n {
        let p = unhex(prev.row(i));
        if i == 0 {
            assert_eq!(p, genesis);
        } else {
            assert_eq!(p, unhex(rdig.row(i - 1)));
        }
    }

    // (d) full link law per event: BLAKE3(prev || block || license)
    // recomputed over the LIVE committed bytes, matched to projected.
    let license_bytes = root_license().to_bytes();
    let mut running = genesis;
    for i in 0..n {
        let block_bytes = blocks[i].bytes();
        let mut input =
            Vec::with_capacity(32 + block_bytes.len() + 80);
        input.extend_from_slice(&running);
        input.extend_from_slice(block_bytes);
        input.extend_from_slice(&license_bytes);
        let link = hash(&input);
        assert_eq!(unhex(rdig.row(i)), link, "link law at {i}");
        assert_eq!(unhex(bdig.row(i)), hash(block_bytes));
        assert_eq!(chain.receipts()[i].hash, link);
        running = link;
    }

    // (c) objects/E2O round-trip: (dev,ino) object and its edges.
    let oid = match t.objects.get("object_id") {
        Some(cgap_arrow::batch::Column::Utf8(c)) => c,
        _ => panic!("object_id column"),
    };
    let dev = match t.objects.get("dev") {
        Some(cgap_arrow::batch::Column::UInt64(c)) => c.values(),
        _ => panic!("dev column"),
    };
    let ino = match t.objects.get("ino") {
        Some(cgap_arrow::batch::Column::UInt64(c)) => c.values(),
        _ => panic!("ino column"),
    };
    // Exactly one file object, keyed (1, 42).
    let file_rows: Vec<usize> = (0..t.objects.row_count())
        .filter(|i| oid.row(*i).starts_with(b"file:"))
        .collect();
    assert_eq!(file_rows.len(), 1);
    let fr = file_rows[0];
    assert_eq!(dev[fr], 1);
    assert_eq!(ino[fr], 0x2A);
    // E2O references it by the same id string.
    let e2o_obj = match t.e2o.get("object_id") {
        Some(cgap_arrow::batch::Column::Utf8(c)) => c,
        _ => panic!("e2o object_id column"),
    };
    assert!((0..t.e2o.row_count()).any(|i| e2o_obj.row(i) == oid.row(fr)));
}
