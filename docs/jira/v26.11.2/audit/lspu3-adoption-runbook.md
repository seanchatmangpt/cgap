# lspu3-adoption-runbook

Consuming branchless-lsp from cgap/CI (LSPU-3, 2026-11-02).
[RUN] = executed this session with real exit; [DOC] = read from source.

## 1. Gate script real contract

scripts/gate-branchless-lsp.py: NO argv/--help at all (main :100-148);
knobs are env only: BRANCHLESS_LSP_GATE_CLIENT_DIR (default <root>/python),
BRANCHLESS_LSP_GATE_TIMEOUT (default 120). Scans `git diff --cached
--name-only --diff-filter=ACM` filtered *.rs (:35-41) — staged-only;
arbitrary-.rs support lives in lsp_client.branchless_client.
diagnostics_for_files(paths) (branchless_client.py:374-431).
Exits: 0 pass / 1 ANDON_REFUSAL / 3 loud [SKIP] (never fake pass);
proof json at .cgap/receipts/branchless/admission-<ts>.json (:84-97).
[RUN] from cgap: exit 3 "[SKIP] client missing" (correct loud behavior).
Import quirk: `from lsp_client import diagnostics_for_files` raises —
reach it via lsp_client.branchless_client (gate falls back :64-73).
Transitive dep: a built server binary (BRANCHLESS_LSP_BIN or explicit
server_bin; else LspError, branchless_client.py:340-344,388-390).
[RUN] smoke: diagnostics_for_files([main.rs]) -> {'n_refused': 0}, exit 0.

## 2. Server lifecycle

[RUN] staleness: BOTH binaries currently postdate newest src
(release 13:44 vs lib.rs 12:56) — the receipts' staleness claim must be
re-checked at adoption time, not trusted:
stat -f '%m %N' target/release/branchless-lsp crates/branchless-lsp/src/*.rs
Newest line must be the binary; else rebuild [DOC]: cargo build
--release -p branchless-lsp.
[RUN] staleness literal: strings target/release/branchless-lsp |
grep -c ANDON_REFUSED -> 3 (0 after rebuild = wrong/stale artifact).
Lifecycle [DOC]: bare stdio JSON-RPC loop (main.rs:11-17), no daemon;
client asyncio Content-Length framing (branchless_client.py:52-301).
Cleanup: shutdown -> exit -> wait bounded -> proc.kill() SIGKILL on
timeout; kill the process GROUP (orphaned rustc --emit=obj children).
install-gate-hook.sh: idempotent, chains existing pre-commit (:23-41).

## 3. Minimal client sequence (exact shapes)

1. initialize (lib.rs:147-153, plain InitializeParams).
2. didOpen {textDocument:{uri,languageId,version,text}} -> publish #1
   Tier-1 advisory (empty today; Tier-1 regex \b\B never matches,
   rule_pack.rs:50).
3. didSave {textDocument:{uri}, text} — include `text` explicitly
   (lib.rs:174-181 falls back to index; never-seen doc silently
   ignored, :179-181). -> real rustc + disasm court -> publish #2
   Tier-2 authoritative (lib.rs:191-194).
4. MASKING BUG: wait_for_diagnostics returns the FIRST publish
   (branchless_client.py:263-273) = stale Tier-1. External harnesses
   must use the two-read pattern: drain publish #1 after didOpen,
   consume publish #2 after didSave (:404-410).
5. max/conformanceVector: Option<SnapshotId> => params must be JSON
   null; {} deserializes as -32602 (quirk documented
   branchless_client.py:275-279). Rule: None => null, never {}.
   Returns vector: admitted ["BRANCHLESS_ADMITTED"]?, refused
   ["ANDON_REFUSED"]?, score 100*(adm+res)/total | null, strict_mode
   true (lib.rs:94-115,:204-216).
Then shutdown -> exit (branchless_client.py:281-301).

## 4. Packaging recommendation (Option A)

Precedent [RUN-read]: smalt ggen.toml:28-31 = [packs] path-pin table;
no ggen.toml exists in lsp-max.
- A (recommended): cgap pins the gate script by absolute path + env
  pins (BRANCHLESS_LSP_GATE_CLIENT_DIR=/Users/sac/lsp-max/python,
  BRANCHLESS_LSP_BIN=/Users/sac/lsp-max/target/release/branchless-lsp),
  install-gate-hook.sh once. Zero build coupling.
  Falsifier: staged clean .rs -> exit 0 + proof json; flip
  BRANCHLESS_LSP_BIN to /nonexistent -> exit 3 (pin is live).
- B (only for in-process ConformanceVector): workspace path dep
  branchless-lsp = { path = ".../crates/branchless-lsp" }. Falsifier:
  cargo tree -i branchless-lsp resolves; broken pin fails cargo check.
- C: marketplace pack copy of smalt pattern. Falsifier: ggen sync
  resolves; removing dir fails loudly.

## See Also

audit/lspu1-consume-plan.md · audit/lspu2-capability-inventory.md
