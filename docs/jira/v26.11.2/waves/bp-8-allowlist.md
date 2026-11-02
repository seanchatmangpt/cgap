# BP-8 Local Allowlist Receipt

Lane BP-8 LOCAL-ALLOWLIST, /Users/sac/cgap, 2026-11-01.

## Files written

- `.claude/settings.local.json` (38 rules, valid JSON)
- `docs/jira/v26.11.2/waves/bp-8-allowlist.md` (this receipt)

## Allowlist with witnessed evidence per family

| Family (rule) | Witnessed cite |
|---|---|
| `cargo build:*` | sm-1-root (`cargo build --workspace`, `--release --target`); a01-finish-receipt |
| `cargo test:*` | v2-07-census (`cargo test -p cgap-court`); a01-finish-receipt (host, shared tree) |
| `cargo metadata:*`, `--no-deps` | sm-1-root (`cargo metadata --no-deps --format-version 1`) |
| `cargo check:*` | settings.json Stop hook (`cargo check -p cgap-blake3 -p cgap-core -q`) |
| `cargo bench --no-run:*` | a01-finish-receipt; v2-07-census (`cargo bench -p bench --no-run`) |
| `cargo tree:*` | v2-07-census (`cargo tree`) |
| `cargo search:*` | v2-07-census (`cargo search divan`, `cargo search datafusion`) |
| `cargo pkgid:*` | v2-07-census (`cargo pkgid -p cgap-blake3/core/crypto/gate/storage`) |
| `CARGO_TARGET_DIR=* cargo {build,test,check,bench --no-run}:*` | lane build-root lease pattern, a01-finish-receipt; sm-1-root release+target builds |
| `ggen sync run`, `just doctor`, `just status`, `manifest_patch.py` | preserved from pre-existing settings.local.json |
| `python3 scripts/*` + explicit 4 gates | boundary_gate.py / zero_string_gate.py (CLAUDE.md commands); |
| `projection_coverage.py` / `hop_ratio.py` | v2-13-commit-manifest:40,55,82 (4-script set) |
| `curl -A cgap-publish-receipt https://crates.io/api/v1/*` | sm-1-root:23 (crates.io census) |
| `curl -A cgap-publish-receipt https://index.crates.io/*` | pre-existing settings.json allow entry |
| `ls/grep/find/cat/wc/awk` | v2-07-census read-only census commands |
| `shasum -a 256` | a01-finish-receipt (sha256-exact restore courts) |
| `git status/log/diff/rev-parse` | v2-13-commit-manifest (`git status --porcelain`); read-only git allowed by contract |
| `python3 -c "import shutil; shutil.rmtree('/tmp/:*` | v2-10-cli-state (`shutil.rmtree('/tmp/v210-target')` — build-root lease cleanup law) |

## Sanitization (contract NEVER-list)

Pre-existing file contained `Bash(cargo publish:*)`. The contract's
NEVER-list is absolute, so it was removed rather than preserved. The two
remaining `publish`-substring rules are the curl user-agent
`cgap-publish-receipt` (GET-only, host-scoped), not a publish path.

## Validation

`python3 json.load` -> VALID, 38 rules, no `Bash(cargo publish` rule
(exit 0, this session).

## Divergences

- `.claude/settings.local.json` is untracked but NOT in `.gitignore`
  (git ls-files: empty; .gitignore: no entry). Fixing .gitignore is
  outside this lane's lease; left to the coordinator.
- Env-prefixed rules (`CARGO_TARGET_DIR=* cargo ...`) use a mid-rule
  glob; if the matcher does not support it those rules are inert and
  those invocations prompt (fail-safe, not fail-open).
