#!/usr/bin/env python3
"""Generate the CSCB coordinate corpus (lane B08).

Deterministic: SplitMix64 stream seeded 0xA20C_0FFE_E5A7_E000
(multiplier 0xBF58_476D_1CE4_E5B9, golden gamma 0x9E3779B97F4A7C15).
Emits 1,000,000 coordinate triples, each 3x u64 little-endian = 24 bytes.

Usage:
  python3 scripts/generate_cscb_corpus.py [--output PATH] --validate

--validate re-reads the written file and re-hashes; the printed sha256
must match the generation-side digest (byte-identical replay court).
"""
import argparse
import hashlib
import os
import struct
import sys

SEED = 0xA20C_0FFE_E5A7_E000
MULTIPLIER = 0xBF58_476D_1CE4_E5B9
GAMMA = 0x9E3779B97F4A7C15
MASK = (1 << 64) - 1
COUNT = 1_000_000
TRIPLE_BYTES = 24


class SplitMix64:
    def __init__(self, seed):
        self._state = seed & MASK

    def next(self):
        self._state = (self._state + GAMMA) & MASK
        z = (self._state ^ (self._state >> 30)) * MULTIPLIER & MASK
        z = (z ^ (z >> 27)) * MULTIPLIER & MASK
        return (z ^ (z >> 43)) & MASK


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--output", default="/tmp/cscb_corpus.bin")
    ap.add_argument("--validate", action="store_true")
    args = ap.parse_args()

    if args.validate:
        h = hashlib.sha256()
        with open(args.output, "rb") as f:
            while True:
                chunk = f.read(1 << 20)
                if not chunk:
                    break
                h.update(chunk)
        n = os.path.getsize(args.output) // TRIPLE_BYTES
        print(f"sha256 {h.hexdigest()}")
        print(f"count  {n}")
        return 0

    rng = SplitMix64(SEED)
    h = hashlib.sha256()
    with open(args.output, "wb") as f:
        rec = bytearray(TRIPLE_BYTES)
        for _ in range(COUNT):
            a = rng.next()
            b = rng.next()
            c = rng.next()
            struct.pack_into("<QQQ", rec, 0, a, b, c)
            f.write(rec)
            h.update(rec)
    print(f"sha256 {h.hexdigest()}")
    print(f"count  {COUNT}")
    print(f"path   {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
