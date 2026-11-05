#!/usr/bin/env python3
"""test_bench_lint_corpus.py — falsifier + corpus run for bench_lint.py.

1. FALSIFIER: planted literal-assert tautology fixture MUST fire (exit 1);
   clean named-const fixture MUST stay silent (exit 0); missing subject MUST
   exit 2 (fail-loud guard law — a lint that never fires is vacuous).
2. CORPUS: run the lint read-only against cgap crates/bench sources and
   print the actual findings verbatim (no fixing).

Exit 0 = falsifiers hold + corpus ran; nonzero = vacuous/failed lint.
"""
import subprocess
import sys
import tempfile
import os

LINT = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    'bench_lint.py')
BENCH_SRC = '/Users/sac/cgap/crates/bench/src'
SOVEREIGN = os.path.join(BENCH_SRC, 'bin/sovereign_stack_court.rs')
CSCT = os.path.join(BENCH_SRC, 'bin/csct_combinatorial_court.rs')

PLANTED = '''\
fn main() {
    let budget_ns = 1500.0;
    let iters = 10;
    let mut acc = 0u64;
    for _ in 0..iters {
        acc = acc.wrapping_add(1);
    }
    println!("  budget: < 1500.0 ns bound, acc: {}", acc);
    assert!(budget_ns < 1500.0, "fixture tautology");
    let a = 422_386.0;
    let b = 1_200.0;
    let roi = (a * 12.0 - b * 12.0) / (b * 12.0);
    assert!(roi >= 351.0, "dead court fixture");
    println!("=== FIXTURE: PASS ===");
}
'''

CLEAN = '''\
const LATENCY_BOUND_NS: f64 = 15.0;
const PRUNE_MIN_PCT: f64 = 40.0;

fn main() {
    let mut acc = 0u64;
    let iters = 100;
    for _ in 0..iters {
        acc = acc.wrapping_add(2);
    }
    let ns_per_hash = acc as f64 / iters as f64;
    println!("  hash: {:.2} ns/record (< {} ns bound)", ns_per_hash,
             LATENCY_BOUND_NS);
    assert!(ns_per_hash < LATENCY_BOUND_NS, "too slow");
    println!("=== CLEAN FIXTURE: PASS ===");
}
'''


def run(args):
    p = subprocess.run([sys.executable, LINT] + args,
                       capture_output=True, text=True)
    return p.returncode, p.stdout, p.stderr


def main():
    failures = []
    tmp = tempfile.mkdtemp(prefix='bench_lint_corpus_')

    # --- falsifier 1: planted tautology must fire ---
    f1 = os.path.join(tmp, 'planted_tautology.rs')
    with open(f1, 'w') as fh:
        fh.write(PLANTED)
    rc, out, _ = run([f1])
    print('--- falsifier: planted tautology (rc=%d) ---' % rc)
    print(out, end='')
    if rc != 1 or 'TAUTOLOGY' not in out:
        failures.append('falsifier-1: planted tautology did not fire '
                        'TAUTOLOGY (rc=%d)' % rc)

    # --- falsifier 2: clean named-const file must stay silent ---
    f2 = os.path.join(tmp, 'clean_named_const.rs')
    with open(f2, 'w') as fh:
        fh.write(CLEAN)
    rc, out, _ = run([f2])
    print('--- falsifier: clean named-const (rc=%d) ---' % rc)
    print(out, end='')
    if rc != 0:
        failures.append('falsifier-2: clean named-const file not silent '
                        '(rc=%d): %s' % (rc, out))

    # --- falsifier 3: missing subject must refuse to fake-pass ---
    rc, out, err = run([os.path.join(tmp, 'no_such_dir')])
    print('--- falsifier: missing subject (rc=%d) ---' % rc)
    print(err, end='')
    if rc != 2 or 'refusing to fake-pass' not in err:
        failures.append('falsifier-3: missing subject did not exit 2 '
                        'fail-loud (rc=%d)' % rc)

    # --- corpus: real crates/bench sources, read-only ---
    for subject in (SOVEREIGN, CSCT):
        if not os.path.isfile(subject):
            failures.append('corpus subject missing: %s' % subject)
            continue
        rc, out, err = run([subject])
        print('--- corpus: %s (rc=%d) ---' % (subject, rc))
        print(out, end='')
        print(err, end='')

    if failures:
        print('CORPUS-TEST: FAIL')
        for f in failures:
            print('  - %s' % f)
        return 1
    print('CORPUS-TEST: PASS (falsifiers fire/silent/fail-loud as specified; '
          'corpus scanned read-only)')
    return 0


if __name__ == '__main__':
    sys.exit(main())
