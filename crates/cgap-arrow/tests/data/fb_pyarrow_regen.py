# Lane 7 (v26.11.6) pyarrow oracle: regenerates the golden schema-message
# vectors IN PROCESS and prints "name len sha256" lines for the court to
# diff against the pinned provenance comments in golden_vectors.rs.
# Exit 0 = ran; the caller prints a loud [SKIP rc=3] when python3/pyarrow
# is absent (Chicago: a skip is loud, never a fake pass).
import hashlib, io, struct, sys
import pyarrow as pa

def schema_message_bytes(schema):
    import pyarrow as pa
    sink = io.BytesIO()
    with pa.ipc.new_stream(sink, schema):
        pass
    buf = sink.getvalue()
    assert buf[0:4] == b"\xff\xff\xff\xff"
    msize = struct.unpack("<I", buf[4:8])[0]
    return buf[8:8 + msize]

def main():
    import pyarrow as pa
cases = {}

# scalar leaves
cases["bool_nn"] = pa.schema([pa.field("b", pa.bool_(), nullable=False)])
cases["i8"] = pa.schema([pa.field("a", pa.int8())])
cases["i16"] = pa.schema([pa.field("a", pa.int16())])
cases["i32"] = pa.schema([pa.field("a", pa.int32())])
cases["i64"] = pa.schema([pa.field("a", pa.int64())])
cases["u8"] = pa.schema([pa.field("a", pa.uint8())])
cases["u16"] = pa.schema([pa.field("a", pa.uint16())])
cases["u32"] = pa.schema([pa.field("a", pa.uint32())])
cases["u64"] = pa.schema([pa.field("a", pa.uint64())])
cases["f16"] = pa.schema([pa.field("a", pa.float16())])
cases["f32"] = pa.schema([pa.field("a", pa.float32())])
cases["f64"] = pa.schema([pa.field("a", pa.float64())])
cases["utf8"] = pa.schema([pa.field("s", pa.string())])
cases["large_utf8"] = pa.schema([pa.field("s", pa.large_string())])
cases["binary"] = pa.schema([pa.field("s", pa.binary())])
cases["large_binary"] = pa.schema([pa.field("s", pa.large_binary())])
cases["date32"] = pa.schema([pa.field("d", pa.date32())])
cases["date64"] = pa.schema([pa.field("d", pa.date64())])
cases["time32_s"] = pa.schema([pa.field("t", pa.time32("s"))])
cases["time64_ns"] = pa.schema([pa.field("t", pa.time64("ns"))])
cases["timestamp_tz"] = pa.schema([pa.field("ts", pa.timestamp("ms", tz="UTC"))])
cases["timestamp_notz"] = pa.schema([pa.field("ts", pa.timestamp("us"))])
cases["decimal128"] = pa.schema([pa.field("dec", pa.decimal128(18, 3))])
# nested
st = pa.struct([pa.field("x", pa.int32(), nullable=False), pa.field("y", pa.string())])
cases["struct"] = pa.schema([pa.field("st", st)])
cases["list"] = pa.schema([pa.field("l", pa.list_(pa.int64()))])
cases["list_nonnullable_elem"] = pa.schema([pa.field("l", pa.list_(pa.field("item", pa.int32(), nullable=False)))])
cases["fixed_size_list"] = pa.schema([pa.field("fl", pa.list_(pa.float32(), 3))])
cases["map"] = pa.schema([pa.field("m", pa.map_(pa.string(), pa.int32()))])
cases["dictionary"] = pa.schema([pa.field("dic", pa.dictionary(pa.int32(), pa.string()))])
# custom_metadata + multi-field
f_meta = pa.field("withmeta", pa.int32(), metadata={"cgap:kind": "probe", "k2": "v2"})
cases["custom_metadata"] = pa.schema([f_meta, pa.field("plain", pa.utf8())],
                                     metadata={"schema_key": "schema_val"})



def emit_hashes():
    for name in sorted(cases):
        meta = schema_message_bytes(cases[name])
        h = hashlib.sha256(meta).hexdigest()
        print(f"{name} {len(meta)} {h}")

if __name__ == "__main__":
    import pyarrow as pa  # loud ImportError -> exit 1, court prints [SKIP rc=3]
    emit_hashes()
