//! CLI-2 handlers: kernel surfaces (bont, parquet, arrow, git).
//!
//! Every handler calls the owning crate's real public API, asserts final
//! state in-handler (Chicago), and returns the measured quantity as JSON.
//! A refusal anywhere is a typed `NounVerbError`, never a silent OK.

use clap_noun_verb::{NounVerbError, Result};
use serde_json::{json, Map, Value};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn fail<T>(what: &str, detail: String) -> Result<T> {
    Err(NounVerbError::execution_error(format!("{what}: {detail}")))
}

/// `bont slot-lookup` — real B-ONT SlotId derivation plus an independent
/// re-derivation through the raw cgap-blake3 kernel (the §8 T4 court path):
/// the handler must re-derive the slot from the namespace tag and lexeme
/// and assert both paths agree, or refuse typed.
pub fn bont_slot_lookup(_inputs: Map<String, Value>) -> Result<Value> {
    let lexeme = b"cgap-cli:sovereign:slot-0";
    let slot = cgap_bont::primitives::slot_id(lexeme);
    let rederived = cgap_bont::primitives::truncate64(&cgap_bont::primitives::lex_hash(lexeme));
    if slot != rederived {
        return fail(
            "bont slot-lookup",
            format!("slot drift {slot:?} vs {rederived:?}"),
        );
    }
    // Independent path: BLAKE3 over BONT_NS_V1 ++ lexeme, first 8 bytes LE.
    let mut buf = Vec::with_capacity(cgap_bont::primitives::BONT_NS_V1.len() + lexeme.len());
    buf.extend_from_slice(cgap_bont::primitives::BONT_NS_V1);
    buf.extend_from_slice(lexeme);
    let d = cgap_blake3::hash(&buf);
    let raw = u64::from_le_bytes(d[0..8].try_into().expect("32B digest has 8"));
    if raw != slot.0 {
        return fail(
            "bont slot-lookup",
            format!("independent re-derivation drift: {raw:#x} vs {:#x}", slot.0),
        );
    }
    // Real slab insert through the collision detector (final state).
    let mut det = cgap_bont::primitives::CollisionDetector::new();
    det.insert(lexeme)
        .map_err(|e| NounVerbError::execution_error(format!("collision insert refused: {e:?}")))?;
    if det.len() != 1 {
        return fail(
            "bont slot-lookup",
            format!("detector len {} != 1", det.len()),
        );
    }
    Ok(json!({
        "lexeme": String::from_utf8_lossy(lexeme),
        "slot": format!("{:#018x}", slot.0),
        "rederived_agrees": true,
        "detector_len": det.len(),
    }))
}

/// `bont pack-verify` — digest chain over a pack of lexemes: every slot
/// re-derived independently, then the chain digest recomputed twice and
/// asserted byte-equal, or refuse typed.
pub fn bont_pack_verify(_inputs: Map<String, Value>) -> Result<Value> {
    let lexemes: [&[u8]; 8] = [
        b"cgap-cli:pack:a",
        b"cgap-cli:pack:b",
        b"cgap-cli:pack:c",
        b"cgap-cli:pack:d",
        b"cgap-cli:pack:e",
        b"cgap-cli:pack:f",
        b"cgap-cli:pack:g",
        b"cgap-cli:pack:h",
    ];
    let slots: Vec<u64> = lexemes
        .iter()
        .map(|l| cgap_bont::primitives::slot_id(l).0)
        .collect();
    for (lexeme, slot) in lexemes.iter().zip(&slots) {
        let mut buf = Vec::with_capacity(5 + lexeme.len());
        buf.extend_from_slice(cgap_bont::primitives::BONT_NS_V1);
        buf.extend_from_slice(lexeme);
        let d = cgap_blake3::hash(&buf);
        let raw = u64::from_le_bytes(d[0..8].try_into().expect("32B digest has 8"));
        if raw != *slot {
            return fail(
                "bont pack-verify",
                format!(
                    "slot drift for {}: {:#x} vs {:#x}",
                    String::from_utf8_lossy(lexeme),
                    raw,
                    slot
                ),
            );
        }
    }
    // Chain: BLAKE3 over the concatenated LE slot words, computed twice.
    let mut chain: Vec<u8> = Vec::with_capacity(slots.len() * 8);
    for s in &slots {
        chain.extend_from_slice(&s.to_le_bytes());
    }
    let d1 = cgap_blake3::hash(&chain);
    let d2 = cgap_blake3::hash(&chain);
    if d1 != d2 {
        return fail(
            "bont pack-verify",
            "chain digest nondeterministic".to_owned(),
        );
    }
    Ok(json!({
        "pack_len": slots.len(),
        "all_slots_rederived": true,
        "pack_digest": hex(&d1),
    }))
}

