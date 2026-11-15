# ERRC slice evaluation SZ

Lane C2, v26.11.14. Range: crate dir names starting s-z; 2 dirs. Subject /home/user/cgap/crates; every cite is file:line re-read this session (paths relative to that dir). "no candidate found" means nothing grounded was located; none invented.

| crate | action | candidate | cite |
|---|---|---|---|
| wasm-footprint | no candidate found | cdylib link wrapper; only a test-local hex helper (lib.rs:415), not worth a candidate | wasm-footprint/src/lib.rs:1 |
| wire-shm | REDUCE | crc32_ieee is one of 5 CRC-32/IEEE copies; frozen datum per CLAUDE.md, so reduce only via shared helper if ever | wire-shm/src/crc32.rs:29 |
