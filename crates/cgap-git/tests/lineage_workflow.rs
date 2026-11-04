//! Lane p1-lineage workflow court (v26.11.2): statement lineage via Git
//! in cgap-git. Real collaborators, Chicago style — no mocks.
//!
//! Workflow under test: seal 3 successive statement segments (real
//! cgap-parquet sealed-slab format via `FooterSealer`/`verify_seal`),
//! content-address them (real `cgap_blake3::hash` register +
//! cgap-git's SHA-1 blob OIDs), pack them (3 packfile blobs via
//! `write_pack_v2`), commit one 3-leaf Merkle tree
//! (`commit_blocks`, node_count = 2L-1 = 5), then commit a child
//! snapshot whose parent head is content-addressed from the parent
//! commit's `.cgap-dag` bytes — 2 commits in a parent chain.
//! Read back through cgap-git's own readers (`parse_pack_v2`,
//! `IdxV2::parse`/`lookup`, `decode_dag_file`) and re-verify seals.
//!
//! Courts (flip-or-FAIL, typed refusals):
//! - tamper one pack byte -> `PackChecksumMismatch` (typed);
//! - tamper one `.cgap-dag` byte -> `DagError::ChecksumMismatch` (typed);
//! - tamper one sealed-slab body byte -> `SealError::ChecksumMismatch`
//!   with `region == Body` (typed);
//! - wrong content address -> `SealError::DigestMismatch` (typed);
//! - lineage proof over a wrong parent -> `DagError::LineageMismatch`;
//! - tampered sibling path -> `DagError::RootMismatch`.
//!
//! Time-travel court: decoding the parent vs child `.cgap-dag` yields
//! different content (different root + parent OID) — both witnesses are
//! read back from the wire, never from in-memory state.
//!
//! Frozen pins respected: license URN, v26.10.31 SHACL IRIs untouched;
//! seed constants not referenced (outside bitslice.rs scope).

use cgap_git::dag::{
    commit_blocks, decode_dag_file, encode_dag_file, lineage_proof, verify_lineage, DagError,
    FrameFields, GENESIS_OID,
};
use cgap_git::idx_read::IdxV2;
use cgap_git::pack_write::{build_pack_and_idx, crc32, git_object_oid, parse_pack_v2, sha1};
use cgap_git::Oid;
use cgap_parquet::footer::{verify_seal, BlockSummary, CorruptRegion, FooterSealer, SealError};

/// Seal one statement segment into the real cgap-parquet sealed-slab
/// format: body bytes + footer (meta block, 80-byte attestation slot,
/// fnv1a64 checksum, `PAR2` magic, footer length). Returns owned bytes
/// plus the 32-byte content address (`block_digest`).
fn seal_segment(body: &[u8]) -> (Vec<u8>, [u8; 32]) {
    let mut buf = vec![0u8; body.len() + 256];
    // The sealer seals `buf[..body_len]` as the body — write it first.
    buf[..body.len()].copy_from_slice(body);
    let block = FooterSealer::new(
        &mut buf,
        body.len(),
        BlockSummary {
            row_groups: 1,
            rows: body.len() as u64,
            spans: &[body.len() as u32],
        },
        None,
    )
    .expect("sealer accepts non-empty body, one span summing to body_len")
    .seal()
    .expect("seal");
    // Own the slab: copy the sealed bytes out of the borrowed buffer
    // before the borrow ends (body + footer, exactly `bytes().len()`).
    (block.bytes().to_vec(), *block.digest())
}