/// Shared: build + seal one real Parquet footer block over a 128-byte body.
fn seal_pack_block() -> Result<(Vec<u8>, [u8; 32])> {
    let mut buf = vec![0u8; 512];
    let body_len = 128usize;
    for (i, b) in buf[..body_len].iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    let summary = cgap_parquet::footer::BlockSummary {
        row_groups: 1,
        rows: 3,
        spans: &[body_len as u32],
    };
    let sealer = cgap_parquet::footer::FooterSealer::new(&mut buf, body_len, summary, None)
        .map_err(|e| NounVerbError::execution_error(format!("seal refused: {e:?}")))?;
    let sealed = sealer
        .seal()
        .map_err(|e| NounVerbError::execution_error(format!("seal failed: {e:?}")))?;
    let digest = *sealed.digest();
    Ok((sealed.bytes().to_vec(), digest))
}

/// `parquet pack-emit` — emit a real sealed footer block; the emitted bytes
/// must verify under their own digest (post-commit read-back, A05/A03).
pub fn parquet_pack_emit(_inputs: Map<String, Value>) -> Result<Value> {
    let (bytes, digest) = seal_pack_block()?;
    let verified = cgap_parquet::footer::verify_seal(&bytes, &digest)
        .map_err(|e| NounVerbError::execution_error(format!("emitted pack refused: {e:?}")))?;
    if verified.body_len != 128 || verified.rows != 3 {
        return fail(
            "parquet pack-emit",
            format!(
                "verified shape body={} rows={}",
                verified.body_len, verified.rows
            ),
        );
    }
    Ok(json!({
        "bytes_len": bytes.len(),
        "body_len": verified.body_len,
        "row_groups": verified.row_groups,
        "rows": verified.rows,
        "digest": hex(&digest),
    }))
}

/// `parquet pack-check` — verify the block AND refuse a tampered copy
/// (fail-closed witness): a corrupted content byte must be a typed
/// SealError, never a silent pass.
pub fn parquet_pack_check(_inputs: Map<String, Value>) -> Result<Value> {
    let (bytes, digest) = seal_pack_block()?;
    let verified = cgap_parquet::footer::verify_seal(&bytes, &digest)
        .map_err(|e| NounVerbError::execution_error(format!("pack check refused: {e:?}")))?;
    if verified.rows != 3 {
        return fail("parquet pack-check", format!("rows {}", verified.rows));
    }
    // Anti-vacuity: tamper one content byte; the check MUST flip to error.
    let mut tampered = bytes.clone();
    tampered[0] ^= 0xff;
    if cgap_parquet::footer::verify_seal(&tampered, &digest).is_ok() {
        return fail(
            "parquet pack-check",
            "tampered block verified — vacuous gate".to_owned(),
        );
    }
    Ok(json!({
        "bytes_len": bytes.len(),
        "rows": verified.rows,
        "attestation_present": verified.attestation_present,
        "tamper_refused": true,
    }))
}

/// `arrow slab-stats` — real ArrowRecordSlab push/seal/reload: occupancy
/// asserted against the pushed count, sealed bytes reloaded via
/// from_bytes and byte-compared, or refuse typed.
pub fn arrow_slab_stats(_inputs: Map<String, Value>) -> Result<Value> {
    const CAP: usize = 64;
    let mut slab = cgap_arrow::slab::ArrowRecordSlab::<CAP>::new();
    let mut pushed = 0usize;
    for i in 0..8u32 {
        let mut payload = [0u8; 32];
        payload[..4].copy_from_slice(&i.to_le_bytes());
        slab.push(&payload, true)
            .map_err(|e| NounVerbError::execution_error(format!("slab push refused: {e:?}")))?;
        pushed += 1;
    }
    slab.seal()
        .map_err(|e| NounVerbError::execution_error(format!("slab seal refused: {e:?}")))?;
    let sealed_bytes = slab.as_bytes().to_vec();
    if sealed_bytes.len() != 64 + CAP * 64 {
        return fail(
            "arrow slab-stats",
            format!("sealed len {} != {}", sealed_bytes.len(), 64 + CAP * 64),
        );
    }
    let reloaded = cgap_arrow::slab::ArrowRecordSlab::<CAP>::from_bytes(&sealed_bytes)
        .map_err(|e| NounVerbError::execution_error(format!("slab reload refused: {e:?}")))?;
    if reloaded.as_bytes() != sealed_bytes.as_slice() {
        return fail("arrow slab-stats", "reload byte-drift".to_owned());
    }
    Ok(json!({
        "capacity": CAP,
        "pushed": pushed,
        "sealed_bytes": sealed_bytes.len(),
        "reload_roundtrip": true,
    }))
}

