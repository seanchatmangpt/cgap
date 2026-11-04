"""Wire-parity tests for the cgap Python binding.

Mirrors the ground-truth computation of ``scripts/gate3_cdata.py``: recompute
the expected validity bitmap and values buffer in python and assert BITWISE
equality against what came through the Arrow C Data Interface. Real
collaborators only — the cdylib and pyarrow are loaded for real.
"""

import pytest

import cgap

VERSION_LAW = "26.11.3"

CONFIGS = [(100_000, 7), (1000, 0), (65537, 1000)]


def _expected(n: int, null_every: int):
    """Recompute ground truth (gate3 logic, generalized to null_every).

    Null wherever ``i % null_every == null_every - 1`` (none when
    ``null_every == 0``); value is ``i`` where valid, ``0`` where null, as
    little-endian u32. Returns (expected null count, validity bitmap bytes,
    values buffer bytes).
    """
    valid = [
        not (null_every > 0 and i % null_every == null_every - 1)
        for i in range(n)
    ]
    bm = bytearray((n + 7) // 8)
    for i, v in enumerate(valid):
        if v:
            bm[i // 8] |= 1 << (i % 8)
    validity = bytes(bm)
    values = b"".join(
        ((i if v else 0) & 0xFFFFFFFF).to_bytes(4, "little")
        for i, v in enumerate(valid)
    )
    nulls = n // null_every if null_every > 0 else 0
    return nulls, validity, values


def _assert_bitmap_bitwise(actual, expected: bytes, n: int) -> None:
    """Bitwise-compare a validity buffer over the n in-range bits only.

    Bits at indexes >= n are builder padding and unspecified, so the final
    partial byte is compared under a mask; full bytes compare exactly.
    """
    gv = bytes(actual)
    full = n // 8
    assert gv[:full] == expected[:full], "validity bitmap mismatch (full bytes)"
    if n % 8:
        mask = (1 << (n % 8)) - 1
        assert (gv[full] & mask) == (expected[full] & mask), (
            "validity bitmap mismatch (partial byte)"
        )


def test_version_parity():
    assert cgap.__version__ == VERSION_LAW
    assert cgap.version() == VERSION_LAW


def test_cgap_lib_env_override():
    # Real collaborator, real env var: re-resolve $CGAP_LIB (absolute path to
    # the same cdylib) and confirm the override actually loads.
    dylib = cgap._default_library_path()
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(cgap, "_LIB", None)
        mp.setenv("CGAP_LIB", dylib)
        lib = cgap.load_library()
        assert cgap.version() == VERSION_LAW
    assert lib.cgap_version().decode("utf-8") == VERSION_LAW


@pytest.mark.parametrize("n, null_every", CONFIGS)
def test_i32_batch_bitwise(n, null_every):
    import pyarrow as pa

    arr = cgap.cgap_new_i32_batch(n, null_every)
    assert arr.type == pa.int32(), f"type: {arr.type}"
    assert len(arr) == n

    # null_count asserts (gate3: null_count == N // NULL_EVERY)
    expected_nulls, expected_validity, expected_values = _expected(n, null_every)
    assert arr.null_count == expected_nulls

    # values buffer: bitwise against recomputed ground truth
    vbuf, dbuf = arr.buffers()[0], arr.buffers()[1]
    assert dbuf is not None, "values buffer missing"
    assert bytes(dbuf)[: 4 * n] == expected_values, "values bitwise mismatch"

    # validity buffer: bitwise over in-range bits; omitting it is legal only
    # for a zero-null batch
    if vbuf is not None:
        _assert_bitmap_bitwise(vbuf, expected_validity, n)
    else:
        assert null_every == 0, "validity buffer missing on a null-bearing batch"
