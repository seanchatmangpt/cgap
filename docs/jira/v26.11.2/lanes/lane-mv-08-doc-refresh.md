# Lane MV-8 — doc refresh against disk truth (2026-11-02)

Repo /Users/sac/cgap @ main, branch main. Zero git writes performed.
Files written (lane-owned only):

1. `CLAUDE.md` — surgical corrections against disk truth:
   - member count: "22-crate root workspace" -> "23-crate root
     workspace". Evidence: `cargo metadata --no-deps --format-version 1`
     piped count -> 23, exit 0 (21 crates + ontologies/fibo +
     ontologies/cloud; cgap-verification on disk but NOT a member).
   - registry-absent note extended with cgap-branchless.
   - publish-law bullet rewritten to on-disk manifest flags:
     explicit publish=false on members = bench (name collision), beam,
     branchless, court (4); explicit true = cgap, arrow, blake3,
     crypto, git, gpu, p28, parquet (8); 11 members carry no explicit
     flag and default publishable (incl. gate). Evidence:
     `grep -n "publish" crates/*/Cargo.toml` exit 0. Prior "19/22
     published" marked UNVERIFIED registry state.
2. `docs/jira/v26.11.2/waves/w-commitplan-v2.md` — appended
   "## Refresh (2026-11-02)" (original session-verbatim plan intact
   above it). Fresh census: `git status --porcelain` -> 252 entries
   (100 M, 152 ??), exit 0, partitioned into R.1-R.9 groups summing
   to 252 (R.9 debris double-count note on wrapper .o inside the
   branchless dir entry). R.8 per-crate M/?-counts derived with a
   partition script, not by eye.
3. `docs/jira/v26.11.2/HANDOFF.md` — duplicate "## 4" fixed: second
   "## 4. p5-integration addendum" -> "## 5", prior "## 5. Post-CL1
   addendum" -> "## 6", subsections 5.1/5.1a/5.1b/5.2/5.3/5.4 ->
   6.1/6.1a/6.1b/6.2/6.3/6.4, in-text SS5.1a/SS5.1b/SS5.1 refs ->
   SS6.1a/SS6.1b/SS6.1.
4. `docs/jira/v26.11.2/audit/release-blockers.md` — status-line
   appends to R5/R8/R9/R11 rows only (R5 fresh receipt_lint rerun:
   FAIL 445 / 190 docs, script exit 0 advisory; R8 ls -la re-witness;
   R9 CLOSED by this lane; R11 REFRESHED by this lane).

Count table (all freshly derived, exits above):

| Claim | Fresh command | Result |
|---|---|
| workspace members | cargo metadata --no-deps count | 23 |
| members explicit publish=false | grep publish crates/*/Cargo.toml | 4 (bench, beam, branchless, court) |
| members explicit publish=true | same | 8 |
| members default-publishable | same (no flag) | 11 |
| live status | git status --porcelain | wc 252 (100 M / 152 ??) |
| receipt_lint census | python3 scripts/receipt_lint.py | FAIL 445, 190 docs |
| debris | ls -la both files | present, mtimes cited |

Falsifier check: every corrected number above cites its command +
exit run this session; a re-read of each edited file (below)
confirms the bytes landed.

Divergences: (a) dispatch said "SEED-B found gate/p28/crypto
publish=true on disk" — p28/crypto explicit true confirmed; gate
carries NO explicit publish line and DEFAULTS publishable, corrected
accordingly. (b) Prior receipts' 138-entry status is stale; fresh is
252. (c) receipts cited in dispatch for R5 (445) re-confirmed live.

Standing: ALIVE (all commands + exits run this session against the
exact subjects; doc-only lane, no builds owned).
