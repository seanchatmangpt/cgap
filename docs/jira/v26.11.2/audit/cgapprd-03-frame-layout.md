# cgapprd-03-frame-layout

CGAP-PRD-3 frame-layout study (2026-11-02, read-only lane): the 64 B
PRD assumption vs the landed 24 B reality, every consumer inventoried.

## 0. Mount state

lib.rs:147-165 — 15 modules mounted (FIX-06); ingress_ring :151,
pack_demux :160; GAP-01 note :153 (digest injected, no hard dep).
Ring consts: ingress_ring.rs:30 INGRESS_RING_BYTES=32_768; :33
CACHE_LINE_BYTES=64 (index padding — NOT a fact frame); :146
InRing<const CAP>; :280 StdIngressRing=IngressRing<32_768>.
FACT_STRIDE does not exist in src — it is test-side derivation
(posh_refusal_matrix_court.rs:34). Derived bound: 1,365 x 24 B =
32,760 B, 8 B short of the ring (court asserts exactly that).

## 1. The two numbers

| artifact | value | where |
|---|---|---|
| PRD/charter frame assumption | 64 B (4x16B "quad") | docs/DFLSS_CHARTER.md:30; echo lane-b03-cscb-c4.md:41 |
| Landed fact frame | 24 B = 3x u64 LE | generate_cscb_corpus.py:6,25,66 |

The 64 B/4x16B shape is real in exactly one place: Quad64,
cgap-arrow/src/layout.rs:36 (QUAD_SIZE=64, QUAD_ALIGN=64, :72/:75,
pinned :214-215) — the RDF-quad buffer layout of cgap-arrow, NOT the
CSCB fact frame. The PRD conflated the two.

## 2. Consumer inventory

- Producer: generate_cscb_corpus.py:25 TRIPLE_BYTES=24; :66 pack
  "<QQQ"; :6 docstring pin.
- csct ingress: cscb_c1_c2_ingress.rs:45 FRAME_BYTES=24; :63; :117;
  :154 chunks_exact(24); :159-160 [u8; 24] try_into (a 64 B frame
  would not compile here — type-level pin).
- Ring: ingress_ring.rs:30; posh court :34/:36/:157/:159 pins
  FACT_STRIDE==24 and 32,760.
- pack_demux.rs:40 RING_WINDOW_BYTES (layout-agnostic); :42-51
  FACT_WINDOW=1_365 pack stride with HONEST-NONE doc — independent
  route to the same number; frame-size change must NOT silently
  change the pack stride (:50-51: any coprime value serves).
- bn3: sigil.rs:265 encode_term takes ONE u64 — layout-INSENSITIVE;
  the only 64 B objects are the unrelated A04 CgapFrameHeader
  (project.rs:464).
- Go: corpus/gen.go:11/:24-27 FactBytes=24 FACT_WINDOW=1365 frozen;
  mutation_court_test.go:188-201 derives 32768/24==1365; byte-
  identity witness README:21 + lane-ipc-01:30-33 (cmp INTEROP_CMP_OK).
- Docs already recording the divergence: lane-ipc-01:19-21;
  workspace-cgap.yaml:23 ("FRAME_BYTES=24. NOT 64 B.").

## 3. Path (a): docs-only errata (RECOMMENDED)

Zero code moves; every consumer already agrees on 24 B. Edits:
1. DFLSS_CHARTER.md:30 — scope the 64 B claim to cgap-arrow Quad64.
2. lane-b03-cscb-c4.md:41 — replace "(64-byte frame chunks)" with
   24-byte FRAME_BYTES=24 (factually stale today).
3. 00-ard-prd.md — clarifying one-liner (no per-byte claim exists
   there today).
4. 15-ctq-tree / 14-dflss-charter — same one-line layout note.
5. Citation of record: workspace-cgap.yaml:23.
Size S; risk none — prose made to match what every court asserts.

## 4. Path (b): versioned 64 B wire format

Header "CGFB" + version + payload padded 24->64 (40 B dead weight,
62.5% overhead); compat FrameV1 decode keyed on magic; corpus
--frame flag; Go mirror. Changes ALL of section 2 + new module +
every receipt quoting 32,760/1,365 (soc receipt embeds "732x1365").
Derived bound under 64 B: 32768/64 = 512 facts/ring (2.67x fewer);
FACT_WINDOW coprime argument survives (1365 coprime to 64).
Size M mechanical, L with courts+FFI+C-ABI surface; the only real
upside is making the mutation court's missing-ingress-export arm
landable.

## 5. Does anything need 64 B? No.

csct wants exactly 3 coordinates ([u8;24] pin); pack_demux is
byte-stream-agnostic; bn3 is layout-blind; the ring is generic over
CAP (64 B buys only wasted capacity); Quad64 already lives where it
is used; NO code reads bytes 24..64 of any fact.

## 6. Recommendation + falsifier

Take (a). Falsifier: proven wrong the moment ANY consumer needs a
4th fact segment — a landed kernel reading bytes 24..64 (timestamp/
weight/predicate id) or a G4/G6 gate constrained by
MAX_INLINE_FACTS=512@64B. Absent that observable, (a) stands.

## See Also

audit/cgapprd-01-prd-audit.md · lanes/lane-ipc-01-scaffold.md ·
cgap-arrow/src/layout.rs:36
