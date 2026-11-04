//! cgap-mcp — MCP-JSON to wire-shm binary-frame translation daemon library.
//!
//! JSON-RPC 2.0 line-delimited over stdin/stdout. Protocol surface:
//! `initialize`, `tools/list`, `tools/call` (one `echo` tool backed by a
//! wire-shm WireRing). Errors: -32700 parse, -32601 method-not-found,
//! -32602 invalid-params, -32603 internal (with typed wireError data).

pub mod json;
pub mod wire;

use json::{parse, to_string, Json};
use std::path::Path;
use wire_shm::{WireError, WireRing};

pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const SERVER_NAME: &str = "cgap-mcpd";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Static tool registry: one echo tool.
pub const TOOLS: &[&str] = &["echo"];

/// Open-or-create a ring at `path` (capacity 64 slots).
pub fn open_ring(path: &Path) -> Result<WireRing, WireError> {
    if path.exists() {
        WireRing::open(path)
    } else {
        WireRing::create(path, 64)
    }
}

/// JSON-RPC error codes.
pub const PARSE_ERROR: f64 = -32700.0;
pub const METHOD_NOT_FOUND: f64 = -32601.0;
pub const INVALID_PARAMS: f64 = -32602.0;
pub const INTERNAL_ERROR: f64 = -32603.0;

/// Handle one JSON-RPC line; None = notification (no id), no response.
pub fn handle_line(line: &str, ring: &mut WireRing) -> Option<String> {
    let req = match parse(line) {
        Ok(v) => v,
        Err(e) => return Some(to_string(&error_response(Json::Null, PARSE_ERROR, &e, None))),
    };
    let id = req.get("id").cloned().unwrap_or(Json::Null);
    let method = req.get("method").and_then(Json::as_str).unwrap_or("").to_string();
    let params = req.get("params").cloned().unwrap_or(Json::Obj(Vec::new()));
    if id.is_null() {
        return None; // notification: no response
    }
    match method.as_str() {
        "initialize" => Some(to_string(&Json::obj(vec![
            ("jsonrpc", Json::s("2.0")),
            ("id", id),
            ("result", result_initialize()),
        ]))),
        "tools/list" => Some(to_string(&Json::obj(vec![
            ("jsonrpc", Json::s("2.0")),
            ("id", id),
            ("result", result_tools_list()),
        ]))),
        "tools/call" => match tools_call(&params, ring) {
            Ok(result) => Some(to_string(&Json::obj(vec![
                ("jsonrpc", Json::s("2.0")),
                ("id", id),
                ("result", result),
            ]))),
            Err((code, msg, data)) => Some(to_string(&error_response(id, code, &msg, data))),
        },
        _ => Some(to_string(&error_response(
            id,
            METHOD_NOT_FOUND,
            &format!("method not found: {}", method),
            None,
        ))),
    }
}

fn result_initialize() -> Json {
    Json::obj(vec![
        ("protocolVersion", Json::s(PROTOCOL_VERSION)),
        (
            "capabilities",
            Json::obj(vec![("tools", Json::obj(vec![("listChanged", Json::Bool(false))]))]),
        ),
        (
            "serverInfo",
            Json::obj(vec![
                ("name", Json::s(SERVER_NAME)),
                ("version", Json::s(SERVER_VERSION)),
            ]),
        ),
    ])
}

fn result_tools_list() -> Json {
    Json::obj(vec![(
        "tools",
        Json::Arr(vec![Json::obj(vec![
            ("name", Json::s("echo")),
            ("description", Json::s("Stamps the message onto the wire ring and reads it back.")),
            (
                "inputSchema",
                Json::obj(vec![
                    ("type", Json::s("object")),
                    (
                        "properties",
                        Json::obj(vec![(
                            "message",
                            Json::obj(vec![("type", Json::s("string"))]),
                        )]),
                    ),
                    ("required", Json::Arr(vec![Json::s("message")])),
                ]),
            ),
        ])]),
    )])
}

fn tools_call(params: &Json, ring: &mut WireRing) -> Result<Json, (f64, String, Option<Json>)> {
    let name = params.get("name").and_then(Json::as_str).unwrap_or("");
    if name.is_empty() {
        return Err((
            INVALID_PARAMS,
            "missing tool name".into(),
            None,
        ));
    }
    if name != "echo" {
        return Err((
            METHOD_NOT_FOUND,
            format!("unknown tool: {}", name),
            None,
        ));
    }
    let message = params
        .get("arguments")
        .and_then(|a| a.get("message"))
        .and_then(Json::as_str)
        .ok_or_else(|| {
            (
                INVALID_PARAMS,
                "echo requires arguments.message (string)".to_string(),
                None,
            )
        })?;
    let (ordinal, read_back) = echo(ring, message.as_bytes()).map_err(|e| {
        (
            INTERNAL_ERROR,
            format!("wire error: {}", e),
            Some(wire_error_data(&e)),
        )
    })?;
    let read_text = String::from_utf8(read_back).map_err(|_| {
        (
            INTERNAL_ERROR,
            "wire read-back was not valid UTF-8".to_string(),
            None,
        )
    })?;
    Ok(Json::obj(vec![
        (
            "content",
            Json::Arr(vec![Json::obj(vec![
                ("type", Json::s("text")),
                ("text", Json::Str(read_text)),
            ])]),
        ),
        (
            "wire",
            Json::obj(vec![("ordinal", Json::Num(ordinal as f64))]),
        ),
    ]))
}

/// echo through the wire ring: stamp at next ordinal, read back, verify.
fn echo(ring: &mut WireRing, message: &[u8]) -> Result<(u32, Vec<u8>), WireError> {
    let next = ring.next_ordinal()?;
    if next > u32::MAX as u64 {
        return Err(WireError::Io("ordinal space exhausted".into()));
    }
    let ordinal = next as u32;
    ring.stamp_frame(message, ordinal)?;
    let read_back = ring.read_frame(ordinal)?;
    if read_back != message {
        return Err(WireError::Io(format!(
            "echo mismatch: stamped {} bytes, read back {} bytes",
            message.len(),
            read_back.len()
        )));
    }
    Ok((ordinal, read_back))
}

/// Typed wireError data for -32603 internal errors.
fn wire_error_data(e: &WireError) -> Json {
    let kind = match e {
        WireError::FrameTooLarge { .. } => "FrameTooLarge",
        WireError::CrcMismatch { .. } => "CrcMismatch",
        WireError::StaleOrdinal { .. } => "StaleOrdinal",
        WireError::BadMagic { .. } => "BadMagic",
        WireError::Io(_) => "Io",
    };
    Json::obj(vec![
        ("kind", Json::s(kind)),
        ("detail", Json::Str(e.to_string())),
    ])
}

/// JSON-RPC error object builder.
pub fn error_response(id: Json, code: f64, msg: &str, data: Option<Json>) -> Json {
    let mut err = vec![
        ("code", Json::Num(code)),
        ("message", Json::Str(msg.to_string())),
    ];
    if let Some(d) = data {
        err.push(("data", d));
    }
    Json::obj(vec![
        ("jsonrpc", Json::s("2.0")),
        ("id", id),
        ("error", Json::obj(err)),
    ])
}
