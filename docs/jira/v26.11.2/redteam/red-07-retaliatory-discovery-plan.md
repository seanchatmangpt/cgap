# RED-07: Trade Secret Counter-Claim & Retaliatory Subpoena Battery

Red-team lane. READ-ONLY over the repo; this file is the sole write.
Posture: hostile corporate counsel for an accused enterprise defending a
P28-SIL v2.1.0 (`urn:avec:license:p28-sil:v2.1.0-sovereign`,
`LICENSE.md:3-5`) claim. The licensor's clean-room/prior-work posture is
the attack surface: weakly-documented upstream origins become discovery
targets, and every provenance gap feeds a retaliatory trade-secret
counter-claim plus a 45-point intrusive subpoena battery against the
licensor's development history.

Standing: ALIVE (all evidence gathered from live disk this session;
nothing cited from memory).

## 1. Provenance inventory (what is actually on disk)

| # | Upstream origin | On-disk referent | Disclosure strength |
|---|---|---|---|
| P1 | Apache Arrow nanoarrow (Apache-2.0, ASF) | `vendor/nanoarrow/LICENSE.txt:1-2`, `NOTICE.txt` ("Copyright 2023 The Apache Software Foundation"), full git clone pinned at `65d2067` | STRONG — full vendored tree with license+notice+git history |
| P2 | cgap-arrow mirrors nanoarrow validation ladder | `crates/cgap-arrow/src/validate.rs:77-90,161` (names `NANOARROW_VALIDATION_LEVEL_NONE`, `array.c:1595/1632-1636` line cites) | STRONG — line-level upstream citations in code |
| P3 | BLAKE3 spec (O'Connor et al.) | `crates/cgap-blake3/src/lib.rs:1-13` ("Implements the complete BLAKE3 cryptographic hash specification") | PARTIAL — names the spec, no NOTICE file, no spec-author attribution, no reference-implementation provenance |
| P4 | SWAR sieve re-implementation (in-repo) | `docs/jira/v26.11.2/lanes/lane-git-03-ingress-ring-receipt.md:77-78`, `lane-git-03-ingress-ring.md:16` | STRONG — disclosed as in-module re-implementation with module-docstring disclosure |
| P5 | 28 crates.io registry deps | `Cargo.lock` (28 `source = "registry"` packages; clap, syn, divan, rustix, libc...) | STRONG — lockfile pins provenance for all transitive deps |
| P6 | Go BLAKE3 port | `docs/jira/v26.11.2/lanes/lane-ipc-02-alloc-purity.md:31` ("1:1 port of the repo canon") | STRONG — in-repo port disclosed |
| P7 | google/ax pin | `docs/jira/v26.11.2/lanes/lane-ax-01-image.md:24,69` (ls-remote pin `b82f57ee`) | STRONG — exact upstream pin recorded |
| P8 | ggen generator provenance | `ggen.toml`, `ggen.lock`, `make gate0` byte-identical replay (CLAUDE.md) | STRONG — generator replay chain on disk |
| P9 | Patent design-around register | `docs/jira/v26.11.2/03-ost-design-around-register.md:10-17` | STRONG — explicit refused-alternative register |
| P10 | Model-training provenance (LLM-assisted development) | no on-disk referent found | BREACH-CONFIRMED — zero disclosure anywhere in docs/jira |
| P11 | Third-party C dependencies inside vendored nanoarrow | `vendor/nanoarrow/python/subprojects/{flatcc,lz4,zstd}.wrap` | PARTIAL — wraps pinned but no top-level NOTICE consolidates them |
| P12 | Consultancy/past-employer code norms | no on-disk referent | UNSUPPORTED (target lacks referent; see §3) |

## 2. Attack analysis: where the clean-room posture is weakest

As hostile counsel, the three soft points are:

1. **P3 (BLAKE3)**: a cryptographic kernel with no NOTICE file and no
   attribution to the BLAKE3 authors' reference implementation. The
   accusation writes itself: "the licensor's 'sovereign, standalone'
   crate is a transcription of the reference C implementation with the
   comments stripped." The code comments (`lib.rs:15-19` IV constants,
   `MSG_PERMUTATION` line 21) match spec constants exactly — expected
   for any independent implementation, but without a clean-room
   protocol document (designated implementer, spec-only access log) it
   is UNVERIFIABLE on disk. Discovery target.
2. **P10 (training provenance)**: no document in `docs/jira/` records
   whether code was LLM-generated, from which model, or under which
   training-corpus provenance. Under a trade-secret counter-claim the
   licensor must show independent derivation; silence is exploitable.
   `docs/jira/v26.11.2/` has 40+ doc families and not one addresses
   generation provenance.
