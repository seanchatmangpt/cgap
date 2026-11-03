# Retained Evidence Ledger — /tmp cold-artifact retention + repo hygiene

Lane: DOD-14, 2026-11-02, 19:35–19:41 PST. Contract: one owned file
(`docs/jira/v26.11.2/dod/retained-evidence-ledger.md`); /tmp deletions of
S-14-class cold build roots only; no git writes. Every sha256 below was
computed this session with `shasum -a 256` against the exact on-disk bytes.

## Rationale

A host reboot destroys `/tmp` irrecoverably. These artifacts are the
replay evidence for closed red-team courts (ADV-02 C-client ABI probes,
ADV-11 rv32imc disassembly) and the DF-4 Cpk anti-vacuity injection
study. Decision for ALL retained artifacts: **digest-recorded** — the
sha256 + size + content summary recorded here permanently preserves
audit reproducibility after /tmp eviction; the bytes themselves stay in
/tmp until the OS reclaims them, with no further lane action needed.

## /tmp/adv02/ — ADV-02 C-client ABI probe (dir manifest)

What it proves: a real C client (`client.c` → 33,752-byte `client`
binary) and a header-mutated variant (`client_mut.c` vs `cgap_mut.h`)
compiled and run against the FFI, plus a Go cross-language probe
(`go/adv02go`), demonstrating the C-ABI surface admits the real client
and refuses/kills the mutated-header variant (`mutant_out.txt`).

| file | bytes | sha256 |
|---|---|---|
| adv02/cgap_mut.h | 13,127 | 73fcc2812db60049d00c53960e46cfafa5538739eb354cb80a0b67df375365c0 |
| adv02/client | 33,752 | cda301c725dfd9057ae7818fa4e13db09e73e09b85ecba3ef22be15fcd72faf1 |
| adv02/client_mut | 33,752 | a70e52485ddbf1d737e790cc774f05ab139422479ef8c2c4d72bb6cfca0bbe76 |
| adv02/client_mut.c | 3,511 | 4657fba494ab1a933f3d2011c4f4e7a07695e7fa6b9b86e8cd3baa5a28b0c74b |
| adv02/client.c | 3,520 | 45f6e65a786ab38288d40655d3cac7c799966fd50a4dcde176ccb14fed35b35a |
| adv02/go/adv02go | 2,332,306 | f81bbf1bbb06b2c66d60affb77f36d248ad79b754b3c4b5c2a8ff5bb8c9a3d85 |
| adv02/go/go.mod | 24 | 58ce5100dc27ca035843f2ed9402f499539540d827821a6dbbaadac01530b11a |
| adv02/go/main.go | 1,133 | 2ae656b792c9e10b7053b4b9d52a92625687450d04064ac6944a9f4bb4c841b6 |
| adv02/mutant_out.txt | 867 | 5bee592796c963cbe709d5c497a6a7a808814a97651ed84d718a419eccf9a1a9 |

## /tmp/adv11-probe/ — ADV-11 rv32imc probe crate

What it proves: the minimal probe crate (`Cargo.toml` pinning the
riscv32imc toolchain, `src/lib.rs` prologue with volatile/black_box
inputs) used to produce the linked-ELF footprint truth and the
`/tmp/adv11-disasm.txt` disassembly of the rv32imc object. The disasm
(7,418,056 bytes) is the footprint claim's replay evidence.

| file | bytes | sha256 |
|---|---|---|
| adv11-probe/Cargo.lock | 155 | 2f1a281c211345baf14e41ad62b7f03853649ecc33076fbf9a349a9945f8e4e1 |
| adv11-probe/Cargo.toml | 95 | 30b69b8f421df928adb88ee459584bcda1fbdc4888f903294d312dfb154a6826 |
| adv11-probe/rust-toolchain.toml | 541 | 6ae17458edf3ba3caca097b7c0fe63beef47c623225af00a9869fcf2f4333414 |
| adv11-probe/src/lib.rs | 167 | f2eebd35cc2efd3dca18a90d2f7c870165db1ea8ef9c95f9049441b49c13ff2f |
| adv11-disasm.txt | 7,418,056 | e832b6c921481f0877591c736b5f236ee1f46fd50ec9e7844d1ba8369cb93207 |

## /tmp/df4-cpk* — DF-4 Cpk study + anti-vacuity injection

What it proves: CTQ-04/05/06 process-capability study over a 30-window
sample (`df4-cpk.json`, verdict FAIL on CTQ-06/CTQ-04), and the paired
run with anti-vacuity injection forced (`df4-cpk-inject.json`,
`"anti_vacuity_injection": true`, USL/LSL limits moved — verdict still
FAIL on CTQ-06). The pair demonstrates the Cpk gate flips verdicts and
limits under injection: the gate is non-vacuous, the FAIL is a real
measurement, not a fixed table.

