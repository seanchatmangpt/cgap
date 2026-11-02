# research-motherduck

R-2 research receipt: MotherDuck and its format relationship to Feather/Arrow.
Lane: R2-motherduck. Date: 2026-11-01. Web research only; no code changes.
All URLs fetched this session via curl (HTTP 200 unless noted).

## 1. What MotherDuck is

MotherDuck is a serverless cloud data warehouse built on DuckDB, launched into
public preview 2023-06-22 ("Announcing MotherDuck: Hybrid Execution Scales
DuckDB from your Laptop into the Cloud",
https://motherduck.com/blog/announcing-motherduck-duckdb-in-the-cloud/).
Stealth funding announcement Nov 15, 2022, $47.5M (a16z, Redpoint, Madrona,
Amplify, Altimeter): https://motherduck.com/blog/announcing-series-seed-and-a/.

Architecture (CIDR 2024 paper "MotherDuck: DuckDB in the Cloud and in the
Client", RJ Atwal, Peter Boncz, Ryan Boyd, Antony Courtney, Till Dohmen,
Florian Gerlinghoff, Jeff Huang, Joseph Hwang, Raphael Hyde, Elena Felder,
Jacob Lacouture, Yves LeMaout, Boaz Leskes, Yao Liu, Alex Monahan, Dan
Perkins, Tino Tereshko, Jordan Tigani, Nick Ursa, Stephanie Wang, Yannick
Welsch; https://www.cidrdb.org/cidr2024/papers/p46-atwal.pdf, landing page
https://motherduck.com/research/motherduck-duckdb-in-the-cloud-and-in-the-client/):

- Compute: short-lived per-user containers ("ducklings"), each running a
  single DuckDB instance; RAM/CPU resized dynamically; containers shut down
  when idle (serverless).
- Storage: object storage (S3-class); a "duckling storage extension" maps
  byte-ranges of DuckDB's single-file storage onto multiple files; changed
  data written as a "mutation tree" (differential storage), enabling
  zero-copy duplication, sharing, branching, time travel, and database
  SHARE snapshots (read-only ATTACH by other ducklings).
- Hybrid/Dual Execution: client DuckDB (including DuckDB-Wasm in the
  browser) and cloud duckling jointly plan and execute one query; stages
  routed to local or cloud depending on where data lives.

Docs statement of the same model ("Dual Execution", formerly Hybrid
Execution; `ATTACH 'md:';` one-line connect; six AWS regions; DuckDB client
versions 1.4.0-1.5.6):
https://motherduck.com/docs/concepts/architecture-and-capabilities/ (fetched
as markdown at https://motherduck.com/docs/<path>.md).

## 2. Storage format: NOT Feather-derived

The format is DuckDB-native, adapted for object storage; no Feather or
Arrow IPC anywhere in the storage path:

- CIDR 2024 paper (URL above): grep of the full text shows zero occurrences
  of "Feather"; only 2 occurrences of "Arrow", both in the related-work
  citation about DuckDB's zero-copy pandas/Arrow integration (cites
  bit.ly/duckdb-blog-arrow), not storage. Direct quote on format:
  "Whereas DuckDB's native data format stores compressed columnar data in a
  single file, the duckling storage extension allows for differential
  storage, where changed data is stored independently as a mutation tree.
  ... seamlessly maps byte-ranges of DuckDB's single-file storage onto
  multiple files, which are then further mapped onto storage resources,
  such as cloud blob storage and ephemeral SSD instance storage."
- MotherDuck's own storage-lifecycle doc distinguishes "native storage
  databases" from DuckLake databases (both billed per lifecycle stage):
  https://motherduck.com/docs/concepts/storage-lifecycle/
- MotherDuck's Feather glossary entry describes Feather V2 as "simply the
  Apache Arrow IPC (interprocess communication) file format" — a separate,
  interchange-oriented format, with no lineage claim to MotherDuck's
  storage: https://motherduck.com/glossary/feather-format/

So: DuckDB-native format (single-file layout byte-mapped onto object
storage + differential mutation trees), i.e. proprietary-but-DuckDB-native,
with DuckLake (open table format, Parquet data files + database-backed
metadata catalog) as the open escape hatch:
https://motherduck.com/docs/concepts/ducklake/.

## 3. Naming history: no Feather lineage claim

- The name was coined by Lloyd Tabb (Looker founder): "Everyone asks about
  the name. Lloyd Tabb, the founder of Looker, has the distinction of
  naming MotherDuck. 'Trust me, it is a good name,' ... 'And the domain
  name is available.'" Origin story names Jordan Tigani (co-founder, ex-
  Google Dremel/BigQuery) and the DuckDB Labs partnership (Hannes
  Muhleisen, Mark Raasveldt) as "a co-founder of the new endeavor":
  https://motherduck.com/about-us/
