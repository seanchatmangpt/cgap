# Receipt: AF-12 — Live Negative-Mutation Flip Demo (kat_flip_demo.rs)

Status: ALIVE (gates green ×3 this session, exact subject below)

## Deliverable

`crates/cgap-crypto/tests/kat_flip_demo.rs` (new, 632 lines, sha256
`0f721d099ffc37f257d3645069ab1e46f094511611ca6d45403c7e012bcc630b`).

The prompts' promise — "flipping one bit in a cryptographic round constant
forces an immediate automated rejection" — is now executable as one demo
court an auditor can run with:

    cargo test -p cgap-crypto --test kat_flip_demo -- --nocapture

Mechanism: the court copies the landed kernel source to a /tmp SANDBOX
(never the repo tree — repo's ct_scan_court/AB-4 pattern), flips one
bit/byte of the constant IN THE COPY, compiles the mutated kernel standalone
with `rustc --edition=2021`, executes it, and requires the KAT/roundtrip
court to reject. 324 flips total, one binary per flip:

- **A. SIGMA (chacha.rs, AB-3)**: all 128 single-bit flips (4 words × 32
  bits) of "expand 32-byte k"; each flip must change ≥1 byte of the
  RFC 8439 §2.3.2 block fixture. Result: 128/128 detected, min changed
  bytes per flip = 1 (exactly the feedforward word), max 64/64.
- **B. RC (fips202.rs, AB-2)**: each of the 192 bytes of RC[0..24] flipped;
  each must make the FIPS 202 SHAKE-256("") KAT reject (official 64-byte
  vector). Result: 192/192 detected. Note: RC[23] (last iota) flips change
  exactly one output byte — detectable but minimal, as theory predicts.
- **C. ZETAS (fips203.rs)**: low bit of entries [1, 2, 64, 127] (all
  consumed by ntt()/intt_tomont()); each must fail the
  intt_tomont(ntt(p)) roundtrip self-test AND change the NTT output.
  Result: 4/4 detected (changed = 507, 254, 8, 8 coefficients).
  Self-test law: intt_tomont(ntt(p)) == fqmul(p, 1) coefficientwise, i.e.
  p scaled by 2^16 mod q (2285), recovered by Montgomery division by R.
- **M. Meta-court**: flips_detected == flips_total (324/324), unflipped
  baselines pass against the LANDED kernel (in-court: chacha20_block ==
  RFC 8439 §2.3.2 fixture; Shake256("") == FIPS 202 vector; landed
  ntt/intt roundtrip canonically compared mod q) AND against unflipped
  sandbox baselines (sandbox == landed outputs byte-exact), and the repo
  tree is restored byte-exact (before/after read of all three src files).

Baselines bound to the landed kernels' pub APIs only (`chacha20_block`,
`Shake256::xof`, `ntt`, `intt_tomont`, `fqmul`); zero new external
dependencies (the court is std-only; the crate's [dependencies] are
untouched). No lib/ file was edited by this lane (src/*.rs hashes stable
across all runs, before == after).

## Gates (commands + exits, this session)

- `cargo test -p cgap-crypto --test kat_flip_demo -- --nocapture` →
  `4 passed; 0 failed`, exit 0, 102.21s — /tmp/af12-run1.log
- `cargo test -p cgap-crypto --test kat_flip_demo` → `4 passed; 0 failed`,
  exit 0, 101.51s — /tmp/af12-run2.log
- Final run on the exact landed bytes (post assert-message fix):
  `4 passed; 0 failed`, exit 0, 102.05s — /tmp/af12-run3.log
  `flips_total=324 flips_detected=324 baselines_ok=true restore_ok=true`

## Defects found and fixed (all session-introduced, caught by the court)

1. `diff_bytes` counted differing hex *characters*/2 — a single-byte change
   with a single-nibble delta (0x46→0x47) counts 1 char → 0 bytes, so all
   8 RC[23] flips read "undetected" while the kernel actually diverged
   (manually witnessed: 46b9dd… → 47b9dd…). Fixed to decode-and-compare.
   This is exactly the "court that cannot flip" failure class the court
   exists to catch — caught here by manual adversarial re-derivation after
   the court ran green on everything except 8 rows.
2. Token parser fed "0x…" to `from_str_radix` (InvalidDigit).
3. RC u64 literals with the top bit set overflow i64 → parse as i128.
4. `const_block` anchored on the first `[` (the type bracket `[u64; 24]`)
   instead of the value bracket — misindexed tokens.
5. SIGMA literal match used uppercase `{:08X}` vs the source's lowercase
   underscored form — court refused (by design) rather than run blind.
6. NTT roundtrip comparison needed canonical representatives: fqmul
   returns centered residues, p0 in [0,q) — compared rem_euclid(3329).

## Notes / disclosures

- The prompt asked for "32 flips" of SIGMA; SIGMA is 4×u32 = 128 bits, so
  the court runs the exhaustive 128 (superset, one flip per bit).
- "Flip each byte of the fips202 round-constant table (AB-2's table)" —
  24 × u64 = 192 byte flips, all detected.
- AB-5 (fips203) has no receipt file in v26.11.3/receipts/, but the module
  is landed with pub ntt/intt_tomont/ZETAS; composed via pub APIs only.
  The ZETAS mirror segment [128..=255] is documentation-only (unconsumed),
  so flips target the consumed segment [1..=127].
- Transient anomaly on run 0 (first failing run): restore_ok=false was
  reported once while the kernel files were hash-identical before/after
  (sha256s above and re-verified at end; chacha 9798f891…, fips202
  dd873c5f…, fips203 919ea22e…). Not reproducible on runs 1–3
  (restore_ok=true); suspected concurrent read during another lane's
  activity on the shared checkout. Disclosed, not silently dropped.
- Sandbox dirs are pid+tag unique under /tmp and self-delete per flip.
- ~102s per full run (328 rustc invocations, 8-way threaded).

## Standing

- kat_flip_demo test target: ALIVE (3 green runs above).
- The promise "one bit → immediate automated rejection" is EXECUTABLE and
  DEMONSTRATED at 324/324, with a non-vacuity meta-court.
