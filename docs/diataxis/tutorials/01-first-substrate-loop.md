# Tutorial: your first substrate loop

Learning-oriented: one complete, guaranteed-achievable win. You build a
small Rust binary that touches all five published kernel crates through the
`cgap` umbrella — BLAKE3 hashing, a columnar batch, a git packfile, DAG
lineage, RLE decoding — and you flip four typed-refusal courts to see that
every boundary in this substrate actually refuses.

## What you need

- Rust stable with `cargo` on PATH; network access to crates.io.
- About five minutes.

## Step 1: create the workspace

```console
$ cargo new consumer
$ cd consumer
$ cargo add cgap@26.11.1
```

`Cargo.toml` gains exactly one line: `cgap = "26.11.1"`. Zero path
dependencies; all five crates (cgap + its four kernels) resolve from
crates.io and checksum-verify against the registry index. This exact flow
was witnessed with a registry-sourced build on 2026-11-02
(docs/jira/v26.11.1/waves/f3-14-consumer.md, W2-6 section; re-run green in
docs/jira/v26.11.1/waves/dia-1-tutorials.md).

## Step 2: replace src/main.rs

Copy this program verbatim. It is the exact shape that ran green against
the registry-published crates this session (see the receipt's commands
table for the exit code).

```rust
use cgap::cgap_blake3 as blake3;
use cgap::cgap_arrow::{batch::BatchBuilder, primitive::PrimitiveBuilder};
use cgap::cgap_git::dag;
use cgap::cgap_git::dag::GENESIS_OID;
use cgap::cgap_git::pack_write::{parse_pack_v2, write_pack_v2};
use cgap::cgap_parquet::{footer, rle};

fn main() {
    // 1. cgap-blake3: one call, 32-byte digest
    let h = blake3::hash(b"cgap-tutorial");
    assert_ne!(h, blake3::hash(b"cgap-tutoria1"));

    // 2. cgap-arrow: build a column, freeze it, read it back
    let mut col = PrimitiveBuilder::<i32>::new();
    col.push(Some(10));
    col.push(Some(20));
    col.push(Some(30));
    let batch = BatchBuilder::new()
        .with_column("nums", cgap::cgap_arrow::batch::ColumnBuilder::Int32(col))
        .finish()
        .expect("batch finish");
    assert_eq!(batch.row_count(), 3);
    assert_eq!(batch.get("nums").expect("col").format_code(), "i");

    // 3. cgap-git: write a pack v2, parse it back, flip a byte
    let blobs: Vec<&[u8]> = vec![b"alpha", b"beta"];
    let pack = write_pack_v2(&blobs).expect("pack write");
    assert_eq!(&pack[0..4], b"PACK");
    assert_eq!(parse_pack_v2(&pack).expect("parse").len(), 2);
    let mut mutated = pack.clone();
    mutated[13] ^= 0xff; // the SHA-1 trailer byte
    assert!(matches!(
        parse_pack_v2(&mutated),
        Err(cgap::cgap_git::GitError::PackChecksumMismatch)
    ));

    // 4. cgap-git: commit blocks into a Merkle DAG, prove lineage
    let blocks: Vec<(&[u8], u64)> =
        vec![(b"block-a", 1), (b"block-b", 2), (b"block-c", 3)];
    let commit = dag::commit_blocks(&blocks, &GENESIS_OID).expect("commit");
    let proof = dag::lineage_proof(&blocks, &GENESIS_OID, 3).expect("proof");
    dag::verify_lineage(&commit.root, &proof, &GENESIS_OID).expect("verify");

    // ...and flip the parent: the typed refusal
    let bad = dag::lineage_proof(&blocks, &[9u8; 20], 3).expect("proof");
    assert!(matches!(
        dag::verify_lineage(&commit.root, &bad, &GENESIS_OID),
        Err(dag::DagError::LineageMismatch)
    ));

    // 5. cgap-parquet: RLE round-trip, then truncation refusal
    let vals: Vec<u32> = (0..16).collect();
    let stream = rle::rle_encode(&vals, 4);
    let mut out = vec![0u32; 16];
    rle::rle_decode(&stream, &mut out).expect("decode");
    assert_eq!(out, vals);
    let cut = &stream[..stream.len() - 1];
    assert!(matches!(
        rle::rle_decode(cut, &mut out),
        Err(cgap::cgap_parquet::ParquetError::LiteralOverrun { .. })
    ));

    // 6. cgap-parquet: seal a block with a footer, verify, wrong-digest
    let mut buf = vec![0u8; 256]; // body (128 bytes) + footer room
    let spans = [64u32, 64];
    let summary = footer::BlockSummary {
        row_groups: 2,
        rows: 10,
        spans: &spans,
    };
    let sealed = footer::FooterSealer::new(&mut buf, 128, summary, None)
        .expect("sealer")
        .seal()
        .expect("seal");
    let digest = *sealed.digest();
    let v = footer::verify_seal(sealed.bytes(), &digest).expect("verify");
    assert_eq!(v.row_groups, 2);

    // verify_seal verdict order: shape -> attestation -> checksum ->
    // digest. To see the digest court, verify intact bytes against a
    // WRONG digest:
    let wrong_digest = [7u8; 32];
    assert!(matches!(
        footer::verify_seal(sealed.bytes(), &wrong_digest),
        Err(footer::SealError::DigestMismatch)
    ));

    println!("DIA-1 FIRST-SUBSTRATE-LOOP: ALL PASS");
}
```

## Step 3: run it

```console
$ cargo run
DIA-1 FIRST-SUBSTRATE-LOOP: ALL PASS
```

If any assert fires, one of the boundaries above did not refuse — that is
a bug in the substrate, not in your copy of the tutorial. File it.

## What just happened

Four ideas, one per court you flipped:

1. **Content addressing** — the pack's SHA-1 trailer and the DAG's block
   OIDs are computed from bytes, so any mutation is detectable
   (`PackChecksumMismatch`, `LineageMismatch`).
2. **Typed refusals, not panics** — every boundary returns
   `Result<_, SpecificError>`; you matched on the exact variant.
3. **Verify is cheap, seal is once** — `verify_seal` and
   `verify_lineage` re-derive the answer from the bytes alone; no state
   to carry.
4. **The umbrella is thin** — `cgap::cgap_blake3`, `cgap::cgap_arrow`,
   `cgap::cgap_git`, `cgap::cgap_parquet` are re-exports of the kernel
   crates; one dependency line buys all five.

## Where next

- Tutorial 02 (`02-ingest-and-seal.md`): the same seal path at
  ingest-dam scale — frames in, sealed ledger out.
- Tutorial 03 (`03-run-the-courts.md`): run the repo's own court suite
  and read a verdict.
- Reference: each kernel crate's docs.rs page (search "cgap" on
  crates.io).
