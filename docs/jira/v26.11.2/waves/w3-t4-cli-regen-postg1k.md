# W3-T4 CLI regen + root projection re-proof post-G1K

## (1) Root ggen sync run x2

- Pass 1: `ggen sync run` (repo root), exit 0
- Pass 2: `ggen sync run` (repo root), exit 0
- Sync report both passes lists generated/campaign-board.md,
  generated/manifest-facts.txt, generated/bench-registry.txt
  (8 mentions each log, none MISSING) — the G1K-4 silent-template
  finding did NOT reproduce; all three render + unchanged:content identical.
- Digests (stable across both passes):
  - campaign-board.md b4260072fa703ffe5e8a2fc03b76b9c47cd05cb7d7b30b1e715b37b075a4f4f0
  - manifest-facts.txt 80548918851878d5636623ed8af7e6ac22bcc830ece67cd7624e9045739e3f63
  - bench-registry.txt 6353775608251b59accd414334f6a798982a0d1b40df6b3c9b085d1a425c138f

## (2) crates/cli regen contract x2

- Pass 1: `rm Cargo.toml && ggen sync run` exit 0; `python3 manifest_patch.py` exit 0
- Pass 2: same, exits 0/0
- Cargo.toml `version = "26.11.2"` after BOTH passes — graph cnv:version
  fix intact (crates/cli/ontology/cgap-cli.ttl:6).
- Byte-stability: shasum of Cargo.toml + src/*.rs set pre-regen (before rm)
  vs post-pass-2 → `CLI_TREES_BYTE_IDENTICAL` (diff empty).
- Post-regen digests: Cargo.toml 68d43be3..., main.rs 24bbb5bd...,
  lib.rs a94150c3..., generated_cli.rs 32f417ed...,
  generated_behavior.rs 0518fc90..., generated_boundaries.rs 9f859e87...,
  zero_code_cli.rs b247b2db... (full sha256 in session log).

## (3) make gate0

- Exit 0 from repo root: byte-stable replay proof PASS (4-file sha256
  pair cmp clean) + C header compile PASS (-std=c99 -ffreestanding
  -Wall -Wextra -pedantic).
- gate0 sha256 table (run2):
  - include/cgap_c_data.h 01d828b391ca739b5d482dcf76df8b841b4b356412ac8b36d49c697dfbef4f88
  - include/cgap_format_codes.h bf7f5b54215c81d30f11ec9cbdb1f60bb72a162b978eed506a10a6da49bf2bc1
  - crates/cgap-arrow/src/generated_layout.rs 96c67c577582d4193513aab686b7a3f7f7a638abe7e12db69004e68f7f5c09c3
  - crates/cgap-arrow/src/generated_version.rs ae0f02f73ce5f4d8fa83c7a6392f9fb3e98754364a53a8ed4d72a1119326939f

## FM-PACK-008 / ggen.lock

- No FM-PACK-008 this session: root sync passes validate cgap-pack
  content_hash cc8c83d3087bd0ce6f98dc4d6d5a9a8eb74d9ef738603be20fd5656b9b671986,
  matching the working-tree ggen.lock (one-line delta vs HEAD:
  0a3069... -> cc8c83..., the G1K lanes' re-lock already in tree).
- No re-lock remediation needed; nothing re-locked by W3-T4.

## Divergences

- First gate0 invocation was accidentally run with cwd=crates/cli and
  failed FM-WRITE-005 (refusing to clobber the patched Cargo.toml).
  That is my command-sequencing error, not a repo defect; rerun from
  repo root exited 0. Recorded for honesty, not a lane finding.
- Standing: ALIVE. Every claim above has command + exit run this
  session (2026-11-02).
