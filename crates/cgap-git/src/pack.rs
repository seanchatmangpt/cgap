//! Stable pack facade (lane A03, DoD-2): one import point over both
//! content-addressing registers.
//!
//! - **Git-compat register** (SHA-1 OIDs, packfile-v2/idx-v2 bytes):
//!   `write_pack_v2`, `write_idx_v2`, `IdxEntry`, `PackBuilder`,
//!   `ParsedObject`, `IdxV2`, parsers, `git_object_oid`.
//! - **Receipt register** (in-crate BLAKE3, 32-byte Merkle root):
//!   `commit_blocks`, `merkle_root`, `lineage_proof`, `verify_lineage`,
//!   `Commit`, `DagTree`, `LineageProof`, `FrameFields`, wire codecs.
//!
//! Layout note: this file is wired as `crate::dag::pack` via `#[path]`
//! from `dag.rs` because `lib.rs` is contract-off-limits this wave. The
//! DoD-2 top-level path `crate::pack` is a one-line coordinator
//! promotion (`pub mod pack;` + `pub use dag::pack::*;` in `lib.rs`);
//! until then the facade is reachable and courted at
//! `cgap_git::dag::pack`. ADR-03-3/ADR-03-5 unchanged.
//!
//! No code lives here: this is `pub use` only (ADR-03-3 — "satisfied
//! without moving existing code"). Both registers stay cryptographically
//! independent; `git_object_oid` never meets BLAKE3 (R-5 guard, D-5).

pub use crate::idx_read::IdxV2;

pub use crate::pack_write::{
    build_pack_and_idx, crc32, encode_object_header, git_object_oid, parse_idx_v2, parse_pack_v2,
    sha1, write_idx_v2, write_pack_v2, Crc32, IdxEntry, PackBuilder, ParsedObject, Sha1,
};

pub use crate::dag::{
    commit_blocks, decode_dag_file, encode_dag_file, lineage_proof, merkle_root, verify_lineage,
    Commit, DagDigest, DagError, DagNodePayload, DagTree, FrameFields, LineageProof,
};

pub use crate::{GitError, Oid};

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    /// DoD-2 facade court: every DoD-2 name resolves through the facade
    /// and one real snapshot drives BOTH registers through it.
    /// Anti-vacuity: mutating the parent edge must move the root
    /// (else the receipt register is vacuous); deleting any re-exported
    /// name is a compile abort (SHIM-FACADE, recorded in the receipt).
    #[test]
    fn facade_court_both_registers_round_trip() {
        let bodies: &[&[u8]] = &[b"block-a", b"block-b", b"block-c"];
        let parent: Oid = git_object_oid("blob", b"parent-snapshot-head");

        // Receipt register: commit + parent-edge binding.
        let refs: Vec<(&[u8], u64)> =
            bodies.iter().enumerate().map(|(i, b)| (*b, i as u64)).collect();
        let commit = commit_blocks(&refs, &parent).expect("facade commit_blocks");
        assert_eq!(commit.tree.parent_oid, parent);
        assert_eq!(commit.tree.block_count, 3);
        let mut foreign_parent = parent;
        foreign_parent[0] ^= 0x80;
        let foreign = commit_blocks(&refs, &foreign_parent).expect("commit");
        assert_ne!(
            foreign.root, commit.root,
            "VACUOUS: parent-edge mutation did not move the root"
        );

        // Git-compat register: pack the same bodies, parse records.
        let pack = write_pack_v2(bodies).expect("facade write_pack_v2");
        let parsed = parse_pack_v2(&pack).expect("facade parse_pack_v2");
        assert_eq!(parsed.len(), 3, "three objects in the pack");
        for (obj, body) in parsed.iter().zip(bodies) {
            assert_eq!(obj.obj_type, 3, "type 3 = blob");
            assert_eq!(obj.size, body.len() as u64, "declared size == body");
        }

        // idx v2 from the parsed records; reader leg (oids, crcs, offs).
        let mut entries: Vec<IdxEntry> = parsed
            .iter()
            .zip(bodies)
            .map(|(obj, body)| IdxEntry {
                oid: git_object_oid("blob", body),
                offset: obj.offset,
                crc: obj.crc,
            })
            .collect();
        let pack_checksum: Oid = pack[pack.len() - 20..].try_into().expect("pack trailer");
        let idx = write_idx_v2(&mut entries, &pack_checksum).expect("facade write_idx_v2");
        let (idx_oids, idx_crcs, idx_offs) = parse_idx_v2(&idx).expect("facade parse_idx_v2");
        assert_eq!(idx_oids, entries.iter().map(|e| e.oid).collect::<Vec<_>>());
        assert_eq!(idx_crcs, entries.iter().map(|e| e.crc).collect::<Vec<_>>());
        assert_eq!(idx_offs, entries.iter().map(|e| e.offset).collect::<Vec<_>>());

        // Cross-register binding: the OIDs the git register committed are
        // exactly the per-block OIDs the receipt register carries.
        let mut commit_oids = commit.oids.clone();
        commit_oids.sort_unstable();
        assert_eq!(
            commit_oids, idx_oids,
            "receipt register and git register committed different content"
        );

        // §4.2 frame supply contract through the facade: pack_sha1 is the
        // transport checksum — the pack trailer, i.e. SHA-1 over the pack
        // bytes above it — distinct from both content OIDs and the root.
        let content_sum = sha1(&pack[..pack.len() - 20]);
        assert_eq!(
            content_sum, pack_checksum,
            "pack trailer is not the SHA-1 of its content bytes"
        );
        let fields: FrameFields = commit.frame_fields(pack_checksum);
        assert_eq!(fields.root, commit.root);
        assert_eq!(fields.parent_git_oid, parent);
        assert_eq!(fields.block_count, 3);
        assert_eq!(fields.pack_sha1, pack_checksum);
        assert_ne!(
            &fields.pack_sha1[..],
            &fields.root[..20],
            "transport checksum collides with the receipt root"
        );

        // §5.1 wire codec round-trips through the facade.
        let file = encode_dag_file(&commit.tree);
        let decoded = decode_dag_file(&file).expect("facade decode_dag_file");
        assert_eq!(decoded, commit.tree);
    }
}
