"""ctypes binding for the cgap-ffi Arrow C Data Interface cdylib.

Runtime dependency-free. pyarrow is a test-only extra: the Arrow C Data
Interface import (``cgap_new_i32_batch`` -> ``pa.Array``) imports it lazily
at call time.

Library resolution: ``$CGAP_LIB`` if set, else ``libcgap_ffi.dylib``/``.so``
under ``<repo>/target/debug`` resolved relative to this file
(``bindings/python/cgap`` -> three levels up is the repo root).
"""

from __future__ import annotations

import ctypes
import os
import sys

__version__ = "26.11.5"

__all__ = [
    "__version__",
    "version",
    "cgap_new_i32_batch",
    "load_library",
]

_LIB = None


def _default_library_path() -> str:
    name = (
        "libcgap_ffi.dylib"
        if sys.platform == "darwin"
        else "libcgap_ffi.so"
    )
    here = os.path.dirname(os.path.abspath(__file__))
    return os.path.normpath(
        os.path.join(here, os.pardir, os.pardir, os.pardir, "target", "debug", name)
    )


def load_library():
    """Load (once, cached) and return the cgap-ffi cdylib ctypes handle.

    Signatures are configured here so pointer returns are never truncated:
    ``cgap_array_addr``/``cgap_schema_addr`` return ``usize`` (c_size_t), and
    ``cgap_i32_batch`` returns a pointer (c_void_p), which would truncate on
    64-bit under ctypes' default ``c_int`` restype.
    """
    global _LIB
    if _LIB is None:
        path = os.environ.get("CGAP_LIB") or _default_library_path()
        try:
            lib = ctypes.CDLL(path)
        except OSError as exc:
            raise RuntimeError(
                f"cgap: cannot load FFI library {path!r}: {exc} "
                "(build the cdylib or set CGAP_LIB)"
            ) from exc
        lib.cgap_i32_batch.restype = ctypes.c_void_p
        lib.cgap_i32_batch.argtypes = [ctypes.c_int64, ctypes.c_int64]
        lib.cgap_array_addr.restype = ctypes.c_size_t
        lib.cgap_array_addr.argtypes = []
        lib.cgap_schema_addr.restype = ctypes.c_size_t
        lib.cgap_schema_addr.argtypes = []
        lib.cgap_batch_rows.restype = ctypes.c_int64
        lib.cgap_batch_rows.argtypes = []
        lib.cgap_version.restype = ctypes.c_char_p
        lib.cgap_version.argtypes = []
        _LIB = lib
    return _LIB


def version() -> str:
    """Version string reported by the loaded cgap-ffi cdylib.

    Refuses loudly if the returned bytes are not a bare dotted version:
    cgap_version() is documented as NUL-terminated, but the current cdylib
    returns a &str pointer whose strlen runs into adjacent rodata literals
    (seen: b"26.11.3there is no such thing as a release load..."), so a
    polluted read is an upstream cgap-ffi contract violation, not valid
    output.
    """
    import re

    raw = load_library().cgap_version()
    if raw is None:
        raise RuntimeError("cgap: cgap_version() returned NULL")
    text = raw.decode("utf-8", errors="replace")
    if re.fullmatch(r"[0-9]+(?:\.[0-9]+)+", text) is None:
        raise RuntimeError(
            "cgap: cgap_version() returned a non-NUL-terminated string "
            f"(read {text!r}); upstream cgap-ffi violates its documented "
            "C-string contract — fix crates/cgap-ffi/src/lib.rs to expose a "
            "NUL-terminated static"
        )
    return text


def cgap_new_i32_batch(n: int, null_every: int) -> "pa.Array":
    """Export an i32 batch and import it through pyarrow's C Data Interface.

    Values are ``0..n-1`` with a null wherever ``i % null_every ==
    null_every - 1`` (no nulls when ``null_every == 0``). Raises if the
    export slots are unset or the batch length disagrees with ``n``.
    pyarrow is imported lazily; it is a test-only dependency.
    """
    lib = load_library()
    lib.cgap_i32_batch(n, null_every)
    array_addr = lib.cgap_array_addr()
    schema_addr = lib.cgap_schema_addr()
    rows = lib.cgap_batch_rows()
    if rows != n:
        raise RuntimeError(f"cgap: batch rows {rows} != requested {n}")
    if not array_addr or not schema_addr:
        raise RuntimeError("cgap: null export slots")
    pa = _pyarrow()
    return pa.Array._import_from_c(array_addr, schema_addr)


def _pyarrow():
    try:
        import pyarrow as pa
    except ImportError as exc:  # pragma: no cover - exercised via extra-less env
        raise RuntimeError(
            "cgap: pyarrow is required for cgap_new_i32_batch "
            "(test extra): pip install 'cgap[test]'"
        ) from exc
    return pa
