# RED-20 — Master Red-Team Synthesis & Armor Plan (lanes RED-01..RED-19)

Lane RED-20 · 2026-11-02 · repo /Users/sac/cgap · READ-ONLY synthesis lane.
Contract: read-only over crates/, include/, scripts/; sole write is this
file. No git commands. Every claim below is re-grounded against the 19
lane receipts on disk in `docs/jira/v26.11.2/redteam/` this session —
the coordinator rollup was verified against the receipts, with two
corrections noted (§0).

## 0. Method and rollup corrections

All 19 receipts read in full (paths in §1/§2/§3 tables). Corrections to
the coordinator rollup, per the receipts as landed:

1. RED-10's "12.2x" is the SLOWEST-SAMPLE ratio under combined
   contention (124.06 ns/frame vs the 10.20 claimed fastest,
   red-10-timing-jitter-exploit.md:79-82); the fastest column under
   cpu-only contention actually SURVIVED at -3.4% (:84-85). The breach
   is the unqualified/hostile-environment claim, not the quiet-host
   receipt, which RED-10 reproduced within 4.5% (:41).
2. RED-11 additionally found the riscv32imc DEFAULT-features build
   FAILS (exit 101, E0463 std; red-11:34-37) — "clean no_std" holds
   only with `--no-default-features`. The rollup carried this; it is
   repeated here because it is a release-gate surface, not just a
   court gap.

Wave standing at synthesis time: 19/19 lanes landed, every receipt
carries its own command+exit record. This lane adds no new execution —
its claim is the consolidation, and each consolidated row cites the
owning receipt where the command+exit lives.

## 1. Breach register (exploitability x blast radius ranked)

Severity: S1 = defeats a revenue/legality load-bearing claim; S2 =
defeats an integrity/anti-tamper claim; S3 = defeats a performance or
ops claim; S4 = claim-hygiene (unmeasured number stated as fact).

