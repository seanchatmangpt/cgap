# OVERNIGHT-PROMPT — W1-W10 dispatch layer (operator-approved 2026-11-01)

Ground-truth charter: OVERNIGHT-LOOP.md + 05-wave-plan.md + 00-ard-prd.md (read first; this
file is the dispatch layer over them). 10 waves x 20 agents, one message per wave.

## Shared contract (prepend verbatim to every agent prompt)

- Repo: /Users/sac/cgap — ONE canonical checkout, branch main. Coordinator owns ALL git
  transitions (commits, pushes, tags); agents never run git state commands, never stash,
  never checkout.
- ABSOLUTE BAN: /Users/sac/stardf is OUT OF SCOPE per operator order 2026-11-01. No reads,
  no writes, no command may touch it. Any lane whose contract mentions stardf executes
  cgap-side only.
- ZERO DataFusion: no datafusion* dep anywhere (scripts/zero_datafusion_gate.py per
  01-release-gates.md G6). cgap-datafusion-removal.md is the citation.
- Frozen pins untouchable: license URN urn:avec:license:p28-sil:v2.1.0-sovereign; v26.10.31
  SHACL IRIs (avec: -> https://avec.chatman.dev/); seeds never conflated (SplitMix64
  0xBF58_476D_1CE4_E5B9 vs corpus 0xA20C_0FFE_E5A7_E000 — bitslice.rs:687-699).
- Chicago courts: real collaborators, final-state asserts, no mocks. Every court names its
  anti-vacuity mutation; mutate /tmp sandbox copies only (sha256 fail-closed restore, bump
  mtimes after restore — a02-finish-receipt.md:170-184). A court that cannot flip is FAIL;
  a mutation-free green gate is VACUOUS (exit nonzero).
- Exit-code contract: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
- Standing vocabulary: ALIVE (witnessed command+exit THIS session) / PARTIAL / BLOCKED
  (file:line + command + exit) / REFUSED / UNSUPPORTED / UNVERIFIED (default; never rounds
  up). Perf numbers are [TARGET-v26.11.2] until Divan-measured with stated method; GB/s is
  never quoted as a multiple of facts/s; every landing-doc claim carries a waves/ file:line
  receipt or [TARGET-v26.11.2].
- No Process.sleep: bounded polls (~30s cadence, stated deadline); ratio fences not
  absolute ms.
- Writes: <50-line chunks or assemble via python; token-sweep verify on-disk bytes after
  every write. Markdown <=100-char lines, H1 = filename, kebab-case.
- Receipts: docs/jira/v26.11.2/waves/<lane-id>-<slug>.md, each ending with JSON
  {"lane","files_written","commands","courts_run","standing","divergences"}.
- Publish law: ONE cargo publish invocation per crate per wave-budget; 429 -> read
  Retry-After, back off, record BLOCKED, never blind-retry; never concurrent with lever
  lanes. dry-run exit 0 (deps live) is the resolution gate before any attempt.
- Build roots: CARGO_TARGET_DIR=/tmp/<lane-id>-target, deleted via python3 shutil.rmtree at
  lane end. No _build/target debris in-tree.
- No worktrees. No mocks. 429/timeout on an agent = loud gap, resume in place via
  SendMessage; never relaunch a lane that has receipts on disk.
- Max ggen projection law: every non-kernel surface (CLI verbs, docs, command-reference,
  boundaries, tests scaffolding) is projected from the RDF graph via ggen — hand-written
  residue lives ONLY in crates/cli/src/custom_handlers.rs (cnv:CustomBehavior seam) and
  kernel hot paths (cgap-{gate,parquet,bn3,ffi,p28} internals). Regen contract per change:
  cd crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py, byte-identical
  replay proved. Packs land in ~/ggen-marketplace/packs/cgap-pack (gates + exact-stem pass/
  fail witnesses + gate-court.toml law). NEVER hand-edit generated files.

## Waves

- **W1 — GGEN PROJECTION FOUNDATION**: ontology audit; CLI regen + two-run sha256 replay;
  root cgap.ttl projection replay; cgap-pack lever extension (per-lever individual + gate +
  witness pair, exact-stem law); doctor extension spec; doc repair wave (HANDOFF completion,
  PRD 4/6 re-point, 05-wave-plan lever-table reconcile vs 02-pareto-levers (02 authoritative),
  OVERNIGHT-LOOP refresh (19/22), RESTART-NOW supersession); write missing numbered docs
  06-bench-protocol / 07-court-corpus / 08-registry-sweep / 09-overnight-mutation-set;
  research/ completeness index (set landed 20/20 this session); release_check.py:21 pin
  repair; waves/ receipt audit (court-auditor); custom_handlers inventory (read-only);
  patent/claims.txt; W1 terminus census.
- **W2 — GATES + G6/G7 MECHANIZATION**: zero_datafusion_gate.py (build + court agents);
  receipt_lint.py; registry_sweep.py (flips A/B); OST design-around evidence greps into
  03-register rows; boundary/zero-string gate re-runs; build+test baseline (compile-abort
  counted separately); L1-L5 court shells in tests/; doctor aggregation; CLI regen;
  cgap-pack witness court end-to-end; adversarial gate audit; hygiene window; W2 terminus.
