# AF-6: version binding + python binding audit (findings 2+3)
Standing: ALIVE

Lane: AF-6 (IMPLEMENTATION). Date: 2026-11-03. Repo: /Users/sac/cgap.
Never ran git. Files touched: `ontology/cgap.ttl`, `crates/cgap-arrow/src/generated_version.rs` (via `ggen sync run` only), `bindings/python/avec` → `bindings/python/cgap` (rename + symbol sweep).

## Finding 3 — version binding drift (FIXED)

- `ontology/cgap.ttl:14` carried `av:version "26.11.2"` (releaseDate 2026-10-29) against a 26.11.3 workspace — SA-4 fixed cgap-cli.ttl only. Fixed the TTL to `26.11.3` / `2026-11-03`.
- `ggen sync run` from repo root: exit 0, `written: ["crates/cgap-arrow/src/generated_version.rs"]`. Now `VERSION = "26.11.3"`, `VERSION_C = b"26.11.3\0"` (generated_version.rs:5,9).
- Determinism: second `ggen sync run` exit 0, `written: []`.
- `cargo check -p cgap-arrow`: Finished, 0 errors.

## Finding 2 — python binding targeted a nonexistent surface (FIXED, minimal-fix ruling)

**Audit**: the binding lived at `bindings/python/avec`, loaded `libavec_ffi.dylib`, and called `avec_{i32_batch,array_addr,schema_addr,batch_rows,version,digest_of}` — a surface that does not exist anywhere in the workspace. Live truth is `libcgap_ffi.dylib` exporting 12 `cgap_*` symbols (`nm -gU target/debug/libcgap_ffi.dylib`). The binding was broken TODAY (any import-time `load_library()` failed at dlopen), so this is restoration, not new work.

**Ruling: minimal-fix** — 6 symbol renames + 1 lib-name rename + env var `AVEC_LIB`→`CGAP_LIB`, all 1:1 signature-identical (verified against `crates/cgap-ffi/src/lib.rs`: `cgap_i32_batch(i64,i64)->*mut ArrowArray`, `cgap_{schema,array}_addr()->usize`, `cgap_batch_rows()->i64`, `cgap_version()->*const c_char`, `cgap_digest_of(usize)->u64`). Well under the ~20-rename descope threshold.

**Change**: directory `bindings/python/avec` → `bindings/python/cgap`; sweep `avec_*`→`cgap_*`, `AVEC_LIB`→`CGAP_LIB`, `avec-ffi`→`cgap-ffi`, bare `avec`→`cgap` across `__init__.py` + 4 test modules; version pins 26.10.29 → 26.11.3 (`__version__` + `VERSION_LAW` in tests). Zero `avec|AVEC` references remain (grep receipt: exit 1 / no matches). Also `bindings/python/pyproject.toml`: name → `cgap`, version → 26.11.3, packages → `["cgap", "cgap.tests"]`; deleted stale `avec.egg-info` build debris.

**Bonus fix**: the stale docstring/guard narrative claimed `cgap_version()` returns a polluted `&str` — false today: `crates/cgap-ffi/src/lib.rs:87` returns `cgap_arrow::VERSION_C` (NUL-terminated, drift-guarded by `version_cstr_parity`). The loud non-NUL guard stays as drift protection; its error text now points at the real file.

## Verification ladder

1. `ggen sync run` ×2 — written:[generated_version.rs], then written:[] — exit 0 both.
2. `cargo check -p cgap-arrow` — pass.
3. `cargo build -p cgap-ffi` — produced `target/debug/libcgap_ffi.dylib` (a concurrent lane's janitor had removed it mid-session; rebuilt).
4. `PYTHONPATH=bindings/python python3 -m pytest bindings/python/cgap/tests -v` — **20 passed, 1 skipped, 0.69s**. Skip is typed GAPS-1/GAPS-2 (cgap-ffi exports no page-pipeline C symbols); pyarrow is installed and its coupled legs (bitwise C-Data import, churn leg) RAN and PASSED. Version-parity courts assert 26.11.3 end to end: TTL → generated_version.rs → cgap_version() → Python `version()`.

## Standing

- Finding 3: ALIVE (sync ×2 determinism + cargo check receipts above).
- Finding 2: ALIVE (20/21 real-ctypes tests against the real cdylib; lone skip loud + typed).
- Typed gap carried forward (pre-existing, out of lane scope): cgap-ffi exposes no `cgap_rle_decode_into` / `cgap_write_page` / `cgap_read_page`; python page-pipeline leg remains loudly SKIPPED until those exports land.