| # | Lane | Finding (grounded) | Reproducer (see owning receipt) | Blast radius | Sev |
|---|---|---|---|---|---|
| B1 | RED-05 | License litigability: covenant-vs-condition (*Ma*) fork + misuse per se + retroactivity kill-shot — the license's money machine "eats itself" (red-05:127-142, 209-222) | DJ complaint skeleton, Part A/B/C | All §4 revenue extraction; the entire P28-SIL enforcement posture | S1 |
| B2 | RED-04 | No revenue-aggregation clause: affiliate rings, dispersed LLC trusts, unconsolidated 50/50 JVs escape §2.2 tests at every node; $50M figure absent from the instrument itself (red-04:70-124, 28-50) | Clause-by-clause W3 structures 1-4 | Safe harbor becomes a per-entity paper shield; $500M enterprise licenses through sub-threshold SPEs | S1 |
| B3 | RED-06 | ASC 450 accrual mandate self-declared, unasserted, unestimable (no tariff, no demand mechanism); meter self-reporting and lossy by the instrument's own §6.2 (red-06:47-139) | 5 defusal points D1-D5 | $0 reserve is the correct audit posture; regularization/true-up demands rejectable as unliquidated | S1 |
| B4 | RED-08 | Consortium neutralization uncaptured: covenant one-directional; no confidentiality/NDA/information-sharing clause; IPR funding not Circumvention Dialogue (red-08:60-121) | W4 clause-sweep table | Peer enterprises can pool invalidation + share circumvention analytics | S1 |
| B5 | RED-18 | P28 attestation is keyless/unsigned/BLAKE3-only — clone-everything: flash read once = valid 80-B attestation recomputable; no HW root of trust; charter's "hardware-attested" (12-ard-prd-cgap.md:31) has zero referent in crates/ (red-18:80-91, 100-106) | §3.3/§3.4 vectors | Supply-chain interdiction undetectable; image cloning with valid attestation | S2 |
| B6 | RED-13 | FA9 spill-gate path-alias O_TRUNC truncation: FNV-of-raw-path keying lets a second thread re-`File::create` the owner's drained spill file — exit 7 BREACH, 2x reproduced, while fa9_commit_gating stays green (red-13:27-56) | /tmp/red13-exploit, exit 7 | Silent destruction of committed spill bytes under path aliasing; court vacuity hole | S2 |
| B7 | RED-11 | ref_store impl32 libcall RMW torn under ISR: ceiling check (:295-301) and fetch_add (:306) separated; lease > ceiling minted by witness; u32 wrap-to-0 replay at WIDTH; zero reentrancy court coverage (red-11:44-96) | /tmp/red11-wit witness+control, exit 0 | Duplicate/replayed lease authority on riscv32imc — the monotonic-lease invariant ref_store.rs:33-39 claims is false under preemption | S2 |
| B8 | RED-12 | SRAM gate subject-list gap: verify_cscb_sram.py DEPENDENCY_CRATES = blake3+branchless only; static NaEngine+NaMatches = 6,521,032 B = 49.75x the 131,072 B ceiling (red-12:29-74) | /tmp/red12-rv llvm-size, exit 0 | "Kernel fits the envelope" is true only of a 2-crate slice; bn3 (the engine) silently outside every SRAM claim | S2 |
| B9 | RED-01 | AHDCLT "technically unconstructable" defeated: C99 clean-room rebuild of all SWAR kernels + prefilter mix from doc comments alone, differential+exhaustive clean, 1.02 ns/frame kernel-proxy (red-01:130-166); falsifier 16-ahdclt.md:104-106 fires | /tmp/red01/cleanroom.c, exit 0 | Doc comments are the complete construction recipe; Pillar 1 is decoration for the compute-kernel surface | S3 |
| B10 | RED-02 | No deconstruction-trap mechanism exists (grep rc=1 for any anti-LLM mechanism; gates never see foreign rewrites); chunked extraction pipeline constructed on measured graph (red-02:101-128); falsifier 16-ahdclt.md:69-71 fires | W4 pipeline spec | Pillar 2 decoration; the repo's own courts double as the attacker's conformance suite (:96-99) | S3 |
| B11 | RED-03 | FFI wire decoupled in ~41h for a dlopen-bind-and-call consumer (11-symbol stub dylib compiled clean); PROOF not decoupled — byte-identity/spoliation/golden-vector courts crack every stub (red-03:64-122) | /tmp/red03-shim/shim.c, LINK_OK | Weak consumers rewirable <48h; strong consumers (court-runners) immune | S3 |
| B12 | RED-10 | Latency claim not robust: combined cache+CPU contention → slowest 124.06 ns/frame, 12.2x claimed; median 19.03 ns/frame breaks the <15 ns target; bandwidth channel dominant (red-10:61-88) | /tmp/red10 hog generators | Quiet-host receipts valid; any hostile-environment or constant-time reading is dead | S3 |
| B13 | RED-16 | Runtime-ops surface absent: no telemetry/health/watchdog, no DR restore court, no RMA/fleet lifecycle, no alerting — verification stack is 100% build-time (red-16:42-47) | grep evidence A1-A6 | Bare-metal/edge deployment thesis carries unpriced ops burden | S3 |
| B14 | RED-17 | Compendium §6.3 economics breach the 11-measured-claims exclusivity law: ">75% core-hour", "$4.5-5.2M/yr", "4.5-5.2x ROI", "30-40% RAM headroom" all TARGET-class stated as fact, and the dollar model fails on min-billing truncation/ingestion licensing/egress untouched (red-17:24-101) | W2 grading vs 11-measured-claims.md:3-8, 52-64 | Marketing compendium is a liability document, not a revenue document | S4 |
| B15 | RED-14 | Trip-gate surface (principle): class 101 trip-able by zero-cost unauthenticated empty-pop; `octopus_merge_andon` panics on caller input; any future host mapping to `process::exit` becomes a one-packet kill switch — no such mapping exists on disk today (red-14:31-35, 62-68) | /tmp/red14 flood probe, exit 0 (2,000,001 trips, liveness intact) | Forward-looking: constrains future host/CLI integration | S3-fwd |
| B16 | RED-15 | Unsalted-BLAKE3 dictionary-reversibility class note on commit_synth identity lines — no redaction claim exists to breach, so UNSUPPORTED not BREACH (red-15:44-51) | V1 analysis | Only if a privacy claim is ever narrated over cgap-git | S4-fwd |
| B17 | RED-19 | Adoption-funnel breach conceded by the repo's own compliance model (calendar not compressible, 11_security...:25-28,51): deterrence moat defends payment/retention, not first adoption vs SOC2-postured incumbents (red-19:55-101) | Attacks A/C grading | Mainstream funnel; high-assurance segment (S4 auditors) defended | S3 |
| B18 | RED-07 | Human-layer provenance: 25/45 retaliatory subpoena items have zero on-disk referent (CLA, COI screening, clean-room protocol, LLM provenance, litigation hold) while artifact provenance defeats 11 (red-07:180-199) | §3 subpoena battery | Trade-secret counter-claim center of mass | S1-legal |

