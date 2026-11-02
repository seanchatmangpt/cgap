# Lane LSPA-5 — branchless-court PostToolUse hook

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Server consumed from
/Users/sac/lsp-max, never edited. No git commands. Files written: exactly
the three lane-owned files.

## Files

- `/Users/sac/cgap/.claude/hooks/branchless-court.sh` (NEW, chmod +x,
  `bash -n` syntax_ok)
- `/Users/sac/cgap/.claude/settings.json` — ONE PostToolUse entry inserted
  at index 2, immediately after the fmt-edited entry (index 1), before the
  write-integrity entry another lane added concurrently (index 3). JSON
  re-parsed post-insert: `python3 json.load` exit 0. Concurrent-lane merge:
  the entry list was re-read fresh immediately before insert; a first
  insert attempt aborted cleanly (assert) when a concurrent lane changed
  the list shape — no clobber.
- `/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-lspa-05-hook.md` (this)

## Hook behavior (as written)

- Skeleton = fmt-edited.sh:8-18: stdin JSON -> `tool_input.file_path` ->
  `*.rs` filter; non-.rs/missing/unreadable -> exit 0 without spawning the
  server.
- Staleness guard: `strings "$BIN" | grep -c ANDON_REFUSED >= 1`, else
  `[SKIP] ... rebuild: cd /Users/sac/lsp-max && cargo build --release -p
  branchless-lsp`, exit 0.
- Debounce: sha256(path)-keyed state file under /tmp/branchless-court-state
  storing the courted mtime; unchanged mtime -> exit 0 (observed live: a
  rerun with state present exited 0 in <1s).
- Server driven via `/Users/sac/lsp-max/python/lsp_client/
  branchless_client.py` `diagnostics_for_files` (one session; drains the
  did_open tier-1 publish so the did_save tier-2 verdict is observed).
- Tier policy: aarch64 always; riscv32imc keyed by the server on path
  (crates/cgap-branchless/). No hook-side tier flag exists; the hook does
  not pass one.
- Non-empty ANDON_REFUSED -> exit 2 with the diagnostic text on stderr.
- Fail-open: any harness failure (transport, timeout, LspError) -> exit 0
  with a loud one-line [SKIP] naming rc. This is a deliberate choice: the
  hook is a convenience court, never a gate — it must not block the
  session on a flaky server (contrast fail-closed gates like
  zero_string_gate.py). Witnessed live: a 60s LspTimeout produced
  `[SKIP] branchless-court: harness failure (rc=1): ...` exit 0.

## Courts run (this session, release binary mtime Nov 2 13:44,
## strings|grep -c ANDON_REFUSED = 3)

| court | command | exit |
|---|---|---|
| clean shared select.rs | hook stdin JSON select.rs | **2** (expected 0 — see divergence D1) |
| mutant /tmp copy select.rs | cmov_u64 body replaced with `if cond { t } else { f }`; shared tree sha256-verified OK post-run | **2** naming ANDON_REFUSED + line 79 |
| non-.rs path | hook stdin notes.md | 0 (<1s, no server spawn) |
| fail-open | minimal /tmp snippet (LspTimeout) | 0 with loud [SKIP] rc=1 |
| debounce | rerun with warm state file | 0 (<1s) |
| settings.json validity | python3 json.load post-insert | 0 (4 PostToolUse entries) |

Full-diagnostic dump (diagnostics_for_files, release binary): shared
select.rs n_refused=12 (lines 80,81,86,179,314,1225 all naming
checked_uniform_select); mutant /tmp copy n_refused=12 with the line set
shifted one line down (79,80,85,...) — the extra line-79 diagnostic is the
injected branch site.

## Divergences and findings

D1 — Falsifier "clean select.rs -> exit 0" FAILS against the current
tree: `checked_uniform_select` (select.rs:80) is branchy BY DESIGN ("this
is where the refusal lives", select.rs:74) but carries no
`#[allow_isel_residual]`, so the Tier-2 court correctly refuses it with
12 ANDON_REFUSED diagnostics. Clean-vs-mutant is therefore NOT
discriminative on this subject (both exit 2). This is a real shared-tree
finding for the cgap-branchless owner: annotate checked_uniform_select
with `#[allow_isel_residual]` (or gate the test module out of the courted
compile). select.rs is NOT a lane-LSPA-5 file — not fixed here. After
that annotation lands, the flip becomes discriminative: clean -> 0
refusals, mutant -> >=1 at the cmov_u64 line.

D2 — Server boundary witnessed: minimal single-fn /tmp snippets (e.g.
bare `#[inline] pub fn cmov_u64`) never receive the did_save tier-2
publish (client times out at skip=1; raw notification probe shows
publish#1 n=0 then silence for 30s). did_save requires
`params.text` or workspace-index content (branchless-lsp lib.rs:227-240);
root cause of the missing publish not root-caused here — out of lane
scope. The hook's fail-open [SKIP] covers this path loudly.

D3 — Concurrent lsp-max edits mid-lane: branchless_client.py changed
under me at 13:54 (wait_for_diagnostics gained `skip=1`, replacing the
WG2 drain pattern described in the G05 receipt) and the release binary
rebuilt at 13:44. All witnesses above were re-run against the 13:44
binary + 13:54 client. The stale-server [SKIP] guard (strings marker
count) was exercised implicitly by pointing BRANCHLESS_LSP_BIN at a
missing path in code review only — UNVERIFIED on the wire; the
binary-missing branch is the same code path (both [SKIP] before spawn).

## Standing

ALIVE for: hook file, settings entry, non-.rs fast path, fail-open path,
debounce, mutant-refusal exit 2, settings JSON validity — all re-run this
session against the 13:44 binary. UNVERIFIED: stale-server [SKIP] on a
real stale (non-missing) binary. The contract falsifier pair is
witnessed at exit-code level but NOT discriminative (D1) until
checked_uniform_select carries #[allow_isel_residual].
