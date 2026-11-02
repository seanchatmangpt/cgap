# Tutorial: run the courts

Learning-oriented: the cgap substrate is proved by *courts* — tests that
mutate a thing and demand a typed refusal, so a green run can never be
vacuous. In this tutorial you run the repo's own court suite and read one
verdict line by line.

## What you need

- The cgap repo cloned locally (`git clone` your fork, or the canonical
  checkout at `/Users/sac/cgap` if you are on this machine).
- Rust stable. No network needed after the first fetch.

## Step 1: run the suite

```console
$ cargo test -p cgap-court --test court_verdict
```

Witnessed output this session (from the repo root):

```text
running 14 tests
test a19_seal::a19_seal_1m_ledger_court ... ok
test a19_seal::a19_seal_1m_eval_court ... ok
test a14_dam_ingress_court ... ok
test a04_identity_court ... ok
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured;
0 filtered out; finished in 26.07s
```

Fourteen courts, all green, in about half a minute. `cgap-court` is the
court harness crate; the tests live in the repo root's `tests/`
directory.

## Step 2: read one verdict

Pick `a19_seal_1m_ledger_court` — the one your tutorial-02 flow scales
into. Its job: build a deterministic 1M-row corpus, push it through
seal -> commit -> footer read-back -> decode, and assert the receipt
matches at every stage. Its `ok` is only meaningful because the same file
contains courts that mutate and demand typed refusals; a suite where
every test passes regardless is vacuous and treated as a failure.

## Step 3: prove non-vacuity yourself

The repo ships fail-closed gates with self-checks. Run the zero-string
gate's own non-vacuity proof:

```console
$ python3 scripts/zero_string_gate.py --self-check
```

A gate that cannot refuse anything proves nothing; the self-check
corrupts a copy and requires the gate to refuse it. Exit codes follow the
repo contract: 0 pass, 1 gate fail, 2 subject missing, 3 loud skip.

Witnessed this session: the self-check ends
`SELF-CHECK RESULT PASS vocab=graph-derivation:...` with exit 0, and its
`[SKIP]` lines name their reason loudly (rustc metadata members are not
objects) — a skip in this repo is never a silent pass.

## What you just learned

- A green suite here is a *court verdict*, not a hope: mutation courts
  inside the suite prove the refusals are live.
- Exit codes are a contract: 0 pass / 1 gate fail / 2 subject missing /
  3 loud skip. A court that cannot flip is FAIL; a mutation-free green
  gate is vacuous.
- The `a19_seal` courts connect tutorial 02's toy scale to the
  1M-row ledger: same sealer, same footer, same verdict order.

## Where next

- How-to guides (`docs/diataxis/how-to/`) for specific tasks.
- Reference (`docs/diataxis/reference/`) for exact contracts.
- The wave receipts under `docs/jira/v26.11.1/waves/` are the ground
  truth for every command this tutorial cites.
