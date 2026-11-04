#!/bin/zsh
# PL15 kernel mutation court (non-vacuity): copy cgap-blake3 into a /tmp
# sandbox, flip a lane offset in the NEW kernel's message-schedule stride
# (lane 1's message load reads lane 2's words), and prove the equality
# court FLIPS to failure. sha256 fail-closed: the shared tree is only read;
# its byte state is recorded before and verified after.
set -eu
CRATE=/Users/sac/cgap/crates/cgap-blake3
SANDBOX=/tmp/pl15_mut_sandbox
rm -rf "$SANDBOX"; mkdir -p "$SANDBOX/src"
cp "$CRATE/Cargo.toml" "$SANDBOX/"
cp "$CRATE/src/lib.rs" "$CRATE/src/multi_chunk.rs" "$SANDBOX/src/"
cp -R "$CRATE/tests" "$SANDBOX/tests"
BEFORE=$(shasum -a 256 "$CRATE/src/multi_chunk.rs" "$CRATE/src/lib.rs" | awk '{print $1}' | tr -d '\n')
echo "tree-before: $BEFORE"
python3 - "$SANDBOX/Cargo.toml" <<'PY'
import sys, re
p = sys.argv[1]
t = open(p).read()
t = t.replace('rust-version.workspace = true', 'rust-version = "1.85"')
t = t.replace('version.workspace = true', 'version = "26.11.3"')
t = t.replace('edition.workspace = true', 'edition = "2021"')
t = t.replace('authors.workspace = true', 'authors = ["Sean Chatman <info@chatmangpt.com>"]')
t = t.replace('repository.workspace = true', 'repository = "https://github.com/seanchatmangpt/cgap"')
t = t.replace('license-file = "../../LICENSE.md"', 'license-file = "/Users/sac/cgap/LICENSE.md"')
t = re.sub(r'\[dev-dependencies\][^\[]*', '[dev-dependencies]\n', t, count=1)
open(p, 'w').write(t)
PY
python3 - "$SANDBOX/src/multi_chunk.rs" <<'PY'
import sys
p = sys.argv[1]
t = open(p).read()
marker = "unsafe fn hash_blocks_4way_independent_neon"
i = t.index(marker)
head, tail = t[:i], t[i:]
n = tail.count("vld1q_u8(bufs[1].as_ptr())")
assert n == 1, f"independent kernel must hold exactly 1 lane-1 load, got {n}"
tail = tail.replace("vld1q_u8(bufs[1].as_ptr())", "vld1q_u8(bufs[2].as_ptr())", 1)
open(p, "w").write(head + tail)
print("mutation applied: independent-kernel lane-1 load now reads lane-2 words")
PY
echo "--- sandbox court run (MUST FAIL) ---"
set +e
(cd "$SANDBOX" && cargo test --release --test independent_batch_courts > /tmp/pl15_mut_run.log 2>&1)
rc=$?
set -e
tail -6 /tmp/pl15_mut_run.log
if [ $rc -eq 0 ]; then
  echo "[COURT-VACUOUS] mutated kernel passed the equality court"
  exit 1
fi
echo "[MUTATION-FLIP] equality court failed on the mutated kernel (rc=$rc)"
echo "--- restore proof ---"
AFTER=$(shasum -a 256 "$CRATE/src/multi_chunk.rs" "$CRATE/src/lib.rs" | awk '{print $1}' | tr -d '\n')
if [ "$BEFORE" = "$AFTER" ]; then
  echo "[RESTORE-OK] shared tree byte-identical: $AFTER"
else
  echo "[RESTORE-FAIL] shared tree drifted"
  exit 1
fi
