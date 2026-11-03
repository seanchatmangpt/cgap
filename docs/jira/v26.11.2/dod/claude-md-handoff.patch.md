# CLAUDE.md coordinator hand-off patch kit (DOD-15)

Status: **AWAITING-COORDINATOR-LANDING**. CLAUDE.md is coordinator-owned;
this lane verified every before-line against the on-disk bytes of
`/Users/sac/cgap/CLAUDE.md` this session (reads + `cargo metadata
--no-deps` exit 0, 23 packages). Apply hunks in order; each hunk lists
its receipt cite. Supersedes nothing in-flight; no other lane owns
CLAUDE.md hunks in this kit.

Sources of truth:
- `docs/jira/v26.11.2/audit/pub-02-flag-table-rederivation.md` (PUB-02b,
  23 members, 20 true / 3 false, registry census)
- `docs/jira/v26.11.2/01-release-gates.md:313-324` (refreshed table,
  23/23 explicit, "12/23" superseded verbatim-noted)
- this session: `cargo metadata --no-deps` = 23 packages (exit 0)

---

## H1 — Publish-law flag paragraph (mandatory)

Cite: `audit/pub-02-flag-table-rederivation.md:22-25,119-120`;
`01-release-gates.md:313-324`.

BEFORE (CLAUDE.md lines ~41-47, exact):

```
  12/23 members declare an explicit flag (9 true, 3 false); the other
  11 default to publishable. Prior "19/22 published" was UNVERIFIED
  registry state — guarded table at
  `docs/jira/v26.11.2/01-release-gates.md:28-32`.
```

AFTER:

```
  23/23 members declare an explicit flag (20 true, 3 false) as of
  2026-11-02 (01-release-gates.md refreshed table; audit/
  pub-02-flag-table-rederivation.md). Prior "19/22 published" was
  UNVERIFIED registry state — guarded table at
  `docs/jira/v26.11.2/01-release-gates.md:28-32`.
```

Note: the preceding sentence "gate/p28/crypto/branchless carry
`publish = true` explicitly (branchless flipped to true by lane
DOD-02, 2026-11-02: first-publish candidate,
`lanes/lane-dod-02-branchless-pub.md`)" remains accurate under PUB-02b
(branchless is flag-true on disk, `crates/cgap-branchless/Cargo.toml:9`,
registry 404 = first-publish pending). No edit required to that
parenthetical; it stays as-is.

## H2 — Layout member-count figure (mandatory)

Cite: PUB-02b W1 (`audit/pub-02-flag-table-rederivation.md:12-18`);
this session `cargo metadata --no-deps` = 23 (exit 0).

BEFORE (CLAUDE.md line 64, exact):

```
- `crates/cgap-verification` — on disk, NOT in the 22-member `cargo metadata` list
```

AFTER:

```
- `crates/cgap-verification` — on disk, NOT in the 23-member `cargo metadata` list
```

Note: PUB-02b confirms the wave added no member; the prior "22" was
drift (fibo/cloud under `ontologies/` are members and were missed). The
header line 3 "23-crate root workspace" is already correct — no edit.

## H3 — "18 crates on crates.io" header figure (recommended)

Cite: PUB-02b W4 (`audit/pub-02-flag-table-rederivation.md:70-76`):
19 of 20 publish-intended member names exist on crates.io; the one
absent is cgap-branchless (404, publish pending).

BEFORE (CLAUDE.md line 4, exact):

```
workspace at 26.11.1, 18 crates on crates.io (`bench` there is a name
```

AFTER:

```
workspace at 26.11.1, 19 of 20 publish-intended member crates exist on
crates.io (`bench` there is a name
```

Note: the old "18" counted existing cgap-prefixed member crates only
(18 = 23 minus branchless/beam/bench/integration/court); it was not
wrong so much as a different denominator. The PUB-02b framing
(19-of-20 publish-intended) is the register the release gates now use.
Line-wrap after this hunk: the following line reads
"collision, not ours; ...)" and still parses after the edit.

## H4 — Sweeps with NO hunk emitted (loud non-edits)

- "91 courts → 95": no numeric court count appears anywhere in
  CLAUDE.md on disk (grep for 91/95: zero hits in a court-count
  context). No hunk; the dispatch premise does not match the bytes.
- Gate exit codes (0/1/2/3, `A skip is loud...` block): match session
  truth; no drift found. No hunk.
