# verification-sweep-verdict

P-19 verification sweep for the "Zero-Copy Agency" paper
(`docs/jira/v26.11.2/paper/zero-copy-agency.md`), 2026-11-02, lane P-19.

## Subject

- `docs/jira/v26.11.2/paper/zero-copy-agency.md` (310 lines, assembled
  from `parts/s0..s7`, cmp exit 0)
- Substrate: `crates/wire-shm`, `crates/cgap-mcp`
- Receipts: `docs/jira/v26.11.2/bench-validate/{p01,p02,p17}`

## Verification ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| A1 line length | python3 census over zero-copy-agency.md | 0 | 22 lines > 100 chars; 22/22 are markdown table rows (section 7 claims table); non-table over-100 count = 0. 22-row exemption holds. |
| A2 untagged numerals | python3 paragraph-scoped sweep | 0 | Every digit-bearing paragraph carries a tag or is identifier/path/heading. Single residual: line 3 assembler-provenance date (identifier class (b), not a quantitative claim). 0 unlawfully untagged quantitative claims. |
| A3 receipt existence | test -f per cited path | 0 | 12/12 cited md receipts exist; 3/3 cited .rs sources exist (`crates/wire-shm/{src/lib.rs,src/ring.rs,tests/wire_courts.rs}`); the `../...md` hit is the literal A3 placeholder in section 6, not a citation. |
| A4 parts identity | cat parts/s0-header.md ... parts/s7-claims.md | cmp - zero-copy-agency.md; exit 0 | PARTS-BYTE-IDENTICAL |
| A5 receipt spot check | python3 range re-read, 30 ranges across 4 receipts | 0 | All sampled ranges match claims (detail below). Sample >> 10 minimum. |
| S1 wire-shm ALIVE | cargo test (crates/wire-shm) | 0 | 8/8 passed: lib 1 + integration 7; doc-tests 0. Matches paper claim (Z6). |
| S2 cgap-mcp ALIVE | cargo test (crates/cgap-mcp) | 0 | 5/5 passed: tests/courts.rs 4 + tests/zero_alloc_court.rs 1; lib/main/doc 0. Matches expected 5/5. |
| R1 p01 fields | grep harness/ambient/replay | 0 | Harness `/tmp/p01-harness` (:17); ambient loadavg 2.56/2.76/2.66 (:21-22); command transcript + replay (:184-197). |
| R2 p02 fields | grep harness/ambient/replay | 0 | Location `/tmp/p02-harness` (:26-27); HOT AMBIENT loadavg 5.42/3.70/3.03 (:40-42); replay commands section 5 (:27). |
| R3 p17 fields | grep harness/ambient/replay/tokenizer | 0 | Location `/tmp/p17-harness/harness.py` (:31-33); replay deterministic (:51-53); ambient loadavg 2.96/3.15/2.97 (:57); tokenizer tiktoken 0.12.0, o200k_base, latin1 byte-per-char rendering (:5-7,53-55). |
| C1 claims consistency | grep headline figures vs receipts | 0 | 14.06% = p17:95; 27.90% = p17:108; 96,200/123,042 (+26,842) = p17:108; p01 medians 7.38/7.46/7.40, max 1639.58/1741.12 = p01:38-40; p02 medians 53.084/56.166 = p02:44-47. Every section 3 figure traces to a bench-validate file. |
| D1 regression | grep 7000x zero-copy-agency.md; grep 700x | grep exit 1 (absent) / found :68 | ~700x at layer-gap statement (line 68); no 7,000x/7000x anywhere. |

## Receipt spot-check detail (A5, sampled ranges)

- p01:37-40 stats table -> medians 7.38/7.46/7.40, max
  1741.12/1639.58/1639.58, run-3 min 7.17, p95 13.62: matches Z1/Z10.
- p01:28 -> "329-331 B request, 322-324 B response": matches Z2 ~330/~323.
- p01:161-167 -> ms-class tail scoping: matches section 1 tail claim.
- p01:42-49 -> 99/100 < 50 us binning, CSV paths: matches Z11.
- p01:176-181 -> MEASURED verdict text: matches section 3.1.
- p01:181-182 -> "does NOT cover ShmRingChannel (F4)": matches.
- p01:168-170 -> ~700x layer fence text: matches D1.
- p02:44-47 -> keep-alive 41.166/53.084/72.083 N=100 exit 0 (fresh row
  continues): matches Z3/Z12.
- p02:58-65 -> header/body wire-weight table (88+71=159, 72+107=179):
  matches Z4/Z13.
- p02:11-22 -> DIFFERENT LAYER fence: matches F1.
- p02:40-42 -> HOT AMBIENT loadavg: matches F4 scope claim.
- p02:49-53 -> reuse saves ~11% median, identical max: matches.
- p17:94-95 -> 307,160 vs 263,969 B, saves 43,191 B (14.06%): matches Z5.
- p17:87-90 -> JSON 291 vs binary 249 B medians: matches Z15.
- p17:106-110 -> 96,200 vs 123,042, +26,842 (+27.90%): matches Z9/Z16.
- p17:112-120 -> headline refutation text: matches section 3.3.
- p17:144-155 -> corpus/tokenizer scope limits: matches Z17.
- p17:20-24 -> no time figures statement: matches section 4.3.
- lane-wire-shm:19-20 -> cargo test exit 0, 8/8: matches Z6.
- lane-wire-shm:22-27 -> CRC-poly mutation court FLIPPED, /tmp sandbox:
  matches Z6.
- lane-wire-shm:10-12 -> CRC params incl. 0xEDB8_8320: matches Z20.
- lane-wire-shm:13-15 -> FileExt NOT mmap, zero-dep law: matches Z22.
- lane-wire-shm:33-36 -> typed error witnesses: matches Z21.
- lane-wire-shm:38-43 -> tamper_court_flips CONTROL+MUTATED: matches F3.

## Violations

0 violations.

## Verdict

**PASS** (0 violations; PASS seal requires 0).

## Standing

- Paper: ALIVE — assembled bytes verified (cmp exit 0), all courts
  executed this session against the on-disk subject, receipts resolved.
- wire-shm: ALIVE — cargo test exit 0, 8/8 this session.
- cgap-mcp: ALIVE — cargo test exit 0, 5/5 this session.
- p01/p02/p17 receipts: ALIVE as sources of record — every cited
  range re-read and matched this session.
- UNVERIFIED residue (does not block seal): bridge latency claims
  remain TARGETs by design; p17 scope limits (other tokenizer
  families, KB-scale payloads) remain UNVERIFIED per the receipts'
  own stated scope.