- No founder blog statement, press release, or page fetched this session
  links the MotherDuck name to the Feather format. The founding story is
  BigQuery/Dremel (Tigani) + DuckDB/CWI (DuckDB Labs). Wes McKinney
  (Feather/Arrow creator) appears in MotherDuck content only as a 2026
  interview guest, described as founder of Kenn Software and Posit
  contributor — no MotherDuck role claimed:
  https://motherduck.com/blog/vibe-coding-dangerous-agentic-engineering-wes-mckinney/
  His own site front page and archive show no MotherDuck affiliation:
  https://wesmckinney.com/
- Verdict: the plausible hypothesis "MotherDuck is Feather-lineage by
  founder heritage" is REFUTED on the evidence fetched. The shared thread
  is columnar layout, not people or format.

## 4. Ingest/export paths and where Feather/Arrow IPC actually appears

Ingestion surface (docs): https://motherduck.com/docs/integrations/ingestion/
- ATTACH 'md:' from any local DuckDB client (Python/CLI/Rust/Go/JDBC) — the
  primary path; local DuckDB then COPY/INSERT into md: databases.
- Managed ingestion partners (dltHub first-class destination, Fivetran,
  Airbyte, Estuary, Kafka via Iceberg, Spark via DuckLake connector or
  Parquet exchange): same URL.
- S3/cloud storage query-in-place via httpfs (server-side httpfs supported,
  S3/GCS/Azure/R2; CREATE SECRET for creds):
  https://motherduck.com/docs/concepts/architecture-and-capabilities/ and
  https://motherduck.com/docs/integrations/cloud-storage/amazon-s3/
- DuckLake managed databases: Parquet data files on object storage with a
  database-backed metadata catalog; Spark connector writes via DuckLake:
  https://motherduck.com/docs/concepts/ducklake/
- Postgres wire endpoint for BI tools:
  https://motherduck.com/docs/getting-started/interfaces/postgres-endpoint/

Where Feather/Arrow IPC explicitly touches MotherDuck:
- Server-side extension matrix (MD Cloud column) has NO `arrow` extension
  entry; arrow/feather reading is not listed as a cloud-supported
  extension, while parquet, iceberg, delta, ducklake ARE:
  https://motherduck.com/docs/concepts/duckdb-extensions/
