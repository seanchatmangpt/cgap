#!/usr/bin/env python3
"""Consumer manifest patch (the one consumer-owned seam in the cgap-cli
ggen six-pack flow).

The clap-noun-verb six-pack family generates a STANDALONE Cargo.toml (its own
[workspace], crates.io pins). FM-PACK-018 (capability closure) makes
crate-pack mandatory, FM-WRITE-008 (single writer per target) forbids a
consumer template targeting the same file, and FM-WRITE-005 (no silent
clobber) makes plain re-sync refuse a patched manifest. The kernel path deps
and the operator-named ~/clap-noun-verb override therefore live in THIS
post-sync patch: idempotent on a fresh ggen-generated manifest.

Replay: rm Cargo.toml && ggen sync run && python3 manifest_patch.py
"""
import sys

P = "Cargo.toml"
s = open(P).read()
orig = s

KERNELS = (
    'cgap-arrow = { path = "../cgap-arrow" }\n'
    'cgap-parquet = { path = "../cgap-parquet" }\n'
    'cgap-git = { path = "../cgap-git" }\n'
    'cgap-blake3 = { path = "../cgap-blake3" }\n'
    'cgap-bont = { path = "../cgap-bont" }\n'
    'cgap-bn3 = { path = "../cgap-bn3" }\n'
    'cgap-gate = { path = "../cgap-gate" }\n'
    'cgap-p28 = { path = "../cgap-p28" }\n'
    'cgap-crypto = { path = "../cgap-crypto" }\n'
    'cgap-audit = { path = "../audit" }\n'
    'cgap-court = { path = "../cgap-court" }\n'
    'cgap-gpu = { path = "../cgap-gpu" }\n'
    'cgap-beam = { path = "../cgap-beam" }\n'
)

# 1) kernel path deps after the generated blake3 pin
if "cgap-arrow" not in s:
    assert 'blake3 = "1.8"' in s, "anchor missing - manifest shape drifted"
    s = s.replace('blake3 = "1.8"', 'blake3 = "1.8"\n' + KERNELS.rstrip())

# 1.5 add workspace-relative license-file after the generated publish pin
# (CLI-01: house style per cgap-core/Cargo.toml:10; crates/cli is one level
# deeper than crates/*, so ../../LICENSE.md — NOT ../LICENSE.md, which
# dangles against the absent crates/LICENSE.md, same defect DOD-01 F3c
# fixed in crates/csct)
if "license-file" not in s:
    assert "publish = false" in s, "anchor missing - manifest shape drifted"
    s = s.replace(
        "publish = false",
        'publish = false\nlicense-file = "../../LICENSE.md"',
    )

# 2) operator-named local clap-noun-verb override appended at the end
PATCHSEC = (
    "\n# Operator-named local checkout override (~/clap-noun-verb, 26.9.1) over\n"
    "# the generated crates.io =26.8.22 pin. Machine-local; consumer-owned.\n"
    "[patch.crates-io]\n"
    'clap-noun-verb = { path = "/Users/sac/clap-noun-verb" }\n'
    "clap-noun-verb-macros = { path = \"/Users/sac/clap-noun-verb/clap-noun-verb-macros\" }\n"
)
if "[patch.crates-io]" not in s:
    s = s.rstrip() + "\n" + PATCHSEC

if s != orig:
    open(P, "w").write(s)
    print("patched")
    sys.exit(0)
print("already patched")
