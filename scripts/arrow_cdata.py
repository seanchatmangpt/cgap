"""arrow_cdata.py — Pure Python / ctypes Arrow C Data Interface bindings.

Implements the official Apache Arrow C Data Interface specification with zero
dependencies (no pyarrow requirement). Matches 80-byte LP64 layout of
crates/cgap-arrow/src/c_abi.rs:
  - ArrowSchema (72 bytes)
  - ArrowArray (80 bytes)
"""
import ctypes
import os
import sys

class ArrowSchema(ctypes.Structure):
    pass

class ArrowArray(ctypes.Structure):
    pass

ArrowSchema._fields_ = [
    ("format", ctypes.c_char_p),
    ("name", ctypes.c_char_p),
    ("metadata", ctypes.c_char_p),
    ("flags", ctypes.c_int64),
    ("n_children", ctypes.c_int64),
    ("children", ctypes.POINTER(ctypes.POINTER(ArrowSchema))),
    ("dictionary", ctypes.POINTER(ArrowSchema)),
    ("release", ctypes.c_void_p),
    ("private_data", ctypes.c_void_p),
]

ArrowArray._fields_ = [
    ("length", ctypes.c_int64),
    ("null_count", ctypes.c_int64),
    ("offset", ctypes.c_int64),
    ("n_buffers", ctypes.c_int64),
    ("n_children", ctypes.c_int64),
    ("buffers", ctypes.POINTER(ctypes.c_void_p)),
    ("children", ctypes.POINTER(ctypes.POINTER(ArrowArray))),
    ("dictionary", ctypes.POINTER(ArrowArray)),
    ("release", ctypes.c_void_p),
    ("private_data", ctypes.c_void_p),
]

assert ctypes.sizeof(ArrowArray) == 80, f"ArrowArray size must be 80, got {ctypes.sizeof(ArrowArray)}"
assert ctypes.sizeof(ArrowSchema) == 72, f"ArrowSchema size must be 72, got {ctypes.sizeof(ArrowSchema)}"


def make_primitive_u64_array(values):
    """Create a pinned ArrowArray for a continuous slice of uint64 values."""
    n = len(values)
    c_vals = (ctypes.c_uint64 * n)(*values)
    
    # 2 buffers: Buffer 0 = null bitmap (NULL if null_count == 0), Buffer 1 = values
    buffers = (ctypes.c_void_p * 2)(None, ctypes.cast(c_vals, ctypes.c_void_p))
    
    arr = ArrowArray()
    arr.length = n
    arr.null_count = 0
    arr.offset = 0
    arr.n_buffers = 2
    arr.n_children = 0
    arr.buffers = ctypes.cast(buffers, ctypes.POINTER(ctypes.c_void_p))
    arr.children = None
    arr.dictionary = None
    arr.release = None
    arr.private_data = None
    
    schema = ArrowSchema()
    schema.format = b"w:8"  # 8-byte uint64 / bit-plane word
    schema.name = b"sovereign_column"
    schema.metadata = None
    schema.flags = 0
    schema.n_children = 0
    schema.children = None
    schema.dictionary = None
    schema.release = None
    schema.private_data = None
    
    # Keep Python references alive in a companion tuple to avoid GC collection
    keepalive = (c_vals, buffers)
    return arr, schema, keepalive
