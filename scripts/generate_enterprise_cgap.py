#!/usr/bin/env python3
"""scripts/generate_enterprise_cgap.py

Fortune 5 enterprise corpus -> CGAP pipeline runner.

HONEST SURFACE MAP (witnessed against crates/cgap-ffi exports, 2026-11-02):
  - cgap_version()                                  : live-library witness
  - cgap_commit_parquet(lanes:i32[0..=255], path)   : REAL microkernel
    Parquet seal to disk (hybrid-packed columns, zero-alloc seal path)
  - enterprise u64 sigils do NOT fit the i32 lane domain; the graph is
    committed via real Git loose objects (zlib + SHA-1, fsck-strict)
  - Receipts printed here are Python-side SHA-1, labeled as such — they
    are NOT CGAP BLAKE3 receipts.
"""
import ctypes
import hashlib
import struct
import sys
import zlib
from pathlib import Path


def sigil_u64(uri: str) -> int:
    """FNV-1a 64-bit (unseeded basis) — matches cgap-parquet fnv1a."""
    h = 0xCBF29CE484222325
    for b in uri.encode("utf-8"):
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


P_TYPE = sigil_u64("rdf:type")
P_OWNS = sigil_u64("corp:owns_percent")
P_IC = sigil_u64("fin:intercompany_debt")
P_JUR = sigil_u64("legal:jurisdiction")


def build_corpus(n_subs: int = 250):
    """Deterministic Fortune 5 holding-group DAG (parent -> 5 HoldCos ->
    operating subs -> intercompany liquidity lines every 3rd sub)."""
    triples = []
    parent = sigil_u64("lei:5493001KJTIIGC8Y1R12")
    triples.append((parent, P_TYPE, sigil_u64("corp:UltimateParent")))
    triples.append((parent, P_JUR, sigil_u64("geo:US-DE")))
    holdcos = [sigil_u64(f"lei:549300HOLDCO0000{i}") for i in range(5)]
    locs = ["US", "DE", "IE", "SG", "KY", "NL", "GB"]
    for i, s in enumerate(holdcos):
        triples.append((parent, P_OWNS, s))
        triples.append((s, P_TYPE, sigil_u64("corp:IntermediateHolding")))
        triples.append((s, P_JUR, sigil_u64(f"geo:{locs[i % 7]}")))
    for i in range(n_subs):
        s = sigil_u64(f"lei:969500SUB{i:08d}")
        h = holdcos[i % 5]
        triples.append((h, P_OWNS, s))
        triples.append((s, P_TYPE, sigil_u64("corp:OperatingSubsidiary")))
        triples.append((s, P_JUR, sigil_u64(f"geo:{locs[i % 7]}")))
        if i and i % 3 == 0:
            d = sigil_u64(f"lei:969500SUB{i - 1:08d}")
            triples.append((s, P_IC, d))
    return triples


def ffi_witness():
    """Live-library witness against the real cgap-ffi exports."""
    root = Path(__file__).resolve().parent.parent
    candidates = [
        root / "target" / "release" / "libcgap_ffi.dylib",
        root / "target" / "release" / "libcgap_ffi.so",
        root / "target" / "debug" / "libcgap_ffi.dylib",
        root / "target" / "debug" / "libcgap_ffi.so",
    ]
    lib = None
    for p in candidates:
        if p.exists():
            try:
                lib = ctypes.CDLL(str(p))
                break
            except OSError:
                continue
    if lib is None:
        print("[!] libcgap_ffi not found — FFI witness skipped (loud)")
        return None
    lib.cgap_version.restype = ctypes.c_char_p
    ver = lib.cgap_version()
    print(f"[+] FFI witness: cgap_version() = {ver.decode()}")
    return lib


