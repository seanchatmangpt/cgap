//! Chicago courts for cgap-mcp: round-trip, protocol shape over REAL stdio,
//! persistence. No mocks; assert final state.

use cgap_mcp::json::{parse, Json};
use cgap_mcp::{handle_line, open_ring};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn temp_ring(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("cgap-mcp-court-{}-{}.ring", tag, std::process::id()))
}

/// Round-trip court: JSON -> wire frame -> JSON, field equality, through the
/// full daemon path (handle_line) on a REAL WireRing file, with an on-disk
/// cross-check of the slot ordinal.
#[test]
fn round_trip_court() {
    let path = temp_ring("roundtrip");
    let _ = std::fs::remove_file(&path);
    let mut ring = open_ring(&path).expect("open ring");
    let cases = vec![
        "hello wire".to_string(),
        "unicode: \u{1F600} h\u{e9}".to_string(),
        String::new(),
        "x".repeat(1000),
    ];
    for msg in &cases {
        let escaped = msg.replace('\\', "\\\\").replace('"', "\\\"");
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"echo\",\"arguments\":{{\"message\":\"{}\"}}}}}}",
            escaped
        );
        let resp = handle_line(&line, &mut ring).expect("response");
        let v = parse(&resp).expect("parse response");
        assert!(v.get("error").is_none(), "unexpected error: {:?}", v.get("error"));
        let content = v.get("result").and_then(|r| r.get("content")).expect("content");
        if let Json::Arr(items) = content {
            let text = items[0].get("text").and_then(Json::as_str).expect("text");
            assert_eq!(text, *msg, "round-trip field equality");
        } else {
            panic!("content not array");
        }
        let ordinal = v
            .get("result")
            .and_then(|r| r.get("wire"))
            .and_then(|w| w.get("ordinal"))
            .and_then(|o| if let Json::Num(n) = o { Some(*n as u32) } else { None })
            .expect("wire.ordinal");
        let rb = std::fs::read(&path).expect("read segment");
        let off = 4096 + (ordinal as usize % 64) * 4096;
        let stored_ord = u32::from_be_bytes(rb[off + 12..off + 16].try_into().unwrap());
        assert_eq!(stored_ord, ordinal, "disk slot ordinal matches response");
    }
    let _ = std::fs::remove_file(&path);
}

/// Protocol-shape court over REAL stdio: spawn the cgap-mcpd binary, write
/// JSON-RPC lines, read responses (blocking line reads are structural sync,
/// no sleeps).
#[test]
fn protocol_court_real_stdio() {
    let path = temp_ring("stdio");
    let _ = std::fs::remove_file(&path);
    let mut child = Command::new(env!("CARGO_BIN_EXE_cgap-mcpd"))
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn cgap-mcpd");
    let stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut w = BufWriter::new(stdin);
    let mut r = BufReader::new(stdout);

    let send = |w: &mut BufWriter<_>, line: &str| {
        writeln!(w, "{}", line).expect("write to daemon");
        w.flush().expect("flush");
    };
    let recv = |r: &mut BufReader<_>| -> Json {
        let mut line = String::new();
        r.read_line(&mut line).expect("read from daemon");
        parse(line.trim()).expect("daemon response is valid JSON")
    };

    // initialize shape
    send(&mut w, "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}");
    let v = recv(&mut r);
    let res = v.get("result").expect("initialize result");
    assert_eq!(res.get("protocolVersion").and_then(Json::as_str), Some("2024-11-05"));
    assert!(res.get("capabilities").is_some(), "capabilities present");
    let si = res.get("serverInfo").expect("serverInfo");
    assert_eq!(si.get("name").and_then(Json::as_str), Some("cgap-mcpd"));
    assert_eq!(si.get("version").and_then(Json::as_str), Some("26.11.5"));

    // tools/list shape
    send(&mut w, "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}");
    let v = recv(&mut r);
    let tools = v.get("result").and_then(|r| r.get("tools")).expect("tools");
    if let Json::Arr(items) = tools {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("name").and_then(Json::as_str), Some("echo"));
        assert!(items[0].get("inputSchema").is_some());
    } else {
        panic!("tools not array");
    }

    // tools/call missing name -> -32602
    send(&mut w, "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{}}");
    let v = recv(&mut r);
    let err = v.get("error").expect("-32602 error object");
    assert_eq!(err.get("code"), Some(&Json::Num(-32602.0)));

    // unknown method -> -32601
    send(&mut w, "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"bogus/method\",\"params\":{}}");
    let v = recv(&mut r);
    let err = v.get("error").expect("-32601 error object");
    assert_eq!(err.get("code"), Some(&Json::Num(-32601.0)));

    // parse error -> -32700 (id null in response)
    send(&mut w, "{not json");
    let v = recv(&mut r);
    let err = v.get("error").expect("-32700 error object");
    assert_eq!(err.get("code"), Some(&Json::Num(-32700.0)));
    assert!(v.get("id").map(|i| i.is_null()).unwrap_or(false));

    // tools/call echo through the live daemon, then shutdown
    send(&mut w, "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"echo\",\"arguments\":{\"message\":\"stdio echo\"}}}");
    let v = recv(&mut r);
    let text = v
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| if let Json::Arr(a) = c { a.first().cloned() } else { None })
        .and_then(|c| c.get("text").cloned())
        .and_then(|t| t.as_str().map(|s| s.to_string()))
        .expect("echo text");
    assert_eq!(text, "stdio echo");

    drop(w);
    let status = child.wait().expect("wait");
    assert!(status.success(), "daemon exited cleanly");
    let _ = std::fs::remove_file(&path);
}

/// Persistence court: stamp via one ring handle, drop it, reopen the
/// segment file, read the frame back, field equality.
#[test]
fn persistence_court() {
    let path = temp_ring("persist");
    let _ = std::fs::remove_file(&path);
    {
        let mut ring = open_ring(&path).expect("create ring");
        let n = ring.next_ordinal().expect("next ordinal");
        ring.stamp_frame(b"persist me", n as u32).expect("stamp");
    } // ring dropped (file closed)
    {
        let ring = open_ring(&path).expect("reopen ring");
        let payload = ring.read_frame(0).expect("read frame 0 after reopen");
        assert_eq!(payload, b"persist me");
    }
    let _ = std::fs::remove_file(&path);
}

/// Internal-error court: oversized echo payload surfaces -32603 with typed
/// wireError data (kind=FrameTooLarge).
#[test]
fn internal_error_typed_wire_data() {
    let path = temp_ring("wireerr");
    let _ = std::fs::remove_file(&path);
    let mut ring = open_ring(&path).expect("open ring");
    let big = "y".repeat(5000); // > MAX_PAYLOAD (4080)
    let line = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{{\"name\":\"echo\",\"arguments\":{{\"message\":\"{}\"}}}}}}",
        big
    );
    let resp = handle_line(&line, &mut ring).expect("response");
    let v = parse(&resp).expect("parse");
    let err = v.get("error").expect("-32603 error");
    assert_eq!(err.get("code"), Some(&Json::Num(-32603.0)));
    let data = err.get("data").expect("typed data");
    assert_eq!(data.get("kind").and_then(Json::as_str), Some("FrameTooLarge"));
    let _ = std::fs::remove_file(&path);
}