## 2. UNSUPPORTED-targets register (narrated but absent from the repo)

The organization narrates the following; the tree contains no referent.
Each row is itself a claim-hygiene defect: any external document or
pitch repeating these narrates unreceipted assertions. Search method
per row lives in the owning receipt.

| Narrated claim | Verdict | Evidence | Defect class |
|---|---|---|---|
| Fogclave 1L desktop appliance, $899 | UNSUPPORTED | zero grep hits repo-wide (red-18:10-20) | product narration w/o referent |
| Tranche 1 Confidentiality / NDA; Tripartite Agreement | UNSUPPORTED | `grep -rni 'tripartite\|tranche'` → only unrelated wave labels (red-05:16-23) | deal-structure narration w/o instrument |
| Tranche 2 Liquidated Historical Arrears / true-up | UNSUPPORTED | tranche/arrears/liquidated/true-up zero substantive hits (red-06:14-21) | revenue-enforcement narration w/o tariff or invoice artifact |
| PII redaction / "SWAR PII stripping mask" | UNSUPPORTED | grep redact/pii/pseudonym → exit 1 (red-15:13-14) | privacy posture w/o code |
| Constant-time SWAR ingress | UNSUPPORTED | absent from 11-measured-claims.md; PRD-prose only (red-10:16-25) | security-property narration w/o measurement |
| Deconstruction traps / anti-LLM tripwires (AHDCLT Pillar 2) | UNSUPPORTED mechanism | grep rc=1; gate inventory targets original-tree properties only (red-02:103-119) | doctrine pillar w/o mechanism |
| Pincer thesis ("Sub-$50M Challenger vs. Regularized Giant") | UNSUPPORTED as phrased | `grep -rln pincer` → 0 files; adjacent wedge/moat theses graded instead (red-19:9-26) | strategy narration w/o label referent |
| SHM boundary (Darwin-vs-Linux) | UNSUPPORTED | zero shm_open anywhere in crates/; real boundary is spill-path registry, sovereign.rs:848 (red-13:20-25) | platform-claim w/o object |
| Zero-headcount / SRE-and-consultant excision | UNSUPPORTED | zero referents; nearest claims self-scope AWAY from headcount (red-16:7-33, :51-55) | staffing thesis the docs themselves disclaim |
| $50M universal safe harbor (LICENSE.md:13) | MISPLACED/ALIVE but not in instrument | threshold lives in TTL layer 34-p28-sil-compliance.md:70-83 + code param; instrument §2.2 delegates to statutory tests (red-04:28-50) | pointer-file overclaim vs instrument |
| "Hardware-attested execution pipelines" (12-ard-prd-cgap.md:31) | UNSUPPORTED referent in crates/ | no TEE/TrustZone/SGX/OTP/measured-boot hits (red-18:100-102) | charter security claim w/o substrate |
| `ΔS + Φ ≡ R (mod Σ_PQC)` receipt law | UNSUPPORTED | Σ_PQC prose-only, 7 docs, zero Rust (red-13:10-16; audit/cgapprd-02-equation-grounding.md:9-16) | formula w/o implementation |
| "90% cloud-cost reduction" / "99% data-reduction" | UNSUPPORTED | zero cost/data-reduction rows anywhere (red-17:9-17) | economics narration w/o row |

Consolidated UNSUPPORTED count: 13 distinct narrated targets across 8
lanes (RED-02, 04, 05, 06, 08, 10, 13, 15, 16, 17, 18, 19).

## 3. Survivors / immunities proven (what the red team could NOT crack)

