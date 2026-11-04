# cgap-mcp

MCP-JSON <-> wire-shm binary-frame translation daemon (`cgap-mcpd`) for the
cgap v26.11.2 starDF campaign, Wave-3 bridge (pulled forward).

- Provenance: derived from the cgap v26.11.2 campaign
  (`docs/jira/v26.11.2/00-ard-prd.md`, wave plan
  `docs/jira/v26.11.2/05-wave-plan.md`, runbook `OVERNIGHT-LOOP.md`).
  NOTICE: this crate descends from cgap 26.11.2 (Apache-2.0).
- License: Apache-2.0. CalVer: version = 26.11.3 (calendar-versioned;
  yyyyy.mm.patch).
- Dependency law: `wire-shm = { path = "../wire-shm" }` path dep ONLY
  (zero other deps, std only). Wave-4 publish swaps to a version dep after
  wire-shm publishes first (coordinator-owned; ONE attempt per crate,
  registry slide backs off 1 hour).

## Wire frame format (wire-shm WIR0)

Big-endian, 16-byte fixed header + payload (slot stride 4096, slot 0 is a
4096-byte bookkeeping region holding `[u64 next_ordinal][u32 capacity]`):

```
[u32 magic 0x5749_5230 ("WIR0")] [u32 crc32(payload)] [u32 payload_len] [u32 frame_ordinal] [payload]
```

The segment is offset-addressed std::fs (FileExt), not mmap (zero-dep law).

## Protocol surface (JSON-RPC 2.0, line-delimited stdio)

- `initialize` -> {protocolVersion:"2024-11-05", capabilities:{tools:{listChanged:false}}, serverInfo:{name:"cgap-mcpd", version:"26.11.3"}}
- `tools/list` -> {tools:[{name:"echo", description, inputSchema}]}
- `tools/call` {name:"echo", arguments:{message}} -> stamps the message
  onto the WireRing at the ring's next ordinal, reads the frame back,
  verifies payload equality, returns
  {content:[{type:"text", text:<read-back>}], wire:{ordinal}}.
- Errors: -32700 parse, -32601 method-not-found, -32602 invalid-params
  (missing tool name / missing arguments.message), -32603 internal with
  typed wireError data {kind: FrameTooLarge|CrcMismatch|StaleOrdinal|BadMagic|Io, detail}.

## Real usage

```bash
$ echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
    | ./target/debug/cgap-mcpd /tmp/echo.ring
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"cgap-mcpd","version":"26.11.3"}}}
```

A client loop (one JSON-RPC line per request, responses line-delimited):

```python
import subprocess
d = subprocess.Popen(["cgap-mcpd", "/tmp/echo.ring"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
d.stdin.write('{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"echo","arguments":{"message":"hi"}}}\n')
d.stdin.flush()
print(d.stdout.readline())
```
