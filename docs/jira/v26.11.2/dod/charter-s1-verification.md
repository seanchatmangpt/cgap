# charter-s1-verification

Lane CE-01 · 2026-11-02 · repo /Users/sac/cgap (main, 7f7ccdf7 + session
diff). Charter §1-2 verification of `dflss/00-dflss-master-dossier.md`
(the session's master dossier; standalone charters also on disk:
`14-dflss-charter.md`, `dflss/01-project-charter.md` — all three read).
Sole write: this file. Every verdict re-grounded against on-disk
receipts and code this session; UNVERIFIED never rounds up.

## 1. Per-claim verdicts

### 1.1 Ingress 10.20 ns/frame, quiet host (claims row 5) — RECEIPTED (scoped)

`waves/w3-l1-ingress.md:63,69-70` verified on disk: divan table shows
10.20 fastest / 10.30 median ns/frame, target <15 MET, 10.1x vs P3-16
102.7 ns/frame baseline (fastest column, stated). Quiet-host scope is
part of the claim per red-10; red-10's own baseline re-run reproduced
within 4.5% (10.66/11.76, `redteam/red-10-timing-jitter-exploit.md`
W5 table). Charter G5a ("hold at 10.20 ns/frame quiet host; divergence
row 28", RECEIPTED) matches the row-5 scope exactly — no overclaim in
the charter's G5a row or its receipt note.

### 1.2 Registered divergence 124.06 ns/frame (claims row 28) — RECEIPTED

`redteam/red-10-timing-jitter-exploit.md:43-66` verified: combined
bw6+cpu9 run 14.71 / 19.03 / 124.06 fastest/median/slowest @1M
(slowest = 12.2x row-5's 10.20; >50 ns bar crossed); :77-88 carries
the breach-confirmed unqualified-claim verdict, cpu-only fastest
survival (-3.4%), and bw-dominance attribution. 11-measured-claims.md
row 28 registers it as the registered divergence of row 5's scope.
Dossier §1 quotes the triple with the red-10 cite — consistent.

### 1.3 SRAM 36,048 B gated slice (claims row 38) — RECEIPTED (subject-scoped)

`dod/dod-04-unsafe-sram-census.md` verified on disk: Leg 1 baseline
`verify_cscb_sram.py` EXIT 0, 36,048 B (.text+.rodata+.data+.bss)
<= 131,072 B, headroom 95,024 B (§2, lines ~72-89); Leg 2
`--subject bn3` EXIT 1 typed BN3_ENCLAVE_REFUSED at 6,520,984 B =
49.75x ceiling (red-12); §4 honest-scoping note states both scopes,
"neither number may be quoted without its scope", plus the dispatch
divergence (SRAM absent from claims rows 1-29 at dod-04 time). Now
registered as claims §6 row 38 with the full gate contract
(exit 0/1/2/3) and flip witnesses
(`lanes/lane-fix02-bn3-sram-gate.md:45-62`). Dossier §1 memory-
envelope paragraph and `dflss/01-project-charter.md` D3 row both
carry the gated-slice scope — consistent, no overclaim.

### 1.4 Zero-alloc court scoping — RECEIPTED (scoped; charter G2 row drifts)

`redteam/red-09-alloc-sentry-breach.md` verified: sentry window
allocs=0 bytes=0 over 46,442 relayed bytes with positive control
flipping (allocs=1 bytes=512); honesty window MEASURED bn3 fixpoint
allocs=2,662 / 650,294,994 bytes, "explicitly scoped OUT of the zero
claim" (:29, :74-78: "the zero-alloc claim is a claim about ring+seal,
not the fixpoint"); :81-82 cross-FFI zero-alloc leg UNSUPPORTED-by-
design (cgap-ffi keeps std on host). Dossier section 1 carries the
scoped form (per-CTQ M-CTQ-02 "VERIFIED (SCOPED)"). DIVERGENCE
(overclaim, flagged for coordinator): `14-dflss-charter.md:36` Goal
table row G2 reads "Zero heap allocation | nm alloc symbols |
0 alloc, 0 panic | MEASURED" UNSCOPED — the receipt note at :49 scopes
it to cgap-arrow, but the table cell itself invites a whole-kernel
reading that red-09's bn3 fixpoint honesty window directly refutes.
Contract restricts this lane to writes on this file only, so the G2
cell is NOT edited here. Suggested fix: append "ring+seal / cgap-arrow
surfaces only; bn3 fixpoint allocates (red-09 :74-78)" to the G2 cell.

### 1.5 Chatman Equation `dS_admitted + Phi_lease == R_receipt (mod Sigma_PQC)` — PROSE-ONLY
(achievable subset is code)

`redteam/red-13-receipt-desync-exploit.md:11-14` verified: the
formula has NO code referent; `Sigma_PQC` exists in prose only (7
docs, zero Rust hits, grep re-confirmed in red-13 section 1). The only
implemented receipt chain is `RingReceiptChain`
(`crates/cgap-ffi/src/receipt_chain.rs:71`), link law
`BLAKE3(prev_hash || block bytes || license bytes)`
(receipt_chain.rs:44,104-106). The achievable subset IS on disk and
receipted: lane XC-03 landed the strict per-link position invariant —
`index == i` zero-gap strict increase + `prev_hash == prior link's
hash`, violation is the single opaque `Spoliation` class
(receipt_chain.rs:26-29 doc; enforced in `verify`) — with court
`chain_position_monotonicity_invariant` (baseline 7/7 green; sandbox
mutation removal -> exit 101; mtime-bumped sha256-exact restore ->
re-green; `lanes/lane-xc03-position-invariant.md:39-64`). Shared-tree
file verified byte-exact this session: sha256(receipt_chain.rs) =
f3b91d5789fffe88d729c7384b8bf9f11619dcca8c44452fb42cf6beb99042c4 —
matches XC-03's recorded restore hash, so the invariant code on disk
is exactly the courted code. Known hole stands (red-13 section 3 /
dossier section 1): commit streams mint ZERO receipts today — the
chain is advisory, not wired into the commit path. Dossier section 1
carries this split faithfully; no charter overclaim found on the
equation itself.

## 2. Charter-text cross-checks (14-dflss-charter.md)

- G5a row ("hold at 10.20 ns/frame (quiet host; divergence row 28)",
  RECEIPTED) — consistent with rows 5/28; receipt note cites
  `waves/p2-penstock-escalation-law.md:32` (present on disk,
  penstock escalation law receipt exists).
- G1a/G2/G3/G5 rows: receipt notes match their cited wave receipts on
  disk (w3-t3-silicon.md:36, gate-5-silicon.md:26,
  p2-asm-0blr-census.md:37, a3-ring-sink.md:34-36). G2 table-cell
  scope drift flagged in section 1.4.
- G1b parquet 55,711 B stays UNVERIFIED-BY-RECEIPT in the charter
  (correct; claims row 17 records it as pre-link non-authoritative).
- Tollgates T2-T5 all OPEN, charter A3/A4/A5 OPEN — matches the
  dossier's RELEASE-CONDITIONAL riders (no READY issued anywhere).
- FMEA row "Unverified parquet figure in prose" and mtime-bump law
  cites are consistent with `waves/a02-finish-receipt.md:170-184`
  (mtime law witness present in v26.11.1 waves).

## 3. Fences (must-not-regress surfaces named + status)

- 11-measured-claims.md rows 5/28/38, sections 4-6 — read-only this
  lane; unchanged on disk (no edit, no mtime touch).
- Shared-tree code surfaces cited (receipt_chain.rs) — read-only;
  sha256 verified above, zero writes.
- No gate or court executable belongs to this lane; the XC-03 court
  and SRAM gate verdicts are cited from their owning receipts
  (commands + exits recorded there), not re-run here. Standing of
  those courts is inherited, explicitly NOT re-witnessed by CE-01.

## 4. Commands + exits (this session)

- ls dflss/ + redteam/ -> exit 0 (charter files located)
- sed -n reads of dod-04-unsafe-sram-census.md, w3-l1-ingress.md,
  red-10-timing-jitter-exploit.md, red-09-alloc-sentry-breach.md,
  red-13-receipt-desync-exploit.md -> exit 0 each
- grep position-invariant over lanes/ -> exit 0 (XC-03 receipt found)
- grep receipt_chain.rs invariant markers -> exit 0
- sha256sum crates/cgap-ffi/src/receipt_chain.rs -> exit 0, hash
  matches XC-03 restore hash
- python3 append to this file -> exit 0

## 5. Receipt (W7 terminus)

```json
{"lane":"CE-01",
 "files_written":[
  "/Users/sac/cgap/docs/jira/v26.11.2/dod/charter-s1-verification.md"],
 "commands":[
  "ls/grep/sed reads over docs/jira/v26.11.2/{dflss,dod,redteam,waves,lanes} (exit 0)",
  "sha256sum crates/cgap-ffi/src/receipt_chain.rs (exit 0, f3b91d57... match)",
  "python3 appends to charter-s1-verification.md (exit 0)"],
 "courts_run":[
  "none executable - verification lane; every court verdict cited from",
  "  its owning receipt (XC-03 flip battery, FIX-02 SRAM flips,",
  "  red-09 sentry battery)"],
 "standing":"ALIVE",
 "divergences":[
  "14-dflss-charter.md G2 table cell carries an UNSCOPED",
  "  '0 alloc, 0 panic MEASURED' - overclaim vs red-09 :74-78 (bn3",
  "  fixpoint allocates); NOT edited (write contract); coordinator to",
  "  scope the cell",
  "dispatch named 'charter sections 1-2': the dossier sections 1-2 are",
  "  the session master charter; standalone 14-dflss-charter.md and",
  "  dflss/01-project-charter.md also verified; no conflict between",
  "  them on the five mission claims"]}
```