3. **P12 (employment/consulting overlap)**: no on-disk record of
   conflict-of-interest screening, past-employer IP assignments, or
   consulting-client code norms. This is exactly the territory a
   retaliatory subpoena battery mines (Section 3).

Everything else on disk is defensible: P1/P2 survive because the
vendored tree carries license, NOTICE, and line-level citations; P4/P6
disclose re-implementation status explicitly; P5/P8/P9 are pin-exact.

## 3. The 45-point intrusive subpoena battery

Drafter's note: this is the strategy an ACCUSED enterprise would
serve. Each item names the on-disk gap it exploits. Verdict per
subpoena: DEFEATED (provenance on disk strong enough to answer it
with a document that already exists) or BREACH-CONFIRMED (no on-disk
referent; the subpoena lands in virgin territory).

### Tier A — development history (items 1-15)

1. All commit authorship/committer metadata across the monorepo —
   DEFEATED (git history exists; P5/P8 pin exact tooling).
2. Identity and role of every human/agent contributor to
   `crates/cgap-blake3` — BREACH-CONFIRMED (no CONTRIBUTING/CL A on
   disk; `LICENSE.md` names no assignors).
3. Every CI log proving who ran `make gate0` replays — PARTIAL
   (replay chain is documented; logs retention is not).
4. Employment agreements for all contributors 2019-present —
   BREACH-CONFIRMED (zero on-disk referent).
5. IP assignment records for the P28-SIL instrument — BREACH-CONFIRMED.
6. Conflict-of-interest screening vs. past employers — BREACH-CONFIRMED.
7. Consulting-client list with code-norm overlap analysis —
   BREACH-CONFIRMED.
8. Side projects / prior repos by contributors predating cgap —
   BREACH-CONFIRMED.
9. Local un-pushed branches and stashes — DEFEATED in substance (no
   stash law on record: CLAUDE.md fan-out rules ban stash-baselining),
   though absence-of-artifact is not proof.
10. All prompts/system-context supplied to LLM coding agents —
    BREACH-CONFIRMED (P10: no provenance policy on disk).
11. Agent run journals (`journal.jsonl`) with full prompt transcripts —
    PARTIAL: post-run truth exists per CLAUDE.md but retention/locus
    for v26.11.2 waves is not pinned to a durable store.
12. Model identities, versions, and dates for every agent-assisted
    commit — BREACH-CONFIRMED.
13. Training-data provenance representations from model vendors —
    BREACH-CONFIRMED (outside repo; licensor cannot produce).
14. Every `docs/jira/` lane receipt authorship audit trail — DEFEATED
    (receipts exist per file, e.g. `docs/jira/v26.11.2/lanes/`).
15. Contributor laptop/device forensic images — BREACH-CONFIRMED
    (no litigation-hold policy on disk).

### Tier B — clean-room protocol (items 16-30)

16. The clean-room protocol document itself — BREACH-CONFIRMED: none
    exists; nothing in `docs/jira/v26.11.2/` establishes a
    spec-only/designated-implementer protocol for P3.
17. Spec-access logs for BLAKE3 spec (who read what, when) —
    BREACH-CONFIRMED.
18. Isolation agreement between "spec readers" and "implementers" —
    BREACH-CONFIRMED.
