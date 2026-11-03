# Charter §4 Phase 1-2 (Define/Measure) verification — CE-03

Lane CE-03, 2026-11-02. Contract: writes ONLY this file. Companions:
`../dflss/03-msa-gauge.md` (gauge law), `../11-measured-claims.md`
(exclusive numeric source), `dod-04-unsafe-sram-census.md` (SRAM legs).

## 1. Per-claim verdicts

### C1 — Row 5 (10.20/10.30 ns/frame, quiet host): ALIVE
- Receipt-read, not re-run (bench subject is the owning lane's).
- `../11-measured-claims.md:29` cites `../waves/w3-l1-ingress.md:63,69-70`.
- On-disk check: w3-l1 lines 63/69-70 carry 10.20 fastest and
  10.30 median, method divan 0.1.21, 1,048,576-frame const;
  quiet-host scope re-confirmed vs red-10 within 4.5%.

### C2 — Row 28 (14.71/19.03/124.06 @1M, registered divergence): ALIVE
- Receipt-read. `../11-measured-claims.md:52`;
  `../redteam/red-10-timing-jitter-exploit.md:63` carries the exact
  fastest/median/slowest triple; host line :53: `Mac15,9, 16 cores,
  48 GB`, Apple Silicon, no SMT.

### C3 — Row 38 (36,048 B gated slice): ALIVE (re-run this session)
- `python3 scripts/verify_cscb_sram.py` -> EXIT 0: 36048 B
  (.text+.rodata+.data+.bss) <= 131072 B; headroom 95024 B.
- `python3 scripts/verify_cscb_sram.py --subject bn3` -> EXIT 1:
  typed `BN3_ENCLAVE_REFUSED`, 6,520,984 B = 49.75x ceiling.
- Both legs re-run this session (anti-vacuity flip leg included).
- Register note: `dod-04` said the figure was UNREGISTERED in
  11-measured-claims.md; row 38 now exists (line 166) — closed.

### C4 — N=7 Min/Med/Max law (03-msa-gauge §2): ALIVE (on-disk const)
- `crates/csct/src/bin/cscb_c3_closure.rs:52`:
  `const RUNS: usize = 7;` — fresh-read this session.
- Corroborated test-side: `crates/csct/tests/posh_batch_court.rs:59`
  `const RUNS: u32 = 7`.
- Quantile honesty (max-of-N is not a p99):
  `../lanes/lane-fix04-cscb-p99.md`; witnessed N=7 rows in
  03-msa-gauge §2 (23.22/23.35/23.43; 183/185/187).

### C5 — 41 ns timer floor: ALIVE (receipt-read)
- `../lanes/lane-ipc-03-latency-court.md:43`: 41 ns min positive
  monotonic delta, paired `time.Now`, 100k iterations.
- 03-msa-gauge §1 cites the same figure as the gauge-law floor.

## 2. THE HONEST MARKER — host-class scope (UNSUPPORTED)

The Phase 1-2 charter posture reads as though the measured baselines
hold on clean Apple Silicon AND x86_64 hosts. That universal claim is
UNSUPPORTED. Every receipt behind C1, C2, C4, C5 is THIS ONE host:

- Mac15,9, 16 cores, 48 GB, arm64, no SMT
  (`redteam/red-10-timing-jitter-exploit.md:53`; w3-l1 "this host").
- No x86_64 measurement exists anywhere in the campaign corpus
  (grep x86_64 over 11-measured-claims.md, dflss/, dod/, waves/,
  redteam/: zero timing-class hits this session).
- C3 (SRAM 36,048 B) is target-scoped (riscv32imc), so host class
  does not apply to it — the only row exempt from this marker.

Registered TARGET for a future window: an x86_64 (and, ideally,
SMT-bearing) reproduction of rows 5/28 with the same divan method of
record + CONTENTION-SUSPECT guard, before any cross-host claim is
admitted. Until then, all timing rows carry scope "one Apple Silicon
host (Mac15,9, arm64, no SMT), quiet vs contended as labeled".

## 3. Commands + exits (this session)

- `python3 scripts/verify_cscb_sram.py` -> EXIT 0
- `python3 scripts/verify_cscb_sram.py --subject bn3` -> EXIT 1
  (typed BN3_ENCLAVE_REFUSED — the anti-vacuity flip leg)
- On-disk reads: 11-measured-claims.md rows 5/28/38 (lines 29/52/166);
  cscb_c3_closure.rs:48-52; w3-l1-ingress.md:63-83;
  red-10-timing-jitter-exploit.md:28-88;
  lane-ipc-03-latency-court.md:40-50; dod-04 receipt §2/§4.

## 4. Divergences

1. The dispatch quoted "the charter's 'clean Apple Silicon AND
   x86_64 hosts' claim". No file on disk carries that string
   (grep over dflss/, 14-dflss-charter.md, 00-ard-prd.md: zero
   hits). The marker in §2 is filed against the charter POSTURE
   (implicit host-universality of the baselines), not a verbatim
   sentence; if a verbatim source exists it is outside my read
   scope and should be errata'd by its owner.
2. `dod-04-unsafe-sram-census.md` §"DIVERGENCE" states the 36,048 B
   figure was UNREGISTERED in 11-measured-claims.md; as of this
   session row 38 EXISTS at line 166 (registered post-dod-04).
   Errata against dod-04: the unregistration is closed.
3. csct test-side RUNS=7 also witnessed at
   `crates/csct/tests/posh_batch_court.rs:59` (`const RUNS: u32 = 7`)
   — corroborates the law beyond the bin.

## 5. Standing

- C1, C2, C5: ALIVE as receipt-read claims (receipts on disk, cited
  numbers match; not re-run this session — the benches are the
  owning lanes' subjects).
- C3: ALIVE — gate + refusal leg re-run this session, exits 0/1.
- C4: ALIVE — const fresh-read on disk.
- Host-universality: UNSUPPORTED / TARGET (x86_64 window, future).
- Lane receipt: this file. No git commands run.