| file | bytes | sha256 |
|---|---|---|
| df4-cpk.json | 1,276 | be6cc048143ce9413f85a59d6c1dd863f59396f830dbf60ec642b85fa425053c |
| df4-cpk.log | 936 | 284516fe0262deae1d7933ecd60c476b20bfe2ef41b1067203ab2a18d45d7c1e |
| df4-cpk-inject.json | 1,272 | 9ae85395029c6a14676ec4a1078e5565f0778ecf3c1b5954b1e78d67079a44ce |
| df4-cpk-inject.log | 827 | ae06869647f8148dc6f79259352d58c912bfdf25969e6b3be65e973161509fe2 |

## Hygiene verdicts (repo tree, run 19:36 PST)

- `*.bak` / `*.orig` / `*.rej`: `find` sweep over the whole tree →
  zero matches. CLEAN.
- Stray temp files (`*.tmp`, `*.swp`, `core`, `.DS_Store`, `target-*`):
  two hits, both pre-existing, neither a wave artifact this lane owns:
  1. `/Users/sac/cgap/ontologies/fibo/target-a09` — cold (mtime
     2026-11-01 02:59) cargo build-root lease ~147 MiB INSIDE the
     canonical checkout, gitignored. RETAINED: repo-tree deletion is
     outside this lane's /tmp-only deletion grant; flag for the
     coordinator / `python3 scripts/lease_janitor.py --clean`.
  2. `/Users/sac/cgap/crates/cgap-parquet/.DS_Store` — macOS Finder
     debris, RETAINED (same grant boundary).
- Anchored-hog `pgrep -fl 'if=/dev/zero|red10_hog|stress-ng'` → no
  matches (exit 1). No hogs running.
- Untracked files (`git status --short | grep '^??'`, ~95 paths): all
  are lane receipts (`docs/jira/v26.11.2/lanes/lane-*.md` + `.alive`
  markers), court test files (`crates/*/tests/*_court.rs`), new gate
  scripts (`scripts/{gate_battery,ffi_surface_gate,journal_mutation_matrix,
  conformance_kit_export}.py`), new crates (`cgap-fleet-ops`,
  `cgap-ocel2`), audit/dflss/redteam docs — every path class has a
  matching landing receipt in `docs/jira/v26.11.2/lanes/`. Zero
  unreceipted temp residue detected. `.alive` markers are lane-liveness
  state, coordinator's to clear at drain — RETAINED.

## /tmp wave residue: deleted this session

S-14-class cold cargo build roots (CACHEDIR.TAG fingerprinted, mtime
13:40–13:41, ~6h cold; S-14's receipt authorizes exactly this class,
`lanes/lane-s14-tmp-janitor.md:14-24`). Deleted via
`python3 shutil.rmtree`, each verified absent post-delete (exit 0):

| path | du -sk KB | fingerprint |
|---|---|---|
| /tmp/exp02-target | 100 | CACHEDIR.TAG+package |
| /tmp/exp03-target | 78,180 | CACHEDIR.TAG+debug+tmp |
| /tmp/exp04-target | 7,924 | CACHEDIR.TAG+debug+package |
| /tmp/exp05-target | 12,968 | CACHEDIR.TAG+release+tmp |
| /tmp/exp06-target | 286,696 | cargo build root |
| /tmp/exp07-target | 34,584 | cargo build root |
| /tmp/exp08-target | 135,828 | cargo build root |
| /tmp/exp09-target | 101,144 | cargo build root |

Freed: ~652 MiB. exp08_a.o / exp08_b.o (small object evidence, ~1.4 KB
total) RETAINED — footprint-comparison evidence class, not build-root.

## /tmp residue: RETAINED (ambiguous or evidence-class)

- All S-14 SKIP-ACTIVE leases (`adv03-target*`, `adv04-target`,
  `r15-target`, `r19-target`, `rc07-target`, `xc01-target`,
  `fix07-sandbox`, `r19`) — now ABSENT (deleted by their owning lanes
  at drain, re-checked 19:38). Nothing to do.
- Cold small logs (adv01/adv03/adv06/adv10fix/adv14/df4-*, exp06-09
  logs, r18-manifest.json, adv10fix-*.rs/orig/sha) — RETAINED: small,
  and several are mutation-court evidence (adv10fix orig/restore sha
  pairs, mutant .rs files). Digest-recorded here only for the four
  evidence families the dispatch named; the long tail remains
  /tmp-transient by design.

