# Tutorial: ingest and seal

Learning-oriented: you saw `FooterSealer` once in tutorial 01; here you
learn the *mental model* behind it — frames through a dam, sealed into a
verifiable ledger — at a scale you can hold in your head. The production
shape (1M-row ledger, attestation) is exactly this flow scaled up and
proved by the `a19_seal` courts in the repo's own suite.

## What you need

- Tutorial 01's `consumer` workspace.
- One extra dependency (the file sink lives behind the `std` feature):

```console
$ cargo add cgap-parquet@26.11.1 --features std
```

`cgap-parquet` is `#![no_std]` at its core; `FileSink` is the hosted
attach-point, so it is opt-in. Everything else stays umbrella-only.

## The mental model

- **Frames** are the unit of ingress: small payloads that stream in.
- **The dam** is a fixed buffer: frames accumulate into a *body*, and the
  body's byte length is the dam's water line.
- **Sealing** appends a 96-byte fixed trailer + footer metadata and
  computes a 32-byte digest of the whole block. Seal once.
- **Verifying** re-derives everything from the bytes alone: verdict order
  is shape -> attestation -> checksum -> digest. Verify anywhere, cheaply,
  forever.

## Step 1: the program

Copy to `examples/seal.rs`:

```rust
use cgap::cgap_parquet::footer::{self, BlockSummary};
use cgap::cgap_parquet::mmap_writer::{BlockSink, FileSink};

fn main() {
    // 1. The dam: N "frames" (row payloads) accumulate into a body.
    let frames: Vec<Vec<u8>> = (0u8..64).map(|i| vec![i; 16]).collect();
    let body_len: usize = frames.iter().map(|f| f.len()).sum();

    // 2. Body + footer room in one fixed buffer.
    let mut buf = vec![0u8; body_len + 256];
    buf[..body_len].copy_from_slice(&frames.concat());

    // 3. Seal: summary declares 4 row groups, 1024 rows total.
    let spans = [256u32; 4];
    let summary = BlockSummary { row_groups: 4, rows: 1024, spans: &spans };
    let sealed = footer::FooterSealer::new(&mut buf, body_len, summary, None)
        .expect("sealer")
        .seal()
        .expect("seal");
    let digest = *sealed.digest();

    // 4. Flush to a REAL file through the std FileSink.
    let path = std::env::temp_dir().join("dia1-seal-demo.bin");
    let file = std::fs::File::create(&path).expect("create");
    let sink = FileSink::new(file, vec![0u8; 8192].into_boxed_slice());
    sealed.flush_to(sink).expect("flush");

    // 5. Read back from disk: verify from bytes alone.
    let bytes = std::fs::read(&path).expect("read");
    let v = footer::verify_seal(&bytes, &digest).expect("verify");
    println!(
        "sealed: body={} meta={} row_groups={} rows={} bytes on disk={}",
        v.body_len, v.meta_len, v.row_groups, v.rows, bytes.len()
    );

    // 6. Corruption court: flip one body byte, read the typed verdict.
    let mut corrupt = bytes.clone();
    let corrupt_idx = 3 * 256 + 5; // inside row group 3
    corrupt[corrupt_idx] ^= 0xff;
    match footer::verify_seal(&corrupt, &digest) {
        Err(footer::SealError::ChecksumMismatch { region, .. }) => {
            println!("corruption localized to region: {region:?}")
        }
        other => panic!("expected ChecksumMismatch, got {other:?}"),
    }
    println!("DIA-1 INGEST-AND-SEAL: ALL PASS");
}
```

## Step 2: run it

```console
$ cargo run --example seal
sealed: body=1024 meta=30 row_groups=4 rows=1024 bytes on disk=1150
corruption localized to region: Body
DIA-1 INGEST-AND-SEAL: ALL PASS
```

(That is the witnessed output of this exact program this session; your
temp path may differ, the numbers will not.)

## What just happened

- `FooterSealer::new` refuses bad *shapes* before sealing: zero row
  groups, empty body, spans that do not sum to `body_len`, or a body
  bigger than the buffer are all typed `RunOverrun` refusals.
- `seal()` computed `digest` over body + footer; `verify_seal` checks the
  same bytes three ways (shape, checksum, digest) and reports **which
  region** corrupted (`region: Body` above, from a flip inside row group
  3).
- `flush_to` pushed the whole sealed block to a real file through the
  `BlockSink` trait — the same trait the mmap backend attaches through.
- Scale this up 1024x and you have the 1M-row ledger the `a19_seal`
  courts exercise in tutorial 03.

## Where next

- Tutorial 03 (`03-run-the-courts.md`): run the suite that proves the
  1M-row ledger court, and learn to read a verdict.
- Reference: `cgap_parquet::footer` module docs (docs.rs, crate
  `cgap-parquet`).
- How-to guides live in `docs/diataxis/how-to/`; rationale in
  `docs/diataxis/explanation/`.