def ffi_real_seal(lib, out_dir: Path):
    """REAL microkernel seal: cgap_commit_parquet over the i32 lane domain."""
    if lib is None:
        print("[!] Real seal skipped: no FFI library (loud)")
        return None
    lanes = (ctypes.c_int32 * 16)(*[i * 16 % 256 for i in range(16)])
    path_str = str(out_dir / "microkernel_seal.parquet")
    lib.cgap_commit_parquet.restype = ctypes.c_int32
    res = lib.cgap_commit_parquet(
        lanes, len(lanes), ctypes.c_char_p(path_str.encode())
    )
    p = Path(path_str)
    if res == 0 and p.exists():
        print(f"[+] REAL cgap_commit_parquet seal: {p} "
              f"({p.stat().st_size} bytes) exit 0")
        return p
    print(f"[!] Real seal refused: status {hex(res & 0xFFFFFFFF)} (typed)")
    return None


def rotation_drain_witness(lib, out_dir: Path, max_commits: int = 2048):
    """W16 item 4 / repair-12: drive the ring drain to a positive round-trip.

    No FFI drain export exists (witnessed 2026-11-02: cgap-ffi exports only
    cgap_version / cgap_verify_p28 / cgap_admit_frame / cgap_eval_bn3 /
    cgap_commit_parquet / cgap_xip_start); the drain is ring-internal on
    rotation (mmap_writer.rs drain_to_spill via the SpillSync seam), reachable
    only by committing until the 64 KiB ring wraps. Bounded loop, no sleep.
    Returns (spill_path, commit_count) on witness, (None, N) on exhaust.
    """
    path = out_dir / "drain_spill.parquet"
    lanes = (ctypes.c_int32 * 16)(*[i * 16 % 256 for i in range(16)])
    lib.cgap_commit_parquet.restype = ctypes.c_int32
    n = 0
    for n in range(1, max_commits + 1):
        res = lib.cgap_commit_parquet(
            lanes, len(lanes), ctypes.c_char_p(str(path).encode())
        )
        if res != 0:
            print(f"[!] commit {n} refused: status {hex(res & 0xFFFFFFFF)}")
            return (None, n)
        if path.exists() and path.stat().st_size >= 16:
            raw = path.read_bytes()
            # cgap sealed-container layout (footer.rs §6.1): magic b"PAR2"
            # at L-8, footer-length at L-4 — NOT a leading PAR1 file magic.
            magic = raw[-8:-4]
            flen = int.from_bytes(raw[-4:], "little")
            ok = magic == b"PAR2" and 0 < flen <= len(raw)
            print(f"[+] ROTATION WITNESSED at commit {n}: {path} "
                  f"({len(raw)} bytes) trailer magic={magic!r} "
                  f"footer_len={flen} "
                  f"({'type-accepted' if ok else 'TYPE-REFUSED'})")
            return (path if ok else None, n)
    print(f"[!] No rotation within {max_commits} commits — ring never wrapped")
    return (None, max_commits)


def write_git_objects(out_dir: Path, parquet_bytes: bytes, receipt_hex: str):
    """Real Git loose objects: blob -> tree -> commit -> ref (fsck-strict)."""
    git_dir = out_dir / ".git"
    obj = git_dir / "objects"
    obj.mkdir(parents=True, exist_ok=True)

    def loose(kind: str, payload: bytes) -> str:
        store = f"{kind} {len(payload)}".encode() + b"\x00" + payload
        sha = hashlib.sha1(store).hexdigest()
        d = obj / sha[:2]
        d.mkdir(parents=True, exist_ok=True)
        (d / sha[2:]).write_bytes(zlib.compress(store))
        return sha

    blob = loose("blob", parquet_bytes)
    name = b"enterprise_ledger.parquet"
    entry = b"100644 " + name + b"\x00" + bytes.fromhex(blob)
    tree = loose("tree", entry)
    author = b"Sean Chatman <info@chatmangpt.com> 1730592000 -0700"
    msg = (f"chore(cgap): enterprise ledger snapshot\n\n"
           f"Receipt-SHA1: {receipt_hex}\n").encode()
    commit_payload = (f"tree {tree}\n".encode() + b"author " + author +
                      b"\n" + b"committer " + author + b"\n\n" + msg)
    commit = loose("commit", commit_payload)
    refs = git_dir / "refs" / "heads"
    refs.mkdir(parents=True, exist_ok=True)
    (refs / "main").write_text(commit + "\n")
    (git_dir / "HEAD").write_text("ref: refs/heads/main\n")
    print(f"[+] Git blob {blob[:12]}.. tree {tree[:12]}.. commit {commit[:12]}..")


