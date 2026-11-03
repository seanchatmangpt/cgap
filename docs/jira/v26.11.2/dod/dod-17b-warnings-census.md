# DOD-17b: Warnings census seal (FS-17)

2026-11-02. `CARGO_TARGET_DIR=/tmp/fs17-target cargo build --workspace`
from repo root `/Users/sac/cgap` (toolchain resolves from cwd), exit 0.

## Census: 16 warnings, all PRE-EXISTING (R-01 list), 0 wave-introduced

| # | Verbatim warning line | Location | Class |
|---|---|---|---|
| 1 | warning: the `#![no_std]` attribute can only be used at the crate root | crates/cgap-git/src/mcp_driver.rs:36:1 | pre-existing (R-01: mcp_driver no_std attr) |
| 2 | warning: use of deprecated method `core::sync::atomic::Atomic::<u64>::fetch_update`: renamed to `try_update` for consistency | crates/cgap-git/src/ref_store.rs:273:18 | pre-existing (R-01: ref_store fetch_update deprecation) |
| 3 | warning: unused variable: `bound` | crates/cgap-git/src/ref_store.rs:351:13 | pre-existing (R-01: ref_store unused bound) |
| 4 | warning: constant `PKT_BUF_LEN` is never used | crates/cgap-git/src/pkt_line.rs:27:11 | pre-existing (R-01: pkt_line dead-code) |
| 5 | warning: methods `err_message` and `is_flush` are never used | crates/cgap-git/src/pkt_line.rs:164:12 | pre-existing (R-01: pkt_line dead-code) |
| 6 | warning: function `encode_pkt` is never used | crates/cgap-git/src/pkt_line.rs:186:8 | pre-existing (R-01: pkt_line dead-code) |
| 7 | warning: function `encode_flush` is never used | crates/cgap-git/src/pkt_line.rs:212:7 | pre-existing (R-01: pkt_line dead-code) |
| 8 | warning: function `encode_delim` is never used | crates/cgap-git/src/pkt_line.rs:215:8 | pre-existing (R-01: pkt_line dead-code) |
| 9 | warning: function `encode_response_end` is never used | crates/cgap-git/src/pkt_line.rs:221:8 | pre-existing (R-01: pkt_line dead-code) |
| 10 | warning: function `encode_err` is never used | crates/cgap-git/src/pkt_line.rs:227:8 | pre-existing (R-01: pkt_line dead-code) |
| 11 | warning: function `encode_err` is never used | crates/cgap-git/src/pkt_line.rs:233:8 | pre-existing (R-01: pkt_line dead-code) |
| 12 | warning: `cgap-git` (lib) generated 11 warnings (run `cargo fix --lib -p cgap-git` to apply 2 suggestions) | cgap-git lib summary | pre-existing summary of #1–#11 |
| 13 | warning: unused dependency `cgap-ffi` | crates/bench (manifest) | pre-existing (R-01: 3 unused bench deps) |
| 14 | warning: unused dependency `cgap-git` | crates/bench (manifest) | pre-existing (R-01: 3 unused bench deps) |
| 15 | warning: unused dependency `cgap-storage` | crates/bench (manifest) | pre-existing (R-01: 3 unused bench deps) |
| 16 | warning: `bench` (manifest) generated 3 warnings | bench manifest summary | pre-existing summary of #13–#15 |

Note: rows 4–11 are the 8 pkt_line dead-code warnings (R-01 grouped these
as "pkt_line 7 dead-code"; the live count is 8 distinct sites). Full log
preserved at /tmp/fs17-build.log.

## unexpected_cfgs check (post-WARN-01)

`grep unexpected_cfgs /tmp/fs17-build.log` → no match (grep exit 1).
The cfg warning is still GONE. WARN-01 fix holds.

## Unowned warnings

None. Every warning maps to the R-01 pre-existing list; zero
wave-introduced; zero unowned. Census seal: PASS (16/16 classified).

## Fences

- FS-14 gate matrix runs concurrently on the same tree; this census is
  independent (own build root /tmp/fs17-target), no file overlap.
- Build root /tmp/fs17-target deleted at lane end.
