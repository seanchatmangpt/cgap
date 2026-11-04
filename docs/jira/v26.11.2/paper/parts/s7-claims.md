## 7. Claims mapping (appendix)

Maps every quantitative claim in this paper to a receipt path:line or an
explicit placeholder class (template: `claims-appendix.md` §A).

| # | Claim (section) | Value | Receipt path:line / placeholder |
|---|---|---|---|
| Z1 (s1) | stdio round-trip | median 7.40 µs, min 7.17, p95 13.62 (run 3), max 1741.12 µs, N=100 x 3 | ../bench-validate/p01-stdio-baseline.md:37-40,:176-181 |
| Z2 (s1) | stdio envelope | ~330 B req + ~323 B resp | ../bench-validate/p01-stdio-baseline.md:28 |
| Z3 (s1) | HTTP round-trip | keep-alive 53.084 µs median / fresh 56.166 µs median | ../bench-validate/p02-http-baseline.md:44-47 |
| Z4 (s1) | HTTP wire weight | 338 B full exchange (159 req + 179 resp) | ../bench-validate/p02-http-baseline.md:58-65 |
| Z5 (s1) | byte saving | harness-local compact codec (3-B header, NOT WIR0): 14.06% smaller, 43,191 of 307,160 B; WIR0-framed saving unmeasured [TARGET-v26.11.2] | ../bench-validate/p17-token-churn.md:43-46,94-95,148-150; crates/wire-shm/src/lib.rs:19-23 |
| Z6 (s1) | substrate courts | 8/8 tests, mutation court FLIPPED | ../lanes/lane-wire-shm.md:19-20,22-27,45-48 |
| Z7 (s1) | mcp bridge | BUILT, ALIVE: v26.11.3, bin cgap-mcpd, 5/5 tests, mutation FLIPPED; minimal JSON-RPC subset, not full MCP | ../../../../crates/cgap-mcp/RECEIPT.md:12-16,20-42,24-29 |
| Z8 (s2) | kernel ns/frame fence | 10.20 fastest / 10.30 median / 124.06 contended | position-paper-v3.md:207; ../waves/w3-l1-ingress.md:63,69-70; ../11-measured-claims.md rows 5/28 |
| Z9 (s2) | token fence | binary +27.90% tokens under o200k_base | ../bench-validate/p17-token-churn.md:106-110,112-120 |
| Z10 (s3.1) | stdio per-run stats | medians 7.38/7.46/7.40; max 1741.12/1639.58/1639.58 µs | ../bench-validate/p01-stdio-baseline.md:37-40 |
| Z11 (s3.1) | stdio binning | 99/100 envelopes < 50 µs per run | ../bench-validate/p01-stdio-baseline.md:42-49 |
| Z12 (s3.2) | HTTP legs | 41.166/53.084/72.083 and 47.000/56.166/72.083 µs, exit 0 x2 | ../bench-validate/p02-http-baseline.md:44-47 |
| Z13 (s3.2) | header blocks | 88+71=159 req; 72+107=179 resp | ../bench-validate/p02-http-baseline.md:58-65 |
| Z14 (s3.3) | p17 leg | N=1000 exchanges, 2000 envelopes, 0 failures, exit 0 | ../bench-validate/p17-token-churn.md:33-34,47-51 |
| Z15 (s3.3) | exchange medians | JSON 291 B vs binary 249 B | ../bench-validate/p17-token-churn.md:87-90 |
| Z16 (s3.3) | token totals | 96,200 vs 123,042 (+26,842) | ../bench-validate/p17-token-churn.md:106-110 |
| Z17 (s3.3) | refutation scope | corpus+tokenizer limits | ../bench-validate/p17-token-churn.md:144-155 |
| Z18 (s3.4) | zero deps | cargo tree: crate only | ../lanes/lane-wire-shm.md:21 |
| Z19 (s4.1) | frame consts | MAGIC 0x5749_5230, 16-B header, slot 4096, max payload 4080 | crates/wire-shm/src/lib.rs:3-4,19-23 |
| Z20 (s4.1) | CRC params | poly 0xEDB8_8320, known vector 0xCBF4_3926 | ../lanes/lane-wire-shm.md:10-12; crates/wire-shm/tests/wire_courts.rs:21-24 |
| Z21 (s4.1) | typed errors | FrameTooLarge/CrcMismatch/StaleOrdinal/BadMagic/Io | crates/wire-shm/src/lib.rs:26-32; ../lanes/lane-wire-shm.md:33-36 |
| Z22 (s4.2) | segment layout | 4096-B bookkeeping + 4096-B slots, FileExt not mmap | crates/wire-shm/src/ring.rs:1-5,7-10; ../lanes/lane-wire-shm.md:13-15 |
| Z23 (s4.2) | ordinal law | next_ordinal persisted after frame | crates/wire-shm/src/ring.rs:113-115; ../lanes/lane-wire-shm.md:30-36 |
| Z24 (s5) | v3 tagging law | law text | position-paper-v3.md:10-15 |
| Z25 (s6) | line-length exemption | table rows exempt-by-convention | ../../v26.11.1/waves/paper-finalize.md:34-36 |
| Z26 (s2/s4) | MCP JSON-RPC shape | p01 envelope fields | ../bench-validate/p01-stdio-baseline.md:23-28 |

Citation placeholders: none used in this paper (no external public facts
cited); the [CITE-TBD] class is defined by the tagging law and reserved.
Any future external citation must land as [CITE-TBD: version+date] first
(falsifier F5).