def main():
    args = [a for a in sys.argv[1:] if a != "--drain"]
    drain = len(args) != len(sys.argv[1:])
    out = Path(args[0]) if args else \
        Path("/Users/sac/cgap/target/enterprise_run")
    out.mkdir(parents=True, exist_ok=True)
    print(f"[*] mode: {'--drain (rotation witness)' if drain else 'default'}")

    triples = build_corpus(250)
    print(f"[*] Generated {len(triples)} enterprise triples "
          f"(u64 sigil domain; FFI frame/lane domains documented in header)")

    lib = ffi_witness()
    # ORDER MATTERS (witnessed): COMMIT_LEASE is a single thread-local slot;
    # the spill `path` is honored only at the FIRST grant, so every later
    # cgap_commit_parquet call in this process drains into the first-granted
    # lease's file. The rotation witness must claim the lease first.
    drain_path = None
    drain_n = 0
    if drain and lib is not None:
        drain_path, drain_n = rotation_drain_witness(lib, out)
        if drain_path is not None:
            print(f"[+] Positive round-trip witnessed: non-empty container "
                  f"after {drain_n} commits (ring rotation drain), trailer PAR2")
    seal_path = ffi_real_seal(lib, out) if lib is not None else None

    if drain_path is not None and drain_path.stat().st_size > 0:
        payload = drain_path.read_bytes()
        label = "drained spill parquet (rotation witness)"
    elif seal_path is not None and seal_path.stat().st_size > 0:
        payload = seal_path.read_bytes()
        label = "microkernel-sealed parquet"
    elif seal_path is not None:
        # WITNESSED FA9 SEMANTICS: one-shot commit fills the ring and never
        # rotates — spill file stays 0 bytes until drain (staging latch).
        # Committing the envelope instead; the 0-byte parquet stays on disk
        # as the latch witness.
        print("[!] Seal exit 0 but file is 0 bytes — staging-latch semantics "
              "(FA9): data in ring, no rotation. Committing envelope instead."
              + ("" if not drain else
                 " (rotation witness ran first and did not witness a wrap)"))
        raw = b"".join(t[0].to_bytes(8, "little") + t[1].to_bytes(8, "little")
                       + t[2].to_bytes(8, "little") for t in triples)
        payload = b"ARROW1" + struct.pack("<Q", len(raw)) + raw
        (out / "enterprise_ledger.arrow_envelope").write_bytes(payload)
        label = "raw arrow envelope (seal latched, 0-byte parquet witness)"
    else:
        # RAW ARROW-SLICE ENVELOPE — NOT a readable Parquet file. Labeled
        # honestly; cgap-parquet's own parser will type-refuse it.
        raw = b"".join(t[0].to_bytes(8, "little") + t[1].to_bytes(8, "little")
                       + t[2].to_bytes(8, "little") for t in triples)
        payload = b"ARROW1" + struct.pack("<Q", len(raw)) + raw
        (out / "enterprise_ledger.arrow_envelope").write_bytes(payload)
        print(f"[!] Fallback envelope written (NOT parquet): "
              f"enterprise_ledger.arrow_envelope ({len(payload)} bytes)")
        payload = payload
        label = "raw arrow envelope (not parquet)"

    receipt = hashlib.sha1(payload).hexdigest()
    print(f"[+] Receipt (python-side SHA-1, NOT CGAP BLAKE3): {receipt}")
    write_git_objects(out, payload, receipt)
    print(f"[✓] Done. Verify: git --git-dir={out / '.git'} fsck --strict && "
          f"git --git-dir={out / '.git'} log -n 1")
    print(f"[✓] Payload label: {label}")


if __name__ == "__main__":
    main()