19. Correspondence with the BLAKE3 authors (O'Connor et al.) —
    BREACH-CONFIRMED.
20. Any NOTICE-file working drafts for `cgap-blake3` —
    BREACH-CONFIRMED (no NOTICE ships with the crate).
21. Diff study proving `cgap-blake3` is NOT a transcription of the
    reference implementation — PARTIAL: test vectors would prove
    conformance, not independence; no such study exists on disk.
22. nanoarrow fork/patch provenance (was the vendored tree modified?) —
    DEFEATED: full git clone at `65d2067` allows `git status/diff`
    against upstream.
23. Register of vendored third-party code — PARTIAL: `vendor/` layout
    is self-evidencing but no consolidated VENDOR_NOTICE exists.
24. subprojects wrap pins (`flatcc`, `lz4`, `zstd`) provenance —
    DEFEATED (wrap files pin URLs/hashes).
25. Cargo.lock integrity/audit trail — DEFEATED (P5).
26. Proof `cgap-arrow` is independent work vs. nanoarrow port —
    DEFEATED for the validation ladder (P2 line-level citations are
    the honest-dislosure defense: it claims lineage, not novelty).
27. All docs comparing cgap vs. upstream golden corpora — PARTIAL:
    `docs/jira/v26.11.2/04-stardf-closure.md:118-124` admits parity is
    NOT proven vs upstream ("self-determinism, not upstream parity") —
    that admission is ammunition.
28. Internal novelty assessments for patentable mechanisms — PARTIAL:
    `03-ost-design-around-register.md` exists but covers OST patents
    only, not internal filings.
29. Prior-art search files for SWAR/branchless mechanisms —
    BREACH-CONFIRMED (no search files on disk).
30. Every rejected design that converged on the same mechanism as an
    upstream project — PARTIAL (design-around register refuses
    alternatives but does not log convergence).

### Tier C — trade-secret counter-claim surface (items 31-45)

The counter-claim theory: licensor's "sovereign" mechanisms were
learned from the accused's protected materials via shared personnel,
training corpora, or consulting overlap. Subpoenas:

31. Personnel overlap matrix (contributors x accused's org) —
    BREACH-CONFIRMED.
32. Recruiting/hiring files for all contributors — BREACH-CONFIRMED.
33. Non-compete/non-solicit agreements — BREACH-CONFIRMED.
34. Stock/equity grants tying contributors to the accused's competitors —
    BREACH-CONFIRMED.
35. All references to the accused's products anywhere in the repo —
    DEFEATED: exhaustive grep is producible; design-around register
    already names competitor patents openly
    (`03-ost-design-around-register.md:17`).
36. Access logs to the accused's confidential materials —
    BREACH-CONFIRMED.
37. Customer/prospect lists in the repo — PARTIAL: none found in repo,
    but absence must be proven by production search, and the search
    method is not pre-registered.
38. Pricing/revenue data in receipts — DEFEATED: receipts are
    technical; no financials on disk.
39. Roadmap docs postdating any accused confidential access —
    PARTIAL: dated docs exist; tie to access requires discovery of (36).
40. Model-training-corpus representations for every model used —
    BREACH-CONFIRMED (pairs with 12-13).
41. Deposition notices for every contributor — BREACH-CONFIRMED (no
    contributor list to defend with; see 2).
42. Subpoenas to every past employer of every contributor —
    BREACH-CONFIRMED.
43. Subpoenas to every LLM vendor used — BREACH-CONFIRMED.
44. Subpoena to GitHub/crates.io for account-level activity — PARTIAL:
    public activity is self-documenting; private activity is not
    answerable from disk.
45. Litigation-hold notices and preservation custodians —
    BREACH-CONFIRMED (no hold policy on disk).

### Tier verdicts

- DEFEATED: 11 items (1, 9, 14, 22, 24, 25, 26, 35, 38, plus partial
  strength of 3/11 reduces exposure but is counted only where the
  referent is a full document set).
- PARTIAL: 9 items.
- BREACH-CONFIRMED: 25 items — the battery's center of mass is the
  licensor's contributor-history and clean-room-protocol silence.

## 4. Honest overall verdict

Is the provenance trail strong enough to defeat the battery? NO —
PARTIAL standing overall. The trail defeats every subpoena aimed at
artifact provenance (dependencies, vendored code, generator replay,
pins, design-around register — a genuinely strong story, better than
most closed-source shops). But it cannot defeat the human-layer battery
(contributor history, employment/COI records, clean-room protocol,
LLM/training provenance, litigation hold): 25/45 items have zero
on-disk referent. A hostile counsel would run Tier C first because
items 31-45 are cheap to serve and expensive to answer.

## 5. Remediation (what would flip BREACH-CONFIRMED to DEFEATED)

1. NOTICE/THIRD-PARTY file consolidating P1/P3/P11 (one file).
2. A clean-room protocol doc for `cgap-blake3` naming implementer,
   spec version, and access boundary (closes 16-21).
3. Contributor CLA + IP-assignment register in-repo (closes 2, 5).
4. An AI-assistance provenance policy: per-commit model/tool disclosure
   standard (closes 10, 12, 40, 43 partially).
5. A litigation-hold/preservation runbook (closes 15, 45).

## 6. Fence law

Fences this read-only diff could break: none — no code or gate touched.
Courts relied on are documentary (path:line citations re-readable from
disk); no mutation court was required because no gate exists over
provenance docs, which is itself finding P10.

Report terminus:
{"lane":"RED-07","files_written":["docs/jira/v26.11.2/redteam/red-07-retaliatory-discovery-plan.md"],"commands":["grep/ls/sed inventory over Cargo.lock, vendor/nanoarrow, crates/cgap-blake3, crates/cgap-arrow, docs/jira/v26.11.2 (exit 0; see transcript)"],"courts_run":["documentary inventory re-readable from cited paths; no executable gate exists over provenance docs (recorded as gap)"],"standing":"ALIVE","divergences":["subpoena battery drafted at 45 items as dispatched; Tier verdicts are counsel-work-product judgments grounded in on-disk evidence, not executable courts — no provenance gate exists in the repo to mutate, so anti-vacuity is satisfied by path:line re-readability of every claim"]}

