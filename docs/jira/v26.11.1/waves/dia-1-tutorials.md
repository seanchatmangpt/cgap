# dia-1-tutorials

DIA-1 TUTORIALS (lane-runner) — Diataxis tutorials: learning-oriented,
first closed loops. Repo /Users/sac/cgap (canonical checkout). Lease:
docs/diataxis/tutorials/{01-first-substrate-loop,02-ingest-and-seal,
03-run-the-courts}.md (NEW files) + this receipt. No git writes.

## Standing: ALIVE

All three tutorials written; both runnable programs executed green this
session as written (verbatim code blocks, exit 0); the court suite and
the gate self-check executed green this session.

## Files written

- docs/diataxis/tutorials/01-first-substrate-loop.md (160 lines)
- docs/diataxis/tutorials/02-ingest-and-seal.md (119 lines)
- docs/diataxis/tutorials/03-run-the-courts.md (79 lines)
- docs/jira/v26.11.1/waves/dia-1-tutorials.md (this receipt)

Existing tutorials (01-first-columnar-batch, 02-hash-and-packfile,
03-ffi-c-data-interface, getting-started) were left untouched — different
filenames, no collision; the dispatch named my three filenames exactly.

## Commands + exits (this session, UTC+8h local PST)

| # | command | exit | evidence |
|---|---|---|---|
| 1 | cargo test -p cgap-court --test court_verdict | 0 | 14 passed / 0 failed / 26.07s; includes a19_seal_1m_ledger_court, a19_seal_1m_eval_court, a14_dam_ingress_court |
| 2 | mktemp -d /tmp/dia1.XXXX; cargo new consumer; cargo add cgap@26.11.1 | 0 | /tmp/dia1.*; manifest `cgap = "26.11.1"` registry-sourced |
| 3 | cargo add cgap-parquet@26.11.1 --features std | 0 | FileSink enabled for tutorial 02 |
| 4 | tutorial-01 program (main.rs) cargo run -q | 0 (after 3 compile fixes) | final line DIA-1 FIRST-SUBSTRATE-LOOP: ALL PASS |
| 5 | tutorial-02 program (examples/seal.rs) cargo run -q --example seal | 0 | sealed: body=1024 meta=30 row_groups=4 rows=1024 bytes on disk=1150; corruption localized to region: Body |
| 6 | tutorial-01 verbatim doc code block re-extracted from the .md and re-run | 0 | the file's code IS the run witness, byte-for-byte |
| 7 | python3 scripts/zero_string_gate.py --self-check | 0 | SELF-CHECK RESULT PASS; [SKIP] lines loud with reasons |
| 8 | awk length>100 sweep over all 3 tutorials | 0 hits | <=100-char line law held |

## Grounding (every example traced to a witnessed surface)

- Tutorial 01's six surfaces are the W2-6 consumer's proven calls
  (waves/f3-14-consumer.md, W2-6-consumer section: registry-sourced,
  26.11.1, all seven courts flipped). This lane re-ran the same shapes
  against the same registry version, exit 0.
- Tutorial 02 is grounded in the a19_seal court shapes
  (FooterSealer/BlockSummary/verify_seal/SealError + FileSink BlockSink,
  mmap_writer.rs:674) and witnessed live: 1150 bytes on disk,
  ChecksumMismatch{region: Body}.
- Tutorial 03's command and output are this session's real run
  (table row 1); the self-check invocation witnessed (row 7).

## Courts (anti-vacuity; this lane's own flip witnesses)

| court | mutation | typed verdict witnessed |
|---|---|---|
| pack trailer | byte 13 ^= 0xff | Err(PackChecksumMismatch) |
| dag lineage | wrong parent OID | Err(LineageMismatch) |
| rle truncation | drop last stream byte | Err(LiteralOverrun{..}) |
| footer digest | wrong digest on intact bytes | Err(DigestMismatch) |
| footer body | byte 3*256+5 ^= 0xff on disk block | Err(ChecksumMismatch{region: Body}) |
| resolution gate | n/a (cited from W2-6: 26.11.99 -> 101 vs 26.11.1 -> 0) | cited, not re-run |

## Divergences (dispatch vs reality)

1. Marketplace pack scouted: readme-diataxis-pack exists but its shape
   is README-splitting (flat README -> quadrants), not tutorial
   authoring; no ggen pack generates these tutorial files, so markdown
   was written directly. Not a pack-eligible surface.
2. Dispatch said "pack checksum, dag lineage, footer digest, rle
   decode" — executed, but the footer-digest court needed a premise fix
   witnessed live: verify_seal's verdict order (shape -> attestation ->
   checksum -> digest) means a BODY flip surfaces as
   ChecksumMismatch{region: Body}, not DigestMismatch; the tutorial
   flips the digest argument on intact bytes instead. Recorded, not
   hidden.
3. Compile fixes during tutorial-01 grounding (each is a doc-level
   premise correction): (a) umbrella module paths are cgap::cgap_blake3
   etc. (re-exports of crate names, not cgap::blake3); (b) format_code
   is a Column method reached via batch.get(..), not a Batch method;
   (c) FooterSealer::seal returns Result and the buffer must be
   body_len + footer room (128-byte body in a 128-byte buffer flips
   RunOverrun{249,128}).
4. Tutorial 02 needed one extra dep (cgap-parquet with features =
   ["std"]) because FileSink is feature-gated; cgap umbrella forwards no
   features. Disclosed in the tutorial.
5. FooterSealer::new signature takes (buf, body_len, BlockSummary,
   attestation) — the tutorial's buffer-sizing rule (body + >= 96-byte
   trailer + meta) is derived from the witnessed RunOverrun flip, not
   from a doc.

## Cleanup

/tmp/dia1.* consumer workspace (incl. target/), /tmp/dia1_dir.txt, and
/tmp/dia1-seal-demo.bin removed via python3 shutil.rmtree/os.remove after
this write. Nothing kept; this receipt plus the tutorial files are the
proof.
