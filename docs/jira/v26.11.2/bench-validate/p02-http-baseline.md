# p02-http-baseline

Lane P-02 HTTP baseline leg, 2026-11-02. Repo `/Users/sac/cgap`, branch
main. MCP-style JSON-RPC echo exchange over localhost HTTP/1.1, measured
this session with a purpose-built zero-dependency harness (`std::net`
TCP only; no new crates, no lockfile churn). Deliverable per dispatch:
round-trip Min/Med/Max over N=100 plus per-envelope wire sizes,
headers and body counted separately and together.

## 1. Layer boundary (fence F1 - read first)

The ns/frame figures in this campaign live on a DIFFERENT LAYER than
this report. The 10.20 ns/frame row (11-measured-claims row 5) is
quiet-host ingress kernel admission, scoped quiet-host only (fence
F5); row 28 is its registered contention divergence; rows 30-32 are
M-CTQ stations on other instruments. NONE of those numbers is a
protocol round-trip figure, and none of the numbers below is an
admission-kernel figure. Any table that places the row-5 figure beside
this report's us figures MUST label the boundary: admission kernel vs
wire protocol round-trip, ns/frame vs us/round-trip, different
instruments, different workloads - not like-for-like, never summed
and never ratioed.

## 2. Harness

- Location: `/tmp/p02-harness` (scratch, not repo capital; binaries
  `p02-server`, `p02-client`; replay commands in section 5).
- Server: binds `127.0.0.1:0` (ephemeral; prints `PORT <n>` on stdout),
  one thread per connection, HTTP/1.1 keep-alive, parses POST with
  Content-Length, echoes a JSON-RPC 2.0 result object.
- Client: `POST / HTTP/1.1` with `Content-Type: application/json` and
  `Content-Length: 71`; body is one JSON-RPC 2.0 echo request (71 B).
  `TCP_NODELAY` set on every socket.
- Warmup: 20 unmeasured round-trips per leg (fresh connections).
- Zero dependencies; `cargo build --release` exit 0 in 2.58 s
  (rustc 1.101.0-nightly (12433c307 2026-10-24), macOS arm64).

## 3. Measured legs (2026-11-02, this session)

Ambient: loadavg 5.42 3.70 3.03 at leg start and at leg end (sysctl
vm.loadavg) - HOT AMBIENT, no quiet-host marker held. All figures
below are hot-ambient-scoped; they are not quiet-host figures.

| Leg | Connection pattern | N | Min (us) | Median (us) | Max (us) | Exit |
|---|---|---|---|---|---|---|
| keep-alive | one connection reused for all 100 round-trips | 100 | 41.166 | 53.084 | 72.083 | 0 |
| fresh | TCP connect per request (connect cost inside each sample) | 100 | 47.000 | 56.166 | 72.083 | 0 |

Keep-alive vs connect-per-request split, stated honestly: reusing the
connection saves ~5.9 us at the median (53.084 -> 56.166, ~11%) and
~5.8 us at the min; max is identical (72.083) in both legs. The
connect-per-request leg includes the localhost TCP handshake in each
timed sample; the keep-alive leg amortizes it across all 100.

## 4. Wire sizes (measured, per envelope)

| Envelope | Bytes |
|---|---|
| Request header block (request line + headers + terminating CRLF) | 88 |
| Request body (JSON-RPC echo request) | 71 |
| Request total wire | 159 |
| Response header block (status line + 3 headers + CRLF) | 72 |
| Response body (JSON-RPC result envelope) | 107 |
| Response total wire | 179 |
| Full exchange (request + response) | 338 |

Headers and body are counted separately and together per dispatch.
Response header weight depends on the minimal server here (3 header
fields); a real MCP server sends more headers and a larger wire size.