| Surface | Receipt | What held |
|---|---|---|
| Zero-alloc sentry (ring/demux/pkt-line) | red-09:64-71 | 15-case adversarial battery + 1M-flood class probes: allocs=0 bytes=0 under open window; positive control flips (non-vacuous). Two honest gaps logged: post-Done poison-tail silent absorption (pack_demux.rs:316) and no surplus-object check (red-09:84-99) |
| Quiet-host ingress latency receipt | red-10:41 | 10.20/10.30 ns/frame reproduced within 4.5%, stated method |
| FFI semantic surface | red-03:113-122 | Byte-identity, spoliation flip, BN3 golden vectors, digest chain all crack a stub — "rewire the WIRE in <48h; cannot rewire the PROOF" |
| Artifact provenance | red-07:55-58 | Vendored nanoarrow (license+NOTICE+git pin 65d2067), line-level arrow citations, Cargo.lock, ggen replay chain, design-around register — 11/45 subpoenas DEFEATED |
| TARGET self-labeling discipline | red-17:102-104 | 02-pareto-levers.md:9-10 and 11-measured-claims.md survive — the gated docs are honest; only the compendium violates the exclusivity law |
| Kernel fail-closed brake non-amplification | red-14:44-54 | 2,000,001 unauthenticated trips at ~0.54 ns/refusal leave ring byte-exact and admitting; no latch exists to hold down |
| bn3 typed bound refusals | red-12:84-87 | Capacities are hard consts exceeded only by loud typed refusal (noalloc.rs:5-7) — no silent overflow inside bn3; gate's anti-vacuity mutant flips |
| P28 logical fail-closedness | red-18:60-64 | One-bit attestation mutation → 0x5032_38FF (trap.rs:34-38); survives at the logic layer (fails only at electrical/glitch layer) |
| Deterrence moat (payment/retention stage) | red-19:111-117 | ASC 450/SOX-embedded licensing internally consistent for already-adopted deployers |
| Receipt-chain unit behavior | red-13:5-7 | RingReceiptChain courts green (5 passed) — the defect is wiring, not the chain primitive |

## 4. Post-breach hardening roadmap (prioritized; concrete edits named)

Priority = exploitability x blast radius from §1. Each item names the
owner surface, the exact edit, and the gate/court that must flip to
prove the fix non-vacuously. No item in this roadmap has been
implemented by this lane (read-only); these are named edits for owner
lanes, each requiring its own falsifier.

### P1 — License rewrite (blocks S1 breaches B1-B4, B18)

Owner: LICENSE-P28-SIL.md (single instrument, 291 lines).

1. **Revenue-aggregation clause** (kills B2). Add a Controlled-Group
   definition (>50% vote/value ownership chain, ASC 810 consolidation,
   IRS § 52/1563 analog) and test every §2.2 statutory trigger at the
   GROUP level, each member jointly classified. Import the $50M figure
   into the instrument's operative text or demote LICENSE.md:13 to a
   cross-reference — the instrument currently prices nothing
   (red-04:159-172 names the four drafting fixes).
