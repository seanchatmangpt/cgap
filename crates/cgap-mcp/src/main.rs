//! cgap-mcpd — MCP-JSON <-> wire-shm translation daemon (line-delimited JSON-RPC 2.0).
//!
//! Usage: cgap-mcpd [RING_PATH]
//!   RING_PATH defaults to $CGBP_MCP_RING, else /tmp/cgap-mcpd-<pid>.ring

use cgap_mcp::{handle_line, open_ring};
use std::io::{BufRead, Write};
use std::path::PathBuf;

fn main() {
    let ring_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::var("CGAP_MCP_RING").ok().map(PathBuf::from))
        .unwrap_or_else(|| std::env::temp_dir().join(format!("cgap-mcpd-{}.ring", std::process::id())));
    let mut ring = match open_ring(&ring_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cgap-mcpd: ring open failed: {}", e);
            std::process::exit(2);
        }
    };
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(resp) = handle_line(&line, &mut ring) {
            if writeln!(stdout, "{}", resp).is_err() {
                break;
            }
            let _ = stdout.flush();
        }
    }
}