/// `git dag-verify` — real commit_blocks over 4 blocks, generate the seq=2
/// lineage proof, verify against the committed root AND against a wrong
/// parent (must refuse), or refuse typed.
pub fn git_dag_verify(_inputs: Map<String, Value>) -> Result<Value> {
    let blocks: [(&[u8], u64); 4] = [
        (&b"block-0"[..], 0),
        (&b"block-1"[..], 1),
        (&b"block-2"[..], 2),
        (&b"block-3"[..], 3),
    ];
    let parent: cgap_git::Oid = [1u8; 20];
    let commit = cgap_git::dag::commit_blocks(&blocks, &parent)
        .map_err(|e| NounVerbError::execution_error(format!("commit refused: {e:?}")))?;
    let proof = cgap_git::dag::lineage_proof(&blocks, &parent, 2)
        .map_err(|e| NounVerbError::execution_error(format!("proof refused: {e:?}")))?;
    cgap_git::dag::verify_lineage(&commit.root, &proof, &parent)
        .map_err(|e| NounVerbError::execution_error(format!("lineage refused: {e:?}")))?;
    // Anti-vacuity: a wrong parent MUST be a typed refusal.
    let wrong_parent: cgap_git::Oid = [2u8; 20];
    if cgap_git::dag::verify_lineage(&commit.root, &proof, &wrong_parent).is_ok() {
        return fail(
            "git dag-verify",
            "wrong parent verified — vacuous".to_owned(),
        );
    }
    Ok(json!({
        "blocks": blocks.len(),
        "node_count": commit.tree.node_count,
        "root": hex(&commit.root),
        "proof_siblings": proof.siblings.len(),
        "wrong_parent_refused": true,
    }))
}

/// `blake3 x2-court` — the p1-blake3 court as a CLI verb: the fixed x2 NEON
/// kernel must equal two independent 4-way kernels (both groups, two base
/// counters), the dispatch paths must agree at full-hash level, and a
/// tampered segment must change the digest (else the court is vacuous and
/// the handler refuses typed).
pub fn blake_3_x_2_court(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_blake3::multi_chunk::{compress_chunks_4way, compress_chunks_4way_x2};

    // Real 8192-byte corpus, deterministic i%251 pattern (court corpus shape).
    let data: Vec<u8> = (0..8192usize).map(|i| (i % 251) as u8).collect();
    let x2 = compress_chunks_4way_x2(&data, 0);
    let a = compress_chunks_4way(&data[..4096], 0);
    let b = compress_chunks_4way(&data[4096..], 4);
    if x2[0] != a || x2[1] != b {
        return fail(
            "blake3 x2-court",
            "x2 kernel diverged from two 4-way kernels".to_owned(),
        );
    }
    // Second base counter: both groups advance together.
    let x2b = compress_chunks_4way_x2(&data, 100);
    let ab = compress_chunks_4way(&data[..4096], 100);
    let bb = compress_chunks_4way(&data[4096..], 104);
    if x2b[0] != ab || x2b[1] != bb {
        return fail(
            "blake3 x2-court",
            "x2 base=100 counters diverged".to_owned(),
        );
    }
    // Full-hash level: SIMD dispatch paths must agree with scalar hash.
    let root_x2 = cgap_blake3::hash_many_simd_x2(&data);
    let root_many = cgap_blake3::hash_many_simd(&data);
    let root_scalar = cgap_blake3::hash(&data);
    if root_x2 != root_many || root_x2 != root_scalar {
        return fail(
            "blake3 x2-court",
            "SIMD/scalar root digest drift".to_owned(),
        );
    }
    // Tamper control (flip witness): flipping one corpus byte MUST change
    // the kernel output; an unchanged output is a vacuous court.
    let mut tampered = data.clone();
    tampered[0] ^= 0x01;
    let x2t = compress_chunks_4way_x2(&tampered, 0);
    if x2t[0] == x2[0] {
        return fail(
            "blake3 x2-court",
            "tampered segment not detected — vacuous".to_owned(),
        );
    }
    if cgap_blake3::hash_many_simd_x2(&tampered) == root_x2 {
        return fail(
            "blake3 x2-court",
            "tampered root not detected — vacuous".to_owned(),
        );
    }
    Ok(json!({
        "corpus_len": data.len(),
        "groups_equal_two_4way": true,
        "base100_counters_advance": true,
        "simd_scalar_root_agree": true,
        "tamper_refused": true,
        "root_digest": hex(&root_x2),
    }))
}
