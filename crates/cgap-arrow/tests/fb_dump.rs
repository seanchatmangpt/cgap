use cgap_arrow::flatbuf::*;
#[test]
fn decode_elixir_bytes() {
    let hex = include_str!("data/elixir_schema_fb.hex").trim().to_string();
    let bytes: Vec<u8> = (0..hex.len() / 2)
        .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap())
        .collect();
    let m = read_message(&bytes).unwrap();
    let f = m.schema.fields().unwrap();
    println!("fields={}", f.len());
    for i in 0..f.len() {
        let fl = f.get(i).unwrap();
        println!("  {} nullable={} meta_len={}", fl.name, fl.nullable, fl.meta.len());
        assert!(matches!(fl.ty, _) || true);
    }
    let f0 = f.get(0).unwrap();
    assert_eq!(f0.name, "a");
    assert!(matches!(f0.ty, Ty::Int { bits: 32, signed: true }));
    let f1 = f.get(1).unwrap();
    assert_eq!(f1.name, "b");
    assert!(matches!(f1.ty, Ty::Utf8));
    println!("ELIXIR-BYTES-OK");
}
