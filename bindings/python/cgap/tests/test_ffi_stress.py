"""AB-15 STRESS lane — Python-side C-ABI stress over the real cgap-ffi cdylib.

Posture note (read before editing): the cgap binding's ``load_library`` speaks
``cgap_*`` symbol names against ``libcgap_ffi.dylib`` — names that do NOT
exist in this repo's cdylib. The actual cdylib is ``libcgap_ffi.dylib``
(cgap-ffi, ``cgap_version``/``cgap_i32_batch``/``cgap_array_addr``/
``cgap_schema_addr``/``cgmod_batch_rows``/``cgap_batch_rows``/
``cgap_digest_of``), so these stress tests load the real library with
``ctypes.CDLL`` directly and declare argtypes/restypes exactly as the cgap
``load_library`` court law requires (pointer returns never truncated). The
cgap NUL-termination court law (``cgap.version()`` in
``cgap/__init__.py``) is applied to the real ``cgap_version`` export: a
non-NUL-terminated read is a loud contract violation, never valid output.

Threading posture, recorded honestly: ALL tests here are single-threaded and
must stay single-threaded — ctypes releases the GIL during the call, the
cgap-ffi export slots (``ARRAY_SLOT``/``SCHEMA_SLOT``/``ROWS``) are process
global and single-slot, and the commit path is thread-lease keyed; a
multi-threaded driver would race the slots by construction. No
threading module is imported.
"""

import ctypes
import os
import re
import sys

import pytest

REPO = os.path.normpath(
    os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        os.pardir, os.pardir, os.pardir, os.pardir,
    )
)

# Single-threaded posture: GIL/contention note — ctypes releases the GIL
# during the foreign call, so "single-threaded" is a posture statement, not a
# GIL guarantee; the tests never spawn threads and serialize on nothing else.
SINGLE_THREADED_NOTE = (
    "single-threaded by charter: ctypes releases the GIL during the foreign "
    "call, so contention safety rests on never driving the process-global "
    "single export slots from more than one thread"
)


def _library_path():
    path = os.environ.get("CGAP_LIB")
    if path:
        return path
    name = "libcgap_ffi.dylib" if sys.platform == "darwin" else "libcgap_ffi.so"
    return os.path.join(REPO, "target", "debug", name)


_LIB = None


def _load():
    """Load the real cgap-ffi cdylib with the cgap load_library court law:
    every pointer-sized return declares its restype/argtypes explicitly so no
    pointer truncates under ctypes' default c_int restype."""
    global _LIB
    if _LIB is None:
        path = _library_path()
        if not os.path.exists(path):
            pytest.skip(
                f"cgap-ffi cdylib not built at {path!r}; run "
                "`cargo build -p cgap-ffi` (loud skip, never a fake pass)",
                allow_module_level=False,
            )
        lib = ctypes.CDLL(path)
        # Sovereign verbs (not driven here beyond the version court; the
        # signatures are declared so any accidental use cannot truncate).
        lib.cgap_version.restype = ctypes.c_char_p
        lib.cgap_version.argtypes = []
        lib.cgap_digest_of.restype = ctypes.c_uint64
        lib.cgap_digest_of.argtypes = [ctypes.c_size_t]
        lib.cgap_array_addr.restype = ctypes.c_size_t
        lib.cgap_array_addr.argtypes = []
        lib.cgap_schema_addr.restype = ctypes.c_size_t
        lib.cgap_schema_addr.argtypes = []
        lib.cgap_batch_rows.restype = ctypes.c_int64
        lib.cgap_batch_rows.argtypes = []
        lib.cgap_i32_batch.restype = ctypes.c_void_p
        lib.cgap_i32_batch.argtypes = [ctypes.c_int64, ctypes.c_int64]
        _LIB = lib
    return _LIB