- CLI section, command examples: verified current; no hunk.
- wasm-footprint (non-member, publish=true, registry UNVERIFIED,
  PUB-02b:121-123,141-142): candidate for a future Publish-law note;
  not emitted here because the coordinator may prefer the refreshed
  01-release-gates.md table as the sole carrier. Coordinator's call.

---

## Fences (must-not-regress)

- CLAUDE.md is coordinator-owned; this lane wrote ONLY the kit file.
- H3's before-line straddles a wrapped sentence; applying H1/H2/H3 in
  order is safe (disjoint lines 4 / 41-47 / 64).
- Kit is prose hunks, not a `git apply` patch — coordinator copies the
  AFTER blocks verbatim (or asks a follow-up lane to emit a real
  unified diff against the then-current bytes).

## H5 — Publish-law guarded-table pointer (KIT-01, mandatory)

Cite: this session — `sed -n '40,50p' CLAUDE.md` shows the stale
pointer; `grep -n "Member table" docs/jira/v26.11.2/01-release-gates.md`
= line 325 (`### Member table (23/23 explicit: 20 true / 3 false)`);
`:323` carries the "this table governs" errata note. Dispatch said
":327"; disk says :325 (divergence recorded, disk wins).

BEFORE (CLAUDE.md line 48, exact):

```
  `docs/jira/v26.11.2/01-release-gates.md:28-32`.
```

AFTER:

```
  `docs/jira/v26.11.2/01-release-gates.md:325ff`.
```

Note: line 28-32 of 01-release-gates.md is the OLD G1-PUBLISH court
(19/22 era); the guarded refreshed table is at :325ff with the
verbatim-noted stale text above it. H1 rewrites the same paragraph's
count lines; applying H1 then H5 is safe (disjoint lines within the
paragraph).

## H6 — riscv opt-in command (KIT-01, loud non-edit)

Dispatch asked for an update if CLAUDE.md carries the single-flag riscv
form anywhere. `grep -rn "no-default-features\|implcell_critical" CLAUDE.md`
→ zero hits. CLAUDE.md's only riscv mentions (lines 102-103) are the
llvm-tools pointer and footprint-truth note, neither carrying a build
command. NO HUNK — premise does not match bytes; FS-14's flag stays
unrepresented in CLAUDE.md (correctly, per its absence).

## H7 — Full-figure sweep (KIT-01): one loud non-edit

- CLI section: regen command text at CLAUDE.md:22 and the generated
  artifacts table row (:75) both read
  `cd crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py`
  — verified current against on-disk bytes this session. No hunk.
- "A01-A20 specs + 152 receipts" (CLAUDE.md:66): `ls
  docs/jira/v26.11.1/lanes docs/jira/v26.11.1/waves | grep -c '\.md$'`
  = 175 .md files across both dirs this session — but the denominator
  of "152" (receipts vs specs vs files) is not recoverable from the
  sentence, so I cannot prove drift, only that the on-disk .md count
  is 175. Recorded as observation, NO hunk (ungrounded either way).
- Gate exit codes block, two-seed pin, frozen byte-pins, status-code
  hexes: match session truth (read this session). No hunk.
- Remaining header figures already covered by H1-H3; no duplicates.

---

## Standing (KIT-01 additions)

ALIVE for H5's before-line (CLAUDE.md:48 read this session) and the
:325 target line (grep on 01-release-gates.md this session). H6/H7 are
loud non-edits with the greps named above. No git commands run; only
the kit file written.

## Standing (DOD-15 original, retained)

ALIVE for every before-line (each read from `/Users/sac/cgap/CLAUDE.md`
on disk this session) and for the 23-member count (`cargo metadata
--no-deps`, exit 0, this session). Registry figures are carried from
PUB-02b (its curls ran 2026-11-02, this session date); not re-run here.
Kit: AWAITING-COORDINATOR-LANDING.

## LANDED (coordinator delegation, 2026-11-02)

LANDED by coordinator delegation 2026-11-02 via lane CLAUDE-05
(contract: owner-delegated CLAUDE.md landing; no git commands).
H1+H5 applied as one merged edit (same paragraph, H1 count lines +
H5 pointer line); H2, H3 applied verbatim. Post-apply sweep:
`grep -n "12/23\|22-member\|18 crates\|28-32" /Users/sac/cgap/CLAUDE.md`
= zero hits (exit 1). Before-lines verified byte-exact against disk
prior to apply; no drift, no adaptation needed.