## Standing

ALIVE — every digest, size, deletion and pgrep verdict above was
produced by a command run this session (2026-11-02, 19:35–19:41 PST)
against the exact subject, with exits shown in the receipt commands.
No git write commands; zero repo-tree deletions; one owned file
written.

## Errata / concurrent-actor observation (19:40 PST, same session)

During post-write digest verification (19:40) a concurrent actor was
observed LIVE inside /tmp/adv02 and /tmp/adv11-probe: `client` (sha
cda301c7...) was replaced by `client_run` (created 19:39) and
`adv11-probe/src/` was transiently empty — an ADV-02/ADV-11 court
replay in flight. The 18 digests above are the AS-OF-CAPTURE record
(19:35–19:36); they remain the audit-of-truth per the
digest-recorded decision. A replay that changes bytes changes the
live artifact, not this ledger; a future ledger lane should re-capture
against the post-replay state. Same-window deletions by the same
concurrent actor: /tmp/adv03-target* and /tmp/adv04-target (S-14
SKIP-ACTIVE list, re-checked 19:38 — absent).

## Seal-time errata (FS-05, 2026-11-02, ~20:23 PST — replay idle)

Per DOD-14's errata instruction ("a future ledger lane should
re-capture against the post-replay state"), all 18 rows were re-hashed
against live /tmp bytes with `shasum -a 256` after the ADV-02/ADV-11
replay went idle. ERRATA, never rewrite: the rows above are untouched;
this section is the correction of record.

### Per-row verdict (13 MATCH / 0 MISMATCH / 5 ABSENT)

MATCH (13) — live digest identical to the recorded digest, byte count
identical: adv02/cgap_mut.h, adv02/client_mut, adv02/client_mut.c,
adv02/client.c, adv02/go/adv02go, adv02/go/go.mod, adv02/go/main.go,
adv02/mutant_out.txt, adv11-disasm.txt, df4-cpk.json, df4-cpk.log,
df4-cpk-inject.json, df4-cpk-inject.log.

MISMATCH: zero. No live artifact diverges from its recorded digest.

ABSENT (5) — externally-absent, not deleted by this lane:

- `/tmp/adv02/client` (recorded 33,752 B,
  sha256 cda301c725dfd9057ae7818fa4e13db09e73e09b85ecba3ef22be15fcd72faf1)
  — replaced by the replay actor with `client_run` (33,752 B, mtime
  19:39). This is the exact concurrent mutation DOD-14's errata
  witnessed live (client → client_run). Permanent record = the
  recorded digest above; new live artifact digest-recorded here:
  client_run = e694562ed306103081c80aab7ab4faa311dd4865d362c54048e8db726101d900.
- `/tmp/adv11-probe/Cargo.lock`, `Cargo.toml`,
  `rust-toolchain.toml`, `src/lib.rs` — the entire adv11-probe
  directory is GONE (`ls: No such file or directory`, exit 1), removed
  by the replay actor's post-replay cleanup, not by any lane action
  this session. The four recorded digests (rows above) stand as the
  permanent record of the probe crate that produced adv11-disasm.txt
  (which still MATCHES, 7,418,056 B).
- C-09's finding (adv11-probe absent at its sweep) is confirmed and
  superseded by this row-level account: the disasm it backed is still
  present and digest-intact.

### Ledger-content errata (transposed byte counts)

The original rows for the two DF-4 logs transposed their `bytes`
columns; digests were always correct (both verified MATCH above):

- ledger line 67: `df4-cpk.log | 936` → live/true size 827
  (digest 284516fe... intact).
- ledger line 69: `df4-cpk-inject.log | 827` → live/true size 936
  (digest ae068696... intact).

### Family presence status (seal time, ~20:23 PST)

- adv02: PRESENT (8/9 rows live-digest-verified; `client` ABSENT,
  superseded by digest-recorded `client_run`).
- adv11-probe: ABSENT (0/4 rows live; 4 recorded digests permanent).
- adv11-disasm: PRESENT (1/1 MATCH).
- df4-cpk: PRESENT (4/4 MATCH; size-column errata above).

### Seal statement

The 18 digests recorded in this ledger are the PERMANENT audit record
for the four evidence families; /tmp is volatile and every live byte
is evictable at the next reboot with no further lane action. As of
seal time the live state corroborates the ledger: 13/13 present rows
digest-identical, zero digest divergences, one rename (client →
client_run, both digest-recorded) and one directory eviction
(adv11-probe, four digests retained above) both explained by the
DOD-14-witnessed concurrent replay. Ledger sealed.