2. **Covenant-vs-condition repair** (kills B1's *Ma* fork). Either
   (a) convert §2.2 triggers into genuine scope-of-grant conditions, or
   (b) concede covenant status and drop the copyright-enforcement
   framing; drop the speech-voiding §3.2 predicate over communications
   (misuse per se, red-05:59-77) and replace with an act-based
   circumvention definition (modification/strip/reroute of receipts —
   which §3.2 already contains, red-04:127-131).
3. **Evidentiary/penalty de-stipulation** (kills B1 Counts on §3.3/3.4,
   §4.1). Delete the irrebuttable-adverse-inference and conclusive-
   receipt stipulations (LICENSE-P28-SIL.md:133-147) — FRE 902
   authenticates, it does not anoint; delete the peak-capacity-damage
   integral (:141) in favor of a real tariff schedule; delete ASC 450
   self-declaration (:155) — accrual is the auditor's judgment, not a
   contract term (red-06:47-68). Add a demand/invoice/notice/cure
   mechanism: the instrument has no assertion pathway at all, which is
   itself D1 (red-06:61-66).
4. **Anti-consortium capture** (kills B4). Define "controlled" and
   "cross-border holding conglomerate" (currently undefined, red-04:97-101);
   add a licensee-side information-sharing/confidentiality covenant or
   explicitly accept consortium neutrality on paper.
5. **Provenance armor** (kills B18): NOTICE/THIRD-PARTY file
   consolidating nanoarrow/BLAKE3/wraps (red-07:202-203); clean-room
   protocol doc for cgap-blake3 naming implementer/spec-version/access
   boundary (closes red-07 subpoenas 16-21); contributor CLA + IP-
   assignment register; AI-assistance provenance policy; litigation-hold
   runbook (red-07:202-209 — five edits, each flipping 25/45 battery
   items to DEFEATED).

### P2 — Cryptographic / attestation armor (blocks B5)

Owner: crates/cgap-p28/src/attestation.rs (80-B layout is a frozen
byte-pin — CLAUDE.md pin law; extend, do not reflow).

1. Add an asymmetric signature over the 80-B block (Ed25519 or P-256),
   public key committed OUT of band (OTP-fused or cert-chain), keeping
   the 80-B layout backward-compatible via the existing reserved bytes
   or a v2 magic — validate() (attestation.rs:271-319) currently binds
   BLAKE3 only, clone-everything (red-18:80-91).
2. Name the hardware root of trust or delete "hardware-attested" from
   12-ard-prd-cgap.md:31 (UNSUPPORTED referent, red-18:100-106) —
   a charter security claim with zero crates/ referent is a legal
   exhibit against the licensor in the B1 litigation posture.
3. Glitch-layer posture: document the trust boundary explicitly
   ("logical fail-closedness only; no electrical-layer countermeasures
   claimed") rather than implying enclave resistance (red-18:66-73).

### P3 — Code fixes (blocks B6, B7)

1. **FA9 canonicalize/inode keying** (kills B6). `claim()`
   (sovereign.rs:881) must key ownership on `fs::canonicalize` or
   dev+inode at grant time, or deny non-canonical path text; the
   grant path's File::create (sovereign.rs:768-770) must be opened
   O_WRONLY-without-TRUNC or verified-still-valid immediately before
   drain. Gate: extend fa9_commit_gating.rs with a path-alias arm
   (dir/./x vs dir/x) asserting the owner's spill bytes survive —
   the current court races only byte-identical paths
   (red-13:49-53, court-vacuity hole).
2. **ref_store ceiling inside the RMW** (kills B7). Move ceiling
   enforcement into a single `fetch_update` with `c < bound` guard,
   refuse on None (repair shape named by red-11:96-103); require/
   supply an interrupt-safe `__atomic_*` runtime on rv32imc or gate
   the register reentrancy-unsafe until one exists. Gate: add
   preemption-reentrancy arms to tests/ref_store_court.rs (currently
   zero coverage, red-11:86-93) incl. an impl16/implcell compile leg
   (both arms never compiled by any court).
3. **Chain/commit wiring** (closes red-13 §3). Wire RingReceiptChain
   into cgap_commit_parquet (~sovereign.rs:740-825) — a commit stream
   currently mints ZERO receipts, making non-repudiation advisory;
   implement or delete the receipt_chain.rs:111-119 cross-check doc
   claim (stated safety property with no code, red-13:67-74).
4. **Sentry hardening** (red-09 F1/F2, low): typed TrailingBytes
   refusal after Phase::Done (pack_demux.rs:316) and an
   objects_seen > declared_objects surplus check at close_record
   (pack_demux.rs:407).
5. **riscv32imc release gate** (red-11:34-37): the default-features
   build fails E0463 — gate the std feature off for
   riscv32imc-unknown-none-elf in cgap-git's Cargo.toml or add a
   --no-default-features leg to the silicon gate so the no_std claim
   is tested the way it ships.

### P4 — Gate subject-list closure (blocks B8, B10, B12)

1. **verify_cscb_sram.py subject list**: DEPENDENCY_CRATES
   (scripts/verify_cscb_sram.py:51-54) must include cgap-bn3 with a
   topology-shaped subject (a linked ELF embedding NaEngine+NaMatches)
   or the gate must emit a loud per-excluded-crate skip line. As-is
   the 131,072 B envelope claim is true of a 2-crate slice and silent
   about the engine (red-12:29-42, 49.75x breach).
2. **Foreign-subject gate reach**: boundary_gate.py / cfg_homology.sh
   / symbol_gate.py all court the ORIGINAL tree only (red-02:113-121).
   Either (a) publish the courts as an external conformance kit
   (a challenger can run the repo's own byte-identity/spoliation/
   golden-vector courts against any provider — red-03:113-122 shows
   that is the actual enforcement mechanism) or (b) delete the AHDCLT
   Pillar-2 prose. There is no third option: prose pillars with no
   mechanism are the wave's most repeated defect class.
3. **Constant-time claim**: either add a measured dudect-style
   interleaved-timing row to 11-measured-claims.md with method+receipt
   or strike "constant-time" from 12-ard-prd-cgap.md:49 /
   audit/cgapprd-prd-v2.md:82 (red-10:16-25). Also scope the latency
   receipts explicitly "quiet host" and add a contention-ratio fence
   (relative, sleep-free baseline per the no-sleep ratio-fence law)
   if a hostile-environment claim is ever wanted.
4. **Compendium §6.3 receipt law** (kills B14): every dollar/percent
   row in REVENUE_GENERATION_COMPENDIUM.md must carry a
   11-measured-claims row or be rewritten as TARGET. Concretely: strike
   or re-scope :308 (>75% core-hour), :311 ($4.5-5.2M/yr + ROI —
   economically false on min-billing truncation, ingestion licensing,
   egress untouched), :309 (30-40% RAM headroom — mechanism mismatch)
   (red-17:97-101 table).
5. **Trip-gate law** (B15, forward): any host/CLI mapping
   andon_exit_class() to process::exit MUST front it with an
   authenticity/rate-limit gate (red-14:66-68). Encode as a
   convention-level rule now (CLAUDE.md-status note) so the future
   integration lane inherits the constraint.

### P5 — Narrative hygiene (blocks B13, B17, B16, UNSUPPORTED register)

1. Runtime-ops seam (red-16 §4): no_std stats block exported via the
   C-ABI (closes A1/A6 detection), a DR restore court over the real
   A05 WAL framing (closes A2), a fleet-ops register or explicit
   "ops burden transferred to deployer" clause in the PRD (closes
   A3/A4/A5).
2. Adoption-funnel honesty: scope the 00-ard-prd.md:14-16 "obsoletes
   it" posture to the high-assurance segment (the only funnel the
   on-disk model defends, red-19:118-126) or add a mainstream-procurement
   artifact (SOC 2 posture plan) — the repo currently concedes the
   calendar objection in its own compliance model.
3. PII: if any privacy claim is ever narrated over cgap-git, it must
   address unsalted-BLAKE3 dictionary reversibility first
   (red-15:44-51); until then, keep the absence honest — no claim
   exists, so nothing to breach.
4. UNSUPPORTED register as a standing artifact: the 13-row table in
   §2 should be tracked (e.g. in a KNOWN_UNSUPPORTED.md) so external
   narration is checked against it before repetition — the wave's
   systemic finding is that narration outruns the tree, and the tree
   is mostly honest about it.

## 5. Standing

ALIVE (synthesis lane): all 19 receipts read from disk this session;
consolidated counts re-derived from the receipts (18 breach rows,
13 UNSUPPORTED narrated targets, 10 survivor surfaces). No new
execution claims made here — every verdict cites its owning receipt,
where the command+exit lives. Report terminus:

## 6. ERRATA (appended 2026-11-02, lane F-04; §2 row 75 historical text unchanged)

Register row 75 ("SHM boundary (Darwin-vs-Linux) | UNSUPPORTED | zero
shm_open in crates/; red-13:20-25") is REFUTED-PARTIALLY:
`crates/cgap-beam/src/shm.rs` (mtime 2026-10-31, predating the report)
carries 7 `shm_open` hits with a Darwin caveat at shm.rs:47 — the SHM
object the row declared absent EXISTS in cgap-beam. The row's verdict
becomes REFUTED-PARTIALLY (platform claim has a partial on-disk
referent; not fully SUPPORTED either — Darwin-vs-Linux split remains
unmeasured). The Σ_PQC prose-only finding (row 79) STANDS unchanged.
Consolidated count: 12 verified + 1 refuted row. Finding receipt:
`docs/jira/v26.11.2/redteam/fix-19-corpus-audit.md` (F1, §5).

```json
{"lane":"RED-20",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/redteam/red-20-hardened-survivor-synthesis.md"],
 "commands":["ls + wc -l over docs/jira/v26.11.2/redteam/ (19 files, exits 0)",
  "Read all 19 receipts in full (this session)"],
 "courts_run":["none executable — read-only synthesis lane; every consolidated verdict cites its owning receipt's command+exit"],
 "standing":"ALIVE",
 "divergences":["coordinator rollup corrections: RED-10 breach is slowest-sample under combined contention (fastest column survived cpu-only, -3.4%); RED-11 default-features riscv32imc build fails E0463 (no_std claim holds only with --no-default-features)"]}
```
