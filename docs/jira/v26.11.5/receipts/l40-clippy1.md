# L40 — clippy cleanup pass 1 (registered follow-up from L36)

Standing: UNVERIFIED

## Scope

Eight leased crates (cgap-crypto, cgap-beam, cgap-aoq excluded — other lanes live).
Method: one `cargo clippy --fix --all-targets` sweep per crate (coordinator), then
4 hand-fix lanes over disjoint leases (L40a: git+arrow, L40b: parquet+ffi,
L40c: storage+bn3, L40d: p28+blake3).

## Per-crate before→after

Warning counts via clippy JSON compiler-message census (primary-span warnings),
`cargo clippy -p <crate> --all-targets`:

| crate | before | after |
|---|---|---|
| cgap-git | 189 | UNVERIFIED |
| cgap-arrow | 178 | UNVERIFIED |
| cgap-parquet | 108 | UNVERIFIED |
| cgap-ffi | 240 (incl. 1 deny-level clippy::iter_skip_zero error) | UNVERIFIED |
| cgap-storage | 67 | UNVERIFIED |
| cgap-bn3 | 95 (incl. 1 deny-level clippy::min_max error) | UNVERIFIED |
| cgap-p28 | 92 | UNVERIFIED |
| cgap-blake3 | 54 | UNVERIFIED |

(Auto-fixable subset was applied by `cargo clippy --fix` before the census; the
L36-era raw counts of ~213/25/17/14/12/12/8/8 "warning lines" counted summary
duplicates, so the JSON census above is the honest baseline.)

## Gates

- clippy zero-warnings per crate: UNVERIFIED
- cargo test per crate: UNVERIFIED
- cargo fmt --check: UNVERIFIED

## Real defects surfaced (to fill at drain)

- (none recorded yet)

## Commit

- (pending lane SHAs; single push of main at drain)
