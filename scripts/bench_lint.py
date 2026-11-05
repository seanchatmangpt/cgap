#!/usr/bin/env python3
"""bench_lint.py — Tier-1 syntactic fabrication lint for Rust bench/court files.

Scans .rs files for fabrication-adjacent classes:
  LITERAL_ASSERT        assert bound is a bare numeric literal, unmatched by any
                        named const or plain-literal let in the same file
  DEAD_COURT (WARN)     literal bound differs from the file's own constant
                        arithmetic (e.g. 351.0 vs computed 350.989)
  TAUTOLOGY             variable assigned a literal then asserted against the
                        same literal in the same function
  PRINT_ASSERT_MISMATCH numeric literal in a print/format string vs the assert
                        bound in the same function differs by >= 10x
  UNCONDITIONAL_PASS    file prints "PASS" with zero executed asserts

Exit codes: 0 clean / 1 findings / 2 subject missing (fail-loud guard law —
refuses to fake-pass when a target path does not exist).
"""
import ast
import os
import re
import sys

FINDING_CLASSES = ("LITERAL_ASSERT", "DEAD_COURT", "TAUTOLOGY",
                   "PRINT_ASSERT_MISMATCH", "UNCONDITIONAL_PASS")

INT_SUFFIX = (r'(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize|'
              r'f32|f64)')
NUM_RE = re.compile(r'\d[\d_]*(?:\.\d+)?(?:' + INT_SUFFIX + r')?')

# ---------------------------------------------------------------- scanner ---
def scan_strings_and_strip(text):
    """Walk chars, strip comments; replace string/char literals with
    placeholders. Length-preserving: offsets in the returned code map 1:1
    onto the original text (true file:line reporting). Returns (code,
    strings) where strings is a list of (start_offset, content)."""
    out = []
    strings = []  # (offset_of_opening_quote, content)
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c == '/' and i + 1 < n and text[i + 1] == '/':
            j = i
            while j < n and text[j] != '\n':
                j += 1
            out.append(' ' * (j - i))
            i = j
            continue
        if c == '/' and i + 1 < n and text[i + 1] == '*':
            end = text.find('*/', i + 2)
            end = n if end < 0 else end + 2
            out.append(''.join(ch if ch == '\n' else ' '
                               for ch in text[i:end]))
            i = end
            continue
        if c == 'r' and text[i:i + 2] in ('r"', 'r#'):
            m = re.match(r'r(#*)"', text[i:])
            if m:
                hashes = m.group(1)
                closer = '"' + hashes
                body_start = i + m.end()
                end = text.find(closer, body_start)
                if end < 0:
                    end = n
                else:
                    strings.append((i, text[body_start:end]))
                    # blank the framing AND content, keep positions
                    out.append(' ' * (end + len(closer) - i))
                    i = end + len(closer)
                continue
        if c == 'b' and text[i:i + 2] in ('b"', "b'"):
            # byte string / byte char: treat like normal string, no capture
            q = text[i + 1]
            j = i + 2
            while j < n:
                if text[j] == '\\':
                    j += 2
                    continue
                if text[j] == q:
                    break
                j += 1
            out.append(' ' * (j + 1 - i))
            i = j + 1
            continue
        if c == '"':
            j = i + 1
            while j < n:
                if text[j] == '\\':
                    j += 2
                    continue
                if text[j] == '"':
                    break
                j += 1
            strings.append((i, text[i + 1:j]))
            out.append(' ' * (j + 1 - i))
            i = j + 1
            continue
        if c == "'":
            # char literal or lifetime — only short char literals get captured
            m = re.match(r"'(\\.|[^\\'])'", text[i:])
            if m:
                i += m.end()
                continue
        out.append(c)
        i += 1
    code = ''.join(out)
    return code, strings

# ------------------------------------------------------------- evaluation ---
def norm_num(expr):
    """Normalize Rust numeric tokens for Python evaluation."""
    expr = re.sub(r'(\d)[_]', r'\1', expr)
    expr = re.sub(r'(?<=\d)\s*' + INT_SUFFIX + r'\b', '', expr)
    return expr

def extract_literals(text):
    """Return set of float values of numeric literals in text (const-valued)."""
    vals = set()
    for m in NUM_RE.finditer(norm_num(text)):
        try:
            vals.add(float(m.group(0)))
        except ValueError:
            pass
    return vals

VERSION_RE = re.compile(r'\d+\.\d+\.\d+(?:\.\d+)?')  # e.g. v26.11.5
# prose bound: a decimal literal OR an integer with a unit suffix ("15 ns")
PRINT_BOUND_RE = re.compile(
    r'(\d[\d_]*\.\d+|\d[\d_]*\s*(?:ns|ms|us|µs|x|B|b|%)?)')