- **W3 — LEVERS L1-L5** (plans in plans/): L1 two-stage admission escalation (penstock.rs
  FrameView+prefilter8+escalate, <15 ns/frame); L2 SWAR FNV (footer.rs fnv1a64_seeded
  word-wise rewrite, byte-exact equity court, <10 ms @1M); L3 PLNF semi-join + radix probe
  (unify.rs JoinProbeScratch + emit_join_masks, noalloc mirror, differential court N in
  {1,64,4096,100_003}, >20x @1M); L4 ring sink (mmap_writer.rs RingSink impl BlockSink,
  LEASE_EXHAUSTED boundary byte-preserved, <4 us/tx); L5 fused ASC450-into-P28 (asc450.rs
  fused_row, byte-equality fused-vs-unfused, <1.45 us/row); Divan PARETO rollup (delta must
  be in claimed direction); court flip-verification; CLI regen; cgap-pack lever gates;
  W3 terminus.
- **W4 — MAT-BENCH + DATALOG KERNEL**: crates/stardf-datalog NEW cgap-side crate (semi-naive,
  u64 fact IDs, zero interning, no_std from birth); benches/mat_closure.rs (1M/5M/10M buckets,
  facts/s, single-core); benches/dred_churn.rs (reuse cgap-storage/src/dred.rs; 90% churn,
  entropy sweep, ratio fence); mutation courts (drop atom -> differential oracle else VACUOUS);
  02-bench-protocol finalization (RDFox NOT executed line; machine+method template); receipt
  skeletons [TARGET]; no_std probe pair (riscv32imc control + attempt, E0463 typed BLOCKED);
  nm purity census; docs/command-reference regen; W4 terminus.
- **W5 — CLI EXTENSION VIA GGEN**: cnv verify-context noun family ontology (16 schema gates
  must pass); regen + 17-stub UNIMPLEMENTED witness; 4 custom_handlers agents (real
  owner-crate invocations, each body embeds a flip witness); tests/zero_code_cli.rs
  extension; full CLI test verify; registry sweep live leg (cells only, no publishes);
  release_check pin + dry-run resolution matrix; crates.io metadata pre-check (dry-run only);
  command-reference projection; cgap-pack mirror; handler flip audit; compile-freeze warden;
  hygiene window 2; W5 terminus.
- **W6 — PUBLISH WAVE** (strictly after W5 sweep exit 0): single-actuator sequencer
  (dependency-ordered, one invocation/crate, 429 -> Retry-After honored -> BLOCKED,
  stop-on-slide law); 8 crates.io watcher agents (num_versions exactly-once witnesses);
  immutability witness; post-publish sweep updates; /tmp consumer proof crate; README
  reconciliation (archive-before-overwrite + sha256); publish-ledger audit; CLI regen;
  hygiene window 3; W6 terminus (22 targets, not 22 forced successes).
- **W7 — BENCH CHAINS + PARETO CLOSURE**: five chains re-runs on post-lever tree; 3 vacuity
  court-auditors; ratio-fence audit; dead-surface audit; ctq-registry staleness; asm census
  trio (0 blr/0 br xN); flash-size riscv32imc leg (object vs ELF distinguished); SRAM static
  estimate update; position-paper claims table (MEASURED only); comparison-honesty pass
  (LIKE-FOR-LIKE / CATEGORY-ERROR / UNVERIFIABLE-BY-ABSENCE); regen; W7 terminus.
- **W8 — DOC COMPLETION WAVE**: finalize 06-09 with W2-W7 receipts; 00-ard-prd final;
  01-release-gates full receipts; 02-pareto-levers status flips; 03-register rows
  resolved-or-gated; 04-stardf-closure OUT-OF-SCOPE annotations; HANDOFF final;
  OVERNIGHT-LOOP refresh; position-paper-v2.md (MEASURED-only, predecessor archived);
  receipt_lint over whole tree (fix in place, zero allowlist); diataxis regen + idempotence;
  README sweep; docs-as-data checksum ledger; adversarial paper audit; W8 terminus.
- **W9 — ADVERSARIAL VIBRATION**: 6 auditors (sampled re-runs + /tmp court flips, verdicts
  only); 3 repair agents (audited defects only, disjoint ownership); 3 creative-mutation
  sweepers (fresh mutations not in named sets); cross-lane drift hunt; 2 /tmp orphan hunters;
  seed-conflation hunt; frozen-pin tripwire; generated-surface purity; staging census;
  W9 terminus (defect ledger + saturation verdict).
- **W10 — INTEGRATION + TERMINAL RECEIPT** (coordinator-serialized; agents propose,
  coordinator commits): integration-prep x3; final workspace proof x2; final doctor; final
  registry sweep + release_check matrix; G1-G8 gate board; ledger folds x3; HANDOFF final;
  mutation-set flip register (every mutation flipped >= once or loud FAIL row); saturation
  receipt; version-bump PROPOSAL memo; archival x3; cleanup audit (all /tmp leases deleted);
  TERMINUS-V5.

## Sequencing

W1 first -> W2 -> W3 (independent of W4) -> W4 after W2 -> W5 after W3+W4 -> W6 strictly
after W5 sweep exit 0 -> W7 after W3/W4 -> W8 after W6/W7 -> W9 after W8 -> W10 last.
Within every wave: all 20 in ONE message; bounded polls on upstream receipts; no sleeps.
429/timeout = resume in place; never relaunch a lane with receipts on disk.

## Non-vacuity metagate

A wave whose receipts contain zero witnessed mutations or zero command+exit citations is
itself VACUOUS — the terminus agent fails the wave (exit nonzero) and names the offending
lanes.
