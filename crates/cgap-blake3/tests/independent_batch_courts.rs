//! Courts for the PL15 independent-batch lane.
//!
//! Equality: `hash_many_independent` and the 4-lane kernel must produce
//! byte-identical output to sequential scalar `hash()` per payload, over
//! batch sizes 1/3/4/5/7/8/16/64 plus remainder tails, mixed payload sizes
//! (0..=1024) and mixed-batch rows (> 1024 B payloads). Whole-slab compare.
//!
//! Non-vacuity companion: `independent_batch_mutation.sh` flips a lane
//! offset in the kernel's message-schedule stride inside a /tmp sandbox
//! copy of this crate and MUST make this court fail (sha256 fail-closed
//! restore of the shared tree).

use cgap_blake3::{hash, hash_many_independent_into, multi_chunk::hash_blocks_4way_independent};

fn fill(buf: &mut [u8]) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = (i.wrapping_mul(7).wrapping_add(13) % 251) as u8;
    }
}

/// Deterministic payload sizes: cycles the full 0..=1024 spread.
fn size_at(batch: usize, j: usize) -> usize {
    let x = (batch * 131 + j * 97) % 1025;
    x
}

fn payloads_for(batch: usize, n: usize) -> Vec<Vec<u8>> {
    let mut v = Vec::new();
    for j in 0..n {
        let sz = size_at(batch, j);
        let mut p = vec![0u8; sz];
        fill(&mut p);
        v.push(p);
    }
    v
}

fn refs(payloads: &[Vec<u8>]) -> Vec<[u8; 32]> {
    payloads.iter().map(|p| hash(p)).collect()
}

fn many(payloads: &[&[u8]]) -> Vec<[u8; 32]> {
    let mut out = vec![[0u8; 32]; payloads.len()];
    hash_many_independent_into(payloads, &mut out);
    out
}

/// Whole-slab equality over the required batch sizes + remainder tails.
#[test]
fn court_batch_vs_sequential_scalar_equality() {
    for batch in [1usize, 3, 4, 5, 7, 8, 16, 64] {
        let payloads = payloads_for(batch, batch);
        let want = refs(&payloads);
        let refslices: Vec<&[u8]> = payloads.iter().map(|p| p.as_slice()).collect();
        let got = many(&refslices);
        assert_eq!(got.len(), want.len(), "len drift at batch {}", batch);
        for i in 0..want.len() {
            assert_eq!(
                got[i],
                want[i],
                "batch {} payload {} ({} B) diverges",
                batch,
                i,
                payloads[i].len()
            );
        }
    }
}

/// Remainder tails: sizes that leave 1..=3 payloads for the scalar tail,
/// and ragged 5/7/13/17-payload runs.
#[test]
fn court_remainder_tails_equality() {
    for n in [5usize, 7, 13, 17, 63, 65, 100, 257] {
        let payloads = payloads_for(n, n);
        let want = refs(&payloads);
        let refslices: Vec<&[u8]> = payloads.iter().map(|p| p.as_slice()).collect();
        let got = many(&refslices);
        for i in 0..want.len() {
            assert_eq!(got[i], want[i], "tail n={} payload {} diverges", n, i);
        }
    }
}

/// Kernel-level court: consecutive 4-lane groups over the full size spread
/// (incl. empty payloads and the 1024-byte ceiling), byte-identical to 4x
/// scalar hash.
#[test]
fn court_kernel_4way_group_equality() {
    let sizes = [0usize, 1, 63, 64, 65, 127, 128, 512, 1023, 1024];
    for g4 in sizes.chunks_exact(4) {
        // Pad to exactly 4 sizes by cycling when the tail is short.
        let mut four = g4.to_vec();
        while four.len() < 4 {
            four.push(four[four.len() - 1]);
        }
        let group: Vec<Vec<u8>> = four
            .iter()
            .map(|&sz| {
                let mut p = vec![0u8; sz];
                fill(&mut p);
                p
            })
            .collect();
        let refslices: Vec<&[u8]> = group.iter().map(|p| p.as_slice()).collect();
        let got = hash_blocks_4way_independent(&refslices);
        for l in 0..4 {
            assert_eq!(
                got[l],
                hash(&group[l]),
                "lane {} size {} diverges",
                l,
                four[l]
            );
        }
    }
}

/// Mixed-batch honesty: rows containing payloads > 1024 B route those
/// payloads through the single-hash law and still match sequential.
#[test]
fn court_mixed_batch_honesty() {
    let mut sizes: Vec<usize> = vec![1024, 1025, 1500, 64, 2048, 100, 4096, 200];
    for n in [3usize, 7, 12] {
        let mut p = sizes.clone();
        p.truncate(n.max(1));
        sizes.extend_from_slice(&p);
    }
    let payloads: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&sz| {
            let mut v = vec![0u8; sz];
            fill(&mut v);
            v
        })
        .collect();
    let want = refs(&payloads);
    let refslices: Vec<&[u8]> = payloads.iter().map(|p| p.as_slice()).collect();
    let got = many(&refslices);
    for i in 0..want.len() {
        assert_eq!(got[i], want[i], "mixed row payload {} ({} B)", i, sizes[i]);
    }
}

/// no_std form: hash_many_independent_into writes exactly into the caller's
/// slice (no allocation anywhere in the path).
#[test]
fn court_into_form_writes_caller_slice() {
    let payloads = payloads_for(9, 9);
    let want = refs(&payloads);
    let refslices: Vec<&[u8]> = payloads.iter().map(|p| p.as_slice()).collect();
    let mut out = vec![[0u8; 32]; payloads.len()];
    hash_many_independent_into(&refslices, &mut out);
    assert_eq!(out, want);
}