- Therefore Arrow IPC/Feather exchange happens client-side: a local DuckDB
  (where "all DuckDB extensions are available") reads/writes Feather via
  the arrow extension, then COPYs into md:; or a DuckDB Python client
  passes Arrow tables over the zero-copy DuckDB<->Arrow scan interface.
  MotherDuck's glossary confirms the DuckDB read path (`INSTALL arrow;
  LOAD arrow; SELECT * FROM read_arrow('data.arrow')`): 
  https://motherduck.com/glossary/feather-format/
- Export: "Materialize query results into local or MotherDuck locations, or
  S3" and "Copy DuckDB databases between local and MotherDuck locations"
  (COPY TO Parquet to S3 is the documented lake-export path):
  https://motherduck.com/docs/concepts/architecture-and-capabilities/

## 5. Current state (as of 2026-11-01)

- DuckDB client versions supported: 1.4.0/1.4.1 through 1.5.6 (regions:
  us-east-1, us-west-2, eu-central-1, eu-west-1, ap-northeast-1,
  ap-southeast-2); six AWS regions only ("We are working on expanding").
  https://motherduck.com/docs/concepts/architecture-and-capabilities/
- Benchmark footnote confirms current cloud runtime: "MotherDuck instances
  (Mega or Giga) running DuckDB version 1.5.6" (TPC-DS 1k/10k):
  https://motherduck.com/blog/bigger-ducks/
- Capability set: hypertenancy per-user ducklings (sub-100ms cold start
  claimed), Dual Execution, database shares/snapshots/time travel,
  zero-copy clones, managed DuckLake, Flights (scheduled Python), Dives
  (dashboards), Postgres endpoint, MCP server, Bring Your Own Compute
  (private region in customer AWS account):
  https://motherduck.com/docs/concepts/architecture-and-capabilities/
  https://motherduck.com/docs/about-motherduck/
- Storage lifecycle: 5 stages (active/historical/retained-for-clone/
  failsafe/deleted); snapshot retention 0-90 days by plan; failsafe 7 days.
  https://motherduck.com/docs/concepts/storage-lifecycle/
- Stated limitations (relevant to a substrate): no custom Python/native
  UDFs server-side, no server-side attach of postgres/sqlite, no custom or
  community extensions in the cloud:
  https://motherduck.com/docs/concepts/architecture-and-capabilities/

## Implications for cgap/stardf (DuckDB-queryable columnar substrate)

MotherDuck is not a Feather-derived format and never claimed to be: its
storage is DuckDB's own single-file columnar layout, byte-range-mapped onto
object storage with differential mutation trees, so any cgap ambition to be
"DuckDB/MotherDuck-queryable" should target the actual interchange seams
DuckDB speaks — Parquet files (universally supported server-side and the
data-file layer of DuckLake), the Arrow ADBC/IPC client path (client-side
only; MD Cloud exposes no arrow extension), or a DuckLake catalog with
Parquet data files, which is MotherDuck's own open, self-described
"petabyte-scale" on-ramp and the only open-table-format path that gives
both DuckDB and MotherDuck native queryability without uploading into
proprietary differential storage. For cgap's Arrow-kernel substrate the
practical consequence: write Arrow C-Data-Interface-parity columns as
Parquet (optionally under DuckLake metadata) for cloud queryability, keep
Feather/Arrow IPC as a local-interchange lane only, and treat DuckDB's
native .duckdb file format as a possible additional export target rather
than an interchange contract — MotherDuck adapts it privately (mutation
trees, byte-range mapping), so byte-compatibility with cloud-side storage
is not a public contract. One correction to prior-session assumptions
recorded here so it is not re-invented: Wes McKinney has no evidenced
MotherDuck role; the founder lineage is Tigani (BigQuery) + DuckDB Labs
(CWI).

## Method note

All sources fetched live this session via curl; the CIDR paper grepped via
pdftotext of https://www.cidrdb.org/cidr2024/papers/p46-atwal.pdf. MotherDuck
docs were fetched as server-rendered markdown (append .md to any
motherduck.com/docs/ path). Standing: all claims above are grounded in the
cited fetched pages; no claim rests on session memory alone except where
explicitly marked as a refuted hypothesis (Section 3 McKinney lineage).

Final JSON:
{"lane":"R2-motherduck","files_written":["docs/jira/v26.11.1/waves/research-motherduck.md"],"commands":["curl -sL https://motherduck.com/blog/ (200)","curl -sL https://www.cidrdb.org/cidr2024/papers/p46-atwal.pdf (200, pdftotext ok)","curl -sL https://motherduck.com/docs/concepts/architecture-and-capabilities.md (200)","curl -sL https://motherduck.com/docs/concepts/storage-lifecycle.md (200)","curl -sL https://motherduck.com/docs/concepts/duckdb-extensions.md (200)","curl -sL https://motherduck.com/docs/concepts/ducklake.md (200)","curl -sL https://motherduck.com/glossary/feather-format/ (200)","curl -sL https://motherduck.com/about-us/ (200)","curl -sL https://motherduck.com/blog/announcing-series-seed-and-a/ (200)","curl -sL https://motherduck.com/blog/vibe-coding-dangerous-agentic-engineering-wes-mckinney/ (200)"],"courts_run":{},"standing":"ALIVE","divergences":["Lane prompt hypothesized a MotherDuck-Feather founder lineage; fetched evidence (about-us origin story, Wes McKinney 2026 interview bio, wesmckinney.com) refutes it — recorded as a finding, not a divergence from the lane doc (no lane doc exists for R-2 beyond the dispatch prompt)."]}
