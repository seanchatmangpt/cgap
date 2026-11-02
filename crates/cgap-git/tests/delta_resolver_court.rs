//! GIT-04 delta-resolver court: correctness, cycle, depth, zero-heap purity.
//! lib.rs is NOT edited (wave contract) - the module is included via
//! #[path] so its API is exercised exactly as it will compile.
//!
//! Chicago discipline: assertions are on final byte state or a typed
//! refusal; no mocks. Anti-vacuity: every refusal court has a flip twin -
//! a minimally-perturbed valid input that MUST succeed, so a resolver that
//! refuses everything also fails the court.

#[path = "../src/delta_resolver.rs"]
mod delta_resolver;

use delta_resolver::{
    apply_delta, resolve_chain, DeltaError, Scratch, MAX_CHAIN_DEPTH, SCRATCH_LEN,
};

fn varint(mut v: usize, out: &mut Vec<u8>) {
    loop {
        let b = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

/// Encode one copy opcode reading base[off..off+n].
fn copy_op(out: &mut Vec<u8>, off: usize, n: usize) {
    let mut op: u8 = 0x80;
    let mut payload: [u8; 8] = [0; 8];
    let mut k = 0;
    for i in 0..4usize {
        let byte = ((off >> (8 * i)) & 0xFF) as u8;
        if byte != 0 || i == 0 && n == 0 {
            op |= 1 << (4 + i);
            payload[k] = byte;
            k += 1;
        }
    }
    let n_enc = if n == 0x10000 { 0 } else { n };
    for i in 0..4usize {
        let byte = ((n_enc >> (8 * i)) & 0xFF) as u8;
        if byte != 0 {
            op |= 1 << i;
            payload[k] = byte;
            k += 1;
        }
    }
    out.push(op);
    out.extend_from_slice(&payload[..k]);
}

/// Encode one insert opcode of literal bytes.
fn insert_op(out: &mut Vec<u8>, literal: &[u8]) {
    assert!(!literal.is_empty() && literal.len() <= 127);
    out.push(literal.len() as u8);
    out.extend_from_slice(literal);
}

/// Assemble a full delta: header varints + ops.
fn delta(target_len: usize, source_len: usize, ops: &[u8]) -> Vec<u8> {
    let mut d = Vec::new();
    varint(target_len, &mut d);
    varint(source_len, &mut d);
    d.extend_from_slice(ops);
    d
}

#[test]
fn copy_insert_delta_is_byte_exact() {
    let base = b"the quick brown fox jumps over the lazy dog";
    // Ops build "quick brown FOX! lazy dog": copy(5,11) + insert + copy(-22,8)
    let mut ops = Vec::new();
    copy_op(&mut ops, 4, 11); // base[4..15] = "quick brown"
    insert_op(&mut ops, b"FOX! ");
    copy_op(&mut ops, 35, 8); // base[35..43] = "lazy dog"
    let d = delta(24, base.len(), &ops);
    let mut sc = Scratch::new();
    let target = apply_delta(base, &d, &mut sc).expect("valid delta applies");
    // Independently computed expectation (no call into the resolver).
    let expected = {
        let b = &base[..];
        let mut v = Vec::new();
        v.extend_from_slice(&b[4..15]);
        v.extend_from_slice(b"FOX! ");
        v.extend_from_slice(&b[35..43]);
        v
    };
    assert_eq!(target, &expected[..]);
    assert_eq!(target.len(), 24);
}

#[test]
fn empty_delta_reproduces_base() {
    let base = b"packfile delta base";
    let mut ops = Vec::new();
    copy_op(&mut ops, 0, base.len());
    let d = delta(base.len(), base.len(), &ops);
    let mut sc = Scratch::new();
    let target = apply_delta(base, &d, &mut sc).expect("identity delta applies");
    assert_eq!(target, &base[..]);
}

#[test]
fn chain_of_16_resolves_iteratively() {
    let base = b"0123456789".to_vec();
    // Each hop re-asserts the full target via copy-from-base; chain of 16.
    let mut deltas: Vec<&[u8]> = Vec::new();
    let mut storage: Vec<Vec<u8>> = Vec::new();
    for hop in 1..=MAX_CHAIN_DEPTH {
        let body: Vec<u8> = vec![b'a' + (hop as u8 - 1); 10];
        let mut ops = Vec::new();
        insert_op(&mut ops, &body);
        storage.push(delta(10, 10, &ops));
    }
    let deltas: Vec<&[u8]> = storage.iter().map(|v| v.as_slice()).collect();

    let mut a = Scratch::new();
    let mut b = Scratch::new();
    let out = resolve_chain(&base, &deltas, &mut a, &mut b).expect("depth-16 chain resolves");
    // Final hop body: hop 16 letter 'p'.
    assert_eq!(out, &[b'p'; 10][..]);
}

#[test]
fn cyclical_chain_is_refused_typed() {
    // A -> B -> A: delta bytes repeat at index 2 => DeltaCycle{at:2}.
    let mut ops_a = Vec::new();
    insert_op(&mut ops_a, b"AAAAAAAAAA");
    let d_a = delta(10, 10, &ops_a);
    let mut ops_b = Vec::new();
    insert_op(&mut ops_b, b"BBBBBBBBBB");
    let d_b = delta(10, 10, &ops_b);
    let deltas: Vec<&[u8]> = vec![&d_a, &d_b, &d_a];
    let mut a = Scratch::new();
    let mut b = Scratch::new();
    let err = resolve_chain(b"0123456789", &deltas, &mut a, &mut b)
        .expect_err("cyclic chain must be refused");
    assert_eq!(err, DeltaError::DeltaCycle { at: 2 });
    // Flip twin: same chain with the repeat replaced by a distinct delta
    // gets PAST cycle detection (reaches application, then succeeds).
    let mut ops_c = Vec::new();
    insert_op(&mut ops_c, b"CCCCCCCCCC");
    let d_c = delta(10, 10, &ops_c);
    let open_chain: Vec<&[u8]> = vec![&d_a, &d_b, &d_c];
    let out = resolve_chain(b"0123456789", &open_chain, &mut a, &mut b)
        .expect("acyclic twin must pass cycle detection");
    assert_eq!(out, b"CCCCCCCCCC");
}

#[test]
fn depth_17_chain_is_refused_typed() {
    let mut deltas: Vec<&[u8]> = Vec::new();
    let mut storage: Vec<Vec<u8>> = Vec::new();
    for hop in 0..(MAX_CHAIN_DEPTH + 1) {
        let mut ops = Vec::new();
        insert_op(&mut ops, &[b'a' + (hop as u8 % 26); 8]);
        storage.push(delta(8, 8, &ops));
    }
    let deltas: Vec<&[u8]> = storage.iter().map(|v| v.as_slice()).collect();
    let mut a = Scratch::new();
    let mut b = Scratch::new();
    let err =
        resolve_chain(b"00000000", &deltas, &mut a, &mut b).expect_err("depth-17 must be refused");
    assert_eq!(err, DeltaError::ChainTooDeep { depth: 17 });
    // Flip twin: dropping one hop reaches depth 16 and resolves.
    let depth16: Vec<&[u8]> = deltas[..MAX_CHAIN_DEPTH].to_vec();
    let out = resolve_chain(b"00000000", &depth16, &mut a, &mut b).expect("depth-16 twin resolves");
    assert_eq!(out, &[b'a' + 15; 8][..]);
}

#[test]
fn mutation_refusals_with_flip_twins() {
    let base = b"the quick brown fox jumps over the lazy dog";
    let mut sc = Scratch::new();
    // Reserved opcode 0: refused.
    let mut ops = Vec::new();
    copy_op(&mut ops, 4, 11);
    insert_op(&mut ops, b"FOX! ");
    ops.push(0x00);
    let d = delta(17, base.len(), &ops); // 16 produced pre-opcode; 17 keeps Overrun silent
    assert_eq!(
        apply_delta(base, &d, &mut sc),
        Err(DeltaError::ReservedOpcode)
    );
    // Flip twin: same ops without the injected zero opcode succeed.
    let mut ops_ok = ops[..ops.len() - 1].to_vec();
    // ...but the target size must be corrected (copy+insert produced 16).
    let d_ok = delta(16, base.len(), &mut ops_ok);
    let t = apply_delta(base, &d_ok, &mut sc).expect("twin applies");
    assert_eq!(t, b"quick brownFOX! ");
    // Copy out of range: refused.
    let mut ops_oob = Vec::new();
    copy_op(&mut ops_oob, 10_000, 4);
    let d_oob = delta(4, base.len(), &ops_oob);
    assert_eq!(
        apply_delta(base, &d_oob, &mut sc),
        Err(DeltaError::CopyOutOfRange)
    );
    // Truncated delta: header varint never terminates.
    assert_eq!(
        apply_delta(base, &[0x80], &mut sc),
        Err(DeltaError::TruncatedDelta)
    );
    // Target size mismatch: declares 5, ops produce 4.
    let mut ops_mm = Vec::new();
    insert_op(&mut ops_mm, b"abcd");
    let d_mm = delta(5, base.len(), &ops_mm);
    assert_eq!(
        apply_delta(base, &d_mm, &mut sc),
        Err(DeltaError::TargetSizeMismatch {
            produced: 4,
            declared: 5
        })
    );
}

#[test]
fn oversized_target_is_refused_scratch_too_small() {
    // Declare a target larger than the fixed 11.6 KiB scratch.
    let d = delta(SCRATCH_LEN + 1, 4, &[]);
    let mut sc = Scratch::new();
    assert_eq!(
        apply_delta(b"abcd", &d, &mut sc),
        Err(DeltaError::ScratchTooSmall {
            needed: SCRATCH_LEN + 1
        })
    );
    // Flip twin: exactly-at-capacity target (all inserts, 11878 bytes in
    // 127-byte chunks) applies cleanly.
    let mut ops = Vec::new();
    let chunk = [b'x'; 127];
    let mut remaining = SCRATCH_LEN;
    while remaining > 0 {
        let n = remaining.min(127);
        insert_op(&mut ops, &chunk[..n]);
        remaining -= n;
    }
    let d_full = delta(SCRATCH_LEN, 4, &ops);
    let t = apply_delta(b"abcd", &d_full, &mut sc).expect("full-scratch twin applies");
    assert_eq!(t.len(), SCRATCH_LEN);
    assert!(t.iter().all(|&c| c == b'x'));
}

/// Zero-heap static invariant: the module never references alloc/std/Vec.
/// (This is the "state which" line: we chose the STATIC INVARIANT form,
/// court-enforced by source scan, not a tracking-alloc instrument.)
#[test]
fn module_source_has_no_heap_or_recursion() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/delta_resolver.rs"),
    )
    .expect("read module source");
    for banned in ["alloc::", "Vec<", "Box<", "std::", "recurse"] {
        assert_eq!(
            src.matches(banned).count(),
            0,
            "banned token {banned} in module"
        );
    }
    // Single definition of each entrypoint, and zero internal call
    // sites of resolve_chain (iterative enforcement).
    assert_eq!(src.matches("fn apply_delta").count(), 1);
    assert_eq!(src.matches("fn resolve_chain").count(), 1);
    assert_eq!(src.matches("resolve_chain").count(), 1);
}