#[test]
fn lineage_workflow_seal_pack_commit_chain_roundtrip_and_typed_refusals() {
    // ---- 1. Seal 3 successive statement segments (real sealed slabs) ----
    let bodies: Vec<Vec<u8>> = (0..3)
        .map(|i| {
            let mut b = Vec::new();
            b.extend_from_slice(format!("statement-segment-{i}: ").as_bytes());
            b.extend_from_slice(&[0x41u8 + i as u8; 48]);
            b
        })
        .collect();
    let sealed: Vec<(Vec<u8>, [u8; 32])> = bodies.iter().map(|b| seal_segment(b)).collect();
    let slabs: Vec<Vec<u8>> = sealed.iter().map(|(s, _)| s.clone()).collect();
    let digests: Vec<[u8; 32]> = sealed.iter().map(|(_, d)| *d).collect();
    assert_eq!(slabs.len(), 3);
    // Distinct segments -> distinct content addresses.
    assert_ne!(digests[0], digests[1]);
    assert_ne!(digests[1], digests[2]);

    // ---- 2. Seal read-back: every slab re-verifies from its bytes ----
    for ((slab, d), body) in slabs.iter().zip(&digests).zip(&bodies) {
        let v = verify_seal(slab, d).expect("clean sealed slab verifies");
        assert_eq!(v.row_groups, 1);
        assert_eq!(v.rows, body.len() as u64, "rows == sealed body length");
    }

    // ---- 3. BLAKE3 register (real cgap-blake3, cross-kernel) ----
    let blake: Vec<[u8; 32]> = slabs.iter().map(|s| cgap_blake3::hash(s)).collect();
    assert_ne!(blake[0], blake[1]);
    let mut tampered_slab = slabs[1].clone();
    let tmid = tampered_slab.len() / 2;
    tampered_slab[tmid] ^= 0x01;
    assert_ne!(
        cgap_blake3::hash(&tampered_slab),
        blake[1],
        "VACUOUS: byte flip did not move the BLAKE3 digest"
    );

    // ---- 4. 3 packfile blobs: pack + idx, read back via own readers ----
    let blocks: Vec<(&[u8], u64)> = slabs
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_slice(), i as u64))
        .collect();
    let refs: Vec<&[u8]> = slabs.iter().map(|s| s.as_slice()).collect();
    let (pack, idx, entries) = build_pack_and_idx(&refs).expect("pack+idx written");
    assert_eq!(entries.len(), 3);
    let objs = parse_pack_v2(&pack).expect("pack parses");
    assert_eq!(objs.len(), 3);
    let body_end = pack.len() - 20;
    for (i, obj) in objs.iter().enumerate() {
        assert_eq!(obj.obj_type, 3, "type 3 = blob");
        assert_eq!(obj.size as usize, slabs[i].len(), "declared size == slab");
        // Locate the varint object header end, then decode the zlib
        // stream through cgap-git's own reader (deflate::zlib_decompress).
        let entry_end = if i + 1 < objs.len() {
            objs[i + 1].offset as usize
        } else {
            body_end
        };
        let start = obj.offset as usize;
        let mut p = start + 1;
        while pack[p] & 0x80 != 0 {
            p += 1;
        }
        p += 1;
        let data = cgap_git::deflate::zlib_decompress(&pack[p..entry_end])
            .expect("zlib stream decodes via cgap-git's own reader");
        assert_eq!(
            &data[..],
            &slabs[i][..],
            "pack body round-trips to the exact sealed slab"
        );
        let crc_span = &pack[obj.offset as usize..entry_end];
        assert_eq!(crc32(crc_span), obj.crc, "crc over the entry bytes");
        // The idx writer sorts entries by OID: match by content address,
        // never by position (entries[i] is NOT blob i).
        let oid = git_object_oid("blob", &slabs[i]);
        let entry = entries
            .iter()
            .find(|e| e.oid == oid)
            .expect("idx entry exists for every slab oid");
        assert_eq!(crc32(crc_span), entry.crc, "idx crc agrees");
    }

    // idx v2 read-back: checksum, lookup, oid/crc/offset agreement.
    let idxt = IdxV2::parse(&idx).expect("idx parses");
    assert_eq!(idxt.len(), 3);
    idxt.verify_idx_checksum().expect("idx trailer checksums");
    for (i, obj) in objs.iter().enumerate() {
        let oid = git_object_oid("blob", &slabs[i]);
        let (slot, off) = idxt
            .lookup(&oid)
            .expect("idx well-formed")
            .expect("oid present");
        assert_eq!(off, u64::from(obj.offset), "idx offset == pack offset");
        assert_eq!(idxt.crc(slot).expect("crc"), obj.crc, "idx crc == parsed crc");
    }

    // Audit court: one flipped pack byte -> typed refusal.
    let mut bad_pack = pack.clone();
    let mid = bad_pack.len() / 2;
    bad_pack[mid] ^= 0x01;
    match parse_pack_v2(&bad_pack) {
        Err(cgap_git::GitError::PackChecksumMismatch) => {}
        other => panic!("pack tamper: expected typed checksum refusal, got {other:?}"),
    }

    // ---- 5. One tree: commit_blocks over the 3 slabs (genesis parent) ----
    let commit_a = commit_blocks(&blocks, &GENESIS_OID).expect("commit A");
    let tree_a = &commit_a.tree;
    assert_eq!(tree_a.block_count, 3);
    assert_eq!(tree_a.node_count, 5, "node count law 2L-1");
    assert_eq!(tree_a.parent_oid, GENESIS_OID);

    // Cross-register binding: the DAG's per-block OIDs are exactly the
    // blob OIDs the pack register committed (same content-address law).
    let mut dag_oids = commit_a.oids.clone();
    dag_oids.sort_unstable();
    let mut pack_oids: Vec<Oid> = entries.iter().map(|e| e.oid).collect();
    pack_oids.sort_unstable();
    assert_eq!(dag_oids, pack_oids, "registers committed different content");

    // ---- 6. Parent chain: commit B parented on commit A's dag bytes ----
    let dag_a = encode_dag_file(tree_a);
    let head_a: Oid = git_object_oid("tree", &dag_a);
    let commit_b = commit_blocks(&[(slabs[2].as_slice(), 0)], &head_a).expect("commit B");
    assert_eq!(commit_b.tree.parent_oid, head_a);
    assert_ne!(commit_b.root, commit_a.root);

    // ---- 7. Time-travel court: decode parent vs child, different ----
    let dag_b = encode_dag_file(&commit_b.tree);
    let back_a = decode_dag_file(&dag_a).expect("dag A decodes from wire");
    let back_b = decode_dag_file(&dag_b).expect("dag B decodes from wire");
    assert_eq!(back_a, *tree_a, "parent dag round-trips byte-exact");
    assert_eq!(back_b, commit_b.tree, "child dag round-trips byte-exact");
    assert_ne!(back_a.root, back_b.root, "different roots witness");
    assert_ne!(
        back_a.parent_oid, back_b.parent_oid,
        "different parent linkage witnesses"
    );

    // ---- 8. Lineage proofs: clean, wrong-parent, tampered sibling ----
    let proof = lineage_proof(&blocks, &GENESIS_OID, 1).expect("proof for seq 1");
    verify_lineage(&commit_a.root, &proof, &GENESIS_OID).expect("clean lineage verifies");
    assert_eq!(proof.block_git_oid, git_object_oid("blob", &slabs[1]));
    match verify_lineage(&commit_a.root, &proof, &head_a) {
        Err(DagError::LineageMismatch) => {}
        other => panic!("wrong parent: expected LineageMismatch, got {other:?}"),
    }
    let mut evil = proof.clone();
    evil.siblings[0].1[0] ^= 0x80;
    match verify_lineage(&commit_a.root, &evil, &GENESIS_OID) {
        Err(DagError::RootMismatch) => {}
        other => panic!("tampered sibling: expected RootMismatch, got {other:?}"),
    }
    let proof_b = lineage_proof(&[(slabs[2].as_slice(), 0)], &head_a, 0).expect("child proof");
    verify_lineage(&commit_b.root, &proof_b, &head_a).expect("child lineage verifies");

    // ---- 9. Dag-file tamper court: one flipped byte -> typed ----
    let mut bad_dag = dag_a.clone();
    bad_dag[20] ^= 0x01;
    match decode_dag_file(&bad_dag) {
        Err(DagError::ChecksumMismatch) => {}
        other => panic!("dag tamper: expected ChecksumMismatch, got {other:?}"),
    }

    // ---- 10. Sealed-slab tamper courts: typed, region-localized ----
    let mut bad_slab = slabs[0].clone();
    bad_slab[2] ^= 0x01;
    match verify_seal(&bad_slab, &digests[0]) {
        Err(SealError::ChecksumMismatch {
            region: CorruptRegion::Body,
        }) => {}
        other => panic!("slab tamper: expected Body checksum refusal, got {other:?}"),
    }
    let mut wrong = digests[0];
    wrong[0] ^= 0x01;
    match verify_seal(&slabs[0], &wrong) {
        Err(SealError::DigestMismatch) => {}
        other => panic!("wrong address: expected DigestMismatch, got {other:?}"),
    }

    // ---- 11. Frame supply contract: transport checksum distinct ----
    let pack_checksum: Oid = pack[body_end..].try_into().expect("pack trailer");
    assert_eq!(sha1(&pack[..body_end]), pack_checksum);
    let fields: FrameFields = commit_a.frame_fields(pack_checksum);
    assert_eq!(fields.root, commit_a.root);
    assert_eq!(fields.parent_git_oid, GENESIS_OID);
    assert_eq!(fields.block_count, 3);
    assert_ne!(&fields.pack_sha1[..], &fields.root[..20]);
}