def _cgap_version_checked(lib) -> str:
    """The cgap version-court law applied to cgap_version: the export is
    documented NUL-terminated; a read that runs into adjacent rodata is a
    loud contract violation (mirrors cgap/__init__.py version())."""
    raw = lib.cgap_version()
    if raw is None:
        raise RuntimeError("cgap stress: cgap_version() returned NULL")
    text = raw.decode("utf-8", errors="replace")
    if re.fullmatch(r"[0-9]+(?:\.[0-9]+)+", text) is None:
        raise RuntimeError(
            "cgap stress: cgap_version() returned a non-NUL-terminated "
            f"string (read {text!r}); upstream violates its documented "
            "C-string contract"
        )
    return text


@pytest.fixture(scope="module")
def lib():
    return _load()


def test_library_loads_and_version_court(lib):
    text = _cgap_version_checked(lib)
    assert text, "version must be a non-empty dotted string"
    print(f"CGAP_VERSION: {text}")


def test_ctypes_roundtrip_stress_10k(lib):
    """10k version()+digest round-trips: NUL-termination court on every
    call, digest stability, argtypes compliance throughout."""
    # Seed an export so the digest leg folds a real batch.
    lib.cgap_i32_batch(64, 3)
    addr = lib.cgap_array_addr()
    assert addr, "export slots must be live"
    assert lib.cgap_batch_rows() == 64
    d0 = lib.cgap_digest_of(addr)
    assert d0 != 0, "null-bearing 64-row batch must fold nonzero"
    for i in range(10_000):
        text = _cgap_version_checked(lib)  # NUL-termination court, every call
        d = lib.cgap_digest_of(addr)
        assert d == d0, f"iter {i}: digest drifted without a re-export"
    print("ROUNDTRIP: 10k version+digest calls, NUL court green every call")


def test_argtypes_compliance():
    """Pointer restypes are declared (non-truncating) — the cgap
    load_library law — and the 64-bit addresses round-trip losslessly."""
    lib = _load()
    assert lib.cgap_array_addr.restype is ctypes.c_size_t
    assert lib.cgap_schema_addr.restype is ctypes.c_size_t
    assert lib.cgap_digest_of.restype is ctypes.c_uint64
    assert lib.cgap_i32_batch.restype is ctypes.c_void_p
    assert lib.cgap_i32_batch.argtypes == [ctypes.c_int64, ctypes.c_int64]
    addr = lib.cgap_array_addr()
    assert addr == 0 or addr > 2**32, "c_size_t restype must preserve 64-bit addresses"


def test_cdata_churn_1k_export_import_byte_stable():
    """1k export->import cycles through pyarrow's C Data Interface: byte
    stability every cycle against recomputed ground truth (gate3 logic).
    Loud SKIP when pyarrow (the test extra) is absent — never a fake pass."""
    pyarrow = pytest.importorskip(
        "pyarrow",
        reason="pyarrow (cgap test extra) absent: C-Data churn leg SKIPPED loudly",
    )
    lib = _load()
    n, null_every = 64, 3
    # Recomputed ground truth (test_cdata.py canon).
    valid = [not (i % null_every == null_every - 1) for i in range(n)]
    expected_values = b"".join(
        ((i if v_ else 0) & 0xFFFFFFFF).to_bytes(4, "little") for i, v_ in enumerate(valid)
    )
    expected_nulls = n // null_every
    for cycle in range(1_000):
        lib.cgap_i32_batch(n, null_every)
        array_addr = lib.cgap_array_addr()
        schema_addr = lib.cgap_schema_addr()
        assert array_addr and schema_addr, f"cycle {cycle}: null export slots"
        assert lib.cgap_batch_rows() == n
        arr = pyarrow.Array._import_from_c(array_addr, schema_addr)
        assert len(arr) == n
        assert arr.null_count == expected_nulls, f"cycle {cycle}: null count"
        _, dbuf = arr.buffers()[0], arr.buffers()[1]
        assert bytes(dbuf)[: 4 * n] == expected_values, f"cycle {cycle}: values drift"
    print("CDATA-CHURN: 1k export->import cycles byte-stable")