def print_literals(s):
    """Numeric literals cited in a print string's prose. Strips Rust format
    specifiers ({:.2}, {}/{}), version tokens, and unitless small integers
    (gate labels like [G1]) — keeps decimal bounds and unit-suffixed ints."""
    s = re.sub(r'\{[^}]*\}', ' ', s)
    s = VERSION_RE.sub(' ', s)
    vals = set()
    for m in PRINT_BOUND_RE.finditer(norm_num(s)):
        tok = m.group(0)
        if '.' not in tok and not re.search(r'(ns|ms|us|µs|x|B|b|%)$', tok):
            continue  # bare small integer (label noise)
        try:
            v = float(tok)
        except ValueError:
            continue
        if '.' not in tok and v < 10:
            continue
        vals.add(v)
    return vals

def try_eval(expr, env):
    """Safely evaluate +-*/ arithmetic over numbers and env names.
    Returns float or None."""
    expr = norm_num(expr).strip()
    if not expr or '"' in expr:
        return None
    try:
        node = ast.parse(expr, mode='eval').body
    except (SyntaxError, ValueError, MemoryError):
        return None
    return _ev(node, env)

def _ev(node, env):
    if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
        return float(node.value)
    if isinstance(node, ast.Name):
        return env.get(node.id)
    if (isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Sub,
            ast.Mult, ast.Div, ast.FloorDiv, ast.Mod, ast.Pow))):
        a = _ev(node.left, env)
        b = _ev(node.right, env)
        if a is None or b is None:
            return None
        try:
            if isinstance(node.op, ast.Add): return a + b
            if isinstance(node.op, ast.Sub): return a - b
            if isinstance(node.op, ast.Mult): return a * b
            if isinstance(node.op, ast.Div): return a / b
            if isinstance(node.op, ast.FloorDiv): return a // b
            if isinstance(node.op, ast.Mod): return a % b
            if isinstance(node.op, ast.Pow): return a ** b
        except (ZeroDivisionError, OverflowError):
            return None
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        v = _ev(node.operand, env)
        return None if v is None else -v
    if isinstance(node, ast.Call):
        # e.g. f64::from / as-casts already stripped; treat unknown calls as None
        return None
    return None

# ------------------------------------------------------------ rust parse ---
def line_of(text, offset):
    return text.count('\n', 0, offset) + 1

MACRO_SPAN = re.compile(
    r'\b(assert|assert_eq|assert_ne|println|print|eprintln|eprint|format|'
    r'panic)\s*!\s*')

def macro_spans(code, name):
    """Yield (start, end, arg_text) spans for `name!` macros via paren match."""
    for m in re.finditer(r'\b' + name + r'\s*!\s*\(', code):
        depth, j = 0, m.end() - 1
        start = m.end() - 1
        while j < len(code):
            if code[j] == '(':
                depth += 1
            elif code[j] == ')':
                depth -= 1
                if depth == 0:
                    yield (m.start(), j + 1, code[start + 1:j])
                    break
            j += 1

def functions(code):
    """Yield (start_offset, end_offset, body_text) for each `fn` in code."""
    for m in re.finditer(r'\bfn\s+\w+', code):
        brace = code.find('{', m.end())
        if brace < 0:
            continue
        depth, j = 0, brace
        while j < len(code):
            if code[j] == '{':
                depth += 1
            elif code[j] == '}':
                depth -= 1
                if depth == 0:
                    yield (m.start(), j + 1, code[brace + 1:j])
                    break
            j += 1

LET_RE = re.compile(r'\blet\s+(?:mut\s+)?([A-Za-z_]\w*)\s*(?::[^=;]+)?=')
CMP_RE = re.compile(r'(>=|<=|==|!=|>|<)')

# ------------------------------------------------------------ lint core ---
def lint_file(path, text):
    findings = []  # (line, class, message)
    code, strings = scan_strings_and_strip(text)
    lines = text.splitlines()

    # named consts (file scope)
    const_env = {}
    for m in re.finditer(r'\bconst\s+([A-Za-z_]\w*)\s*:[^=]*=\s*([^;]+);', code):
        const_env[m.group(1)] = try_eval(m.group(2), const_env)
    const_vals = {v for v in const_env.values() if v is not None}

    total_asserts = 0
    pass_print_lines = []
    print_literals_by_fn = {}

    for (fs, fe, body) in functions(code):
        fn_head = code[fs:code.find('{', fs)]
        env = dict(const_env)
        fn_findings = []

        # let bindings (sequential, statement-level approximation)
        lets = {}  # name -> (value_or_None, literal_or_None, line)
        for m in re.finditer(
                r'\blet\s+(?:mut\s+)?([A-Za-z_]\w*)\s*(?::[^=;]+)?=\s*([^;]+);',
                body):
            name, expr = m.group(1), m.group(2)
            # drop trailing `else`/block junk from if-let shapes
            expr = expr.split('}')[0].strip()
            lit = None
            litvals = extract_literals(expr)
            if len(litvals) == 1:
                lit = next(iter(litvals))
            val = try_eval(expr, env)
            env[name] = val
            lets[name] = (val, lit, line_of(text, fs + m.start()))

        # assertions
        for (ms, me, args) in macro_spans(code[fs:fe], 'assert'):
            args = args.replace('assert_eq', '').replace('assert_ne', '')
            total_asserts += 1
            cond = args.split(',', 1)[0] if '!' in code[fs + ms:fs + ms + 12] else args
            astart = fs + ms
            cond_text = args
            cond_text = CMP_RE.sub(' ', cond_text)
            # strip format-string remainder of the assert message
            lits = extract_literals(cond_text.split(',')[0] if ',' in cond_text and '"' not in cond_text.split(',')[0] else cond_text.split(',')[0])
            names = re.findall(r'\b[A-Za-z_]\w*\b', cond_text.split(',')[0])
            litvals = extract_literals(cond_text.split(',')[0])
            bound = next(iter(litvals)) if len(litvals) == 1 else None
            cond_line = line_of(text, astart)
            envvals = {nm: env.get(nm) for nm in names}
            # (a) LITERAL_ASSERT
            unmatched = {v for v in litvals
                         if v not in const_vals and not any(
                             lets.get(nm, (None, None, 0))[1] == v for nm in names)}
            for v in sorted(unmatched):
                fn_findings.append((cond_line, 'LITERAL_ASSERT',
                    'assert bound %.4g is a bare literal (no named const or '
                    'plain-literal let produces it)' % v))
            # (a2) DEAD_COURT warn: literal differs from computed arithmetic
            if bound is not None:
                for nm, val in envvals.items():
                    if val is not None and val != bound:
                        denom = max(abs(bound), 1e-9)
                        if abs(val - bound) / denom < 0.02:
                            fn_findings.append((cond_line, 'DEAD_COURT',
                                'WARN: literal bound %.4g differs from the '
                                'value %s = %.6g produced by the file\'s own '
                                'constant arithmetic (assert may be '
                                'unsatisfiable)' % (bound, nm, val)))
            # (b) TAUTOLOGY
            for nm, (val, lit, lline) in lets.items():
                if lit is not None and nm in names and lit in litvals:
                    fn_findings.append((cond_line, 'TAUTOLOGY',
                        "variable '%s' is assigned literal %.4g (line %d) "
                        'then asserted against the same literal' %
                        (nm, lit, lline)))

        # print/format strings inside this fn: numeric literals in prose
        for (soff, scontent) in strings:
            if not (fs <= soff < fe):
                continue
            prefix = code[max(0, soff - 30):soff]
            if not re.search(r'\b(println|print|eprintln|eprint|format|write|'
                             r'writeln)\s*!\s*\($', prefix):
                continue
            plits = print_literals(scontent)
            if plits:
                print_literals_by_fn.setdefault(fs, []).append(
                    (line_of(text, soff), plits))
            if re.search(r'\bPASS\b', scontent):
                pass_print_lines.append(line_of(text, soff))

        # (c) PRINT_ASSERT_MISMATCH (>= 10x divergence, warn-level)
        alits = set()
        for (ms, me, args) in macro_spans(code[fs:fe], 'assert'):
            first = args.split(',')[0]
            alits |= extract_literals(first)
        for (pline, plits) in print_literals_by_fn.get(fs, []):
            for p in plits:
                # fire only when NO assert bound in the fn corroborates p
                if alits and not any(p == a for a in alits):
                    near = min(alits, key=lambda a: abs(a - p))
                    if p >= 10 * near or near >= 10 * p:
                        fn_findings.append((pline, 'PRINT_ASSERT_MISMATCH',
                            'WARN: print string cites %.4g but the fn\'s '
                            'assert bound(s) are %.4g (>=10x divergence, no '
                            'matching bound)' % (p, near)))
        findings.extend(fn_findings)

    # (d) UNCONDITIONAL_PASS
    if pass_print_lines and total_asserts == 0:
        for pl in pass_print_lines:
            findings.append((pl, 'UNCONDITIONAL_PASS',
                'file prints PASS on a path with zero executed asserts '
                '(assert count = 0)'))

    return findings


def collect_rs_files(targets):
    files = []
    for t in targets:
        if os.path.isdir(t):
            for root, dirs, names in os.walk(t):
                dirs[:] = [d for d in dirs if d not in
                           ('target', '.git')]
                for nm in sorted(names):
                    if nm.endswith('.rs'):
                        files.append(os.path.join(root, nm))
        elif os.path.isfile(t) and t.endswith('.rs'):
            files.append(t)
        else:
            sys.stderr.write(
                'bench_lint: refusing to fake-pass: subject missing: %s\n' % t)
            sys.exit(2)
    return files


def main(argv):
    targets = argv[1:]
    if not targets:
        sys.stderr.write('usage: bench_lint.py <file.rs | dir> ...\n')
        return 2
    all_findings = []
    files = collect_rs_files(targets)
    for f in files:
        try:
            with open(f, 'r', encoding='utf-8', errors='replace') as fh:
                text = fh.read()
        except OSError as e:
            sys.stderr.write('bench_lint: unreadable %s: %s\n' % (f, e))
            sys.exit(2)
        for (line, cls, msg) in lint_file(f, text):
            all_findings.append('%s:%d: %s: %s' % (f, line, cls, msg))
    for fnd in all_findings:
        print(fnd)
    print('bench_lint: %d file(s) scanned, %d finding(s)' %
          (len(files), len(all_findings)), file=sys.stderr)
    return 1 if all_findings else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
