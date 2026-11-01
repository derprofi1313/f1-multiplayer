#!/usr/bin/env python3
import argparse, datetime, hashlib, json, os, re, struct, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
DEFAULT_SIGS = os.path.join(ROOT, 're', 'db', 'signatures.json')

REGS = {'rax', 'rbx', 'rcx', 'rdx', 'rsi', 'rdi', 'rbp', 'rsp', 'rflags'} | {f'r{i}' for i in range(8, 16)} | {f'xmm{i}' for i in range(16)}
SCALAR = {'u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64', 'f32', 'f64', 'ptr', 'bool'}
BUFFER = {'hex', 'cstr', 'wstr', 'nestr'}
HOOK_ID = re.compile(r'^[a-z0-9_.]{1,63}$')
TEST_ID = re.compile(r'^T\d{3}$')
FIELD_NAME = re.compile(r'^[A-Za-z_][A-Za-z0-9_.]{0,30}$')
LIMITS = dict(hooks=64, fields=24, when=4, regs=24, args=8, bt=32, read=1024)


class BuildError(Exception):
    pass


def num(v):
    if isinstance(v, int):
        return v
    if isinstance(v, str):
        return int(v, 0)
    raise BuildError(f'not a number: {v!r}')


def check_expr(s, func_start):
    toks = re.findall(r'0x[0-9a-fA-F]+|\d+|[a-z][a-z0-9]*|[\[\]+\-]|\S', s)
    pos = 0

    def term():
        nonlocal pos
        if pos >= len(toks):
            raise BuildError(f'expression {s!r}: unexpected end')
        t = toks[pos]
        pos += 1
        if t == '[':
            expr()
            if pos >= len(toks) or toks[pos] != ']':
                raise BuildError(f'expression {s!r}: missing ]')
            pos += 1
            return
        if t in REGS:
            return
        m = re.fullmatch(r'a(\d)', t)
        if m:
            if int(m.group(1)) >= 4 and not func_start:
                raise BuildError(f'expression {s!r}: stack args a4+ only make sense at kind func_start')
            return
        if re.fullmatch(r'0x[0-9a-fA-F]+|\d+', t):
            return
        raise BuildError(f'expression {s!r}: unknown token {t!r}')

    def expr():
        nonlocal pos
        term()
        while pos < len(toks) and toks[pos] in '+-':
            pos += 1
            if pos >= len(toks) or not re.fullmatch(r'0x[0-9a-fA-F]+|\d+', toks[pos]):
                raise BuildError(f'expression {s!r}: + and - take a number')
            pos += 1

    expr()
    if pos != len(toks):
        raise BuildError(f'expression {s!r}: trailing {toks[pos:]}')


def norm_type(t):
    m = re.fullmatch(r'([a-z0-9]+)(?:\[(0x[0-9a-fA-F]+|\d+)\])?', t)
    if not m or m.group(1) not in SCALAR | BUFFER:
        raise BuildError(f'unknown type {t!r}')
    if m.group(2) is not None:
        if m.group(1) in SCALAR:
            raise BuildError(f'type {t!r}: only hex/cstr/wstr/nestr take a length')
        n = int(m.group(2), 0)
        if not 1 <= n <= LIMITS['read']:
            raise BuildError(f'type {t!r}: length must be 1..{LIMITS["read"]}')
    return t


def norm_field(f, func_start):
    if isinstance(f, str):
        m = re.fullmatch(r'([^=]+)=([^:]+):(.+)', f)
        if not m:
            raise BuildError(f'field {f!r}: use name=type:expr')
        f = {'name': m.group(1).strip(), 'type': m.group(2).strip(), 'expr': m.group(3).strip()}
    if not FIELD_NAME.match(f.get('name', '')):
        raise BuildError(f'field name {f.get("name")!r} invalid')
    out = {'name': f['name'], 'type': norm_type(f.get('type', 'u64')), 'expr': f['expr']}
    if 'len' in f:
        n = num(f['len'])
        if not 1 <= n <= LIMITS['read']:
            raise BuildError(f'field {f["name"]}: len must be 1..{LIMITS["read"]}')
        out['len'] = n
    check_expr(out['expr'], func_start)
    return out


def norm_cond(c, func_start):
    m = re.fullmatch(r'\s*(?:([a-z0-9]+):)?(.+?)\s*(==|!=|<=|>=|<|>|&)\s*(-?(?:0x[0-9a-fA-F]+|\d+))\s*', c)
    if not m:
        raise BuildError(f'when {c!r}: use [type:]expr OP number')
    if m.group(1) and m.group(1) not in SCALAR:
        raise BuildError(f'when {c!r}: type must be scalar')
    check_expr(m.group(2), func_start)
    return c.strip()


def load_sigs(path):
    with open(path) as f:
        db = json.load(f)
    return db, {s['id']: s for s in db.get('signatures', [])}


def runtime_sig(sid, s, build_prefix, warnings, raw=False):
    kind = s.get('kind', 'mid')
    resolve = dict(s.get('resolve') or {'kind': 'addr'})
    anchor = num(s.get('anchor', 0))
    if kind == 'call':
        if resolve.get('kind') != 'rel32':
            raise BuildError(f'signature {sid}: kind call needs resolve rel32')
        hook_kind = 'func_start'
    elif kind in ('mid', 'func_start'):
        if resolve.get('kind', 'addr') != 'addr':
            raise BuildError(f'signature {sid}: kind {kind} needs resolve addr')
        hook_kind = kind
    else:
        raise BuildError(f'signature {sid}: kind {kind!r} is not code; cannot hook it')
    if not raw:
        if s.get('section') != '.xtls':
            raise BuildError(f'signature {sid}: section {s.get("section")!r}; trace hooks are .xtls only')
        if build_prefix and not any(build_prefix.startswith(v.get('build', '-')) or v.get('build', '').startswith(build_prefix[:16])
                                    for v in s.get('verified', [])):
            warnings.append(f'signature {sid} has no verified entry for build {build_prefix[:16]}')
    out = {'id': sid, 'section': s.get('section', '.xtls'), 'pattern': s['pattern'], 'anchor': anchor,
           'resolve': {'kind': resolve.get('kind', 'addr')}}
    if s.get('expect') and kind != 'call':
        out['expect'] = s['expect']
    if s.get('target'):
        out['rva'] = hex(num(s['target']) - IMAGE_BASE)
    return hook_kind, out


IMAGE_BASE = 0x140000000


def exe_build(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        head = f.read(4096)
        h.update(head)
        for chunk in iter(lambda: f.read(1 << 22), b''):
            h.update(chunk)
    pe = struct.unpack_from('<I', head, 0x3c)[0]
    stamp = struct.unpack_from('<I', head, pe + 8)[0]
    return {'exe': os.path.basename(path), 'sha256': h.hexdigest(), 'size': os.path.getsize(path), 'pe_timestamp': stamp}


def build(args):
    global IMAGE_BASE
    db, sigs = load_sigs(args.signatures)
    b = dict(db.get('build', {}))
    IMAGE_BASE = num(b.get('image_base', '0x140000000'))
    if args.build_from_exe:
        b = exe_build(args.build_from_exe)
    warnings = []
    out = {'schema': 1, 'generated_by': 're/tools/trace_build.py',
           'generated': datetime.datetime.now().isoformat(timespec='seconds'),
           'signatures_file': os.path.relpath(os.path.abspath(args.signatures), ROOT),
           'tests': [], 'sources': [],
           'build': {k: b[k] for k in ('exe', 'sha256', 'size', 'pe_timestamp') if k in b},
           'check_sha256': not args.no_sha256, 'allow_build_mismatch': args.allow_build_mismatch,
           'allowed_sections': ['.xtls'],
           'log': {'max_mb': 64, 'on_full': 'stop', 'flush_ms': 100, 'summary_every_s': 60},
           'limits': {'max_events_total': 200000, 'max_read': 64}, 'hooks': []}
    seen = set()
    for path in args.author:
        with open(path) as f:
            a = json.load(f)
        test = a.get('test')
        if test is not None and not TEST_ID.match(test):
            raise BuildError(f'{path}: test id {test!r} must be T###')
        if test:
            out['tests'].append(test)
        out['sources'].append(os.path.relpath(os.path.abspath(path), ROOT))
        for k in ('log', 'limits'):
            for kk, v in a.get(k, {}).items():
                if kk in out[k] and out[k][kk] != v and len(out['sources']) > 1:
                    warnings.append(f'{path}: {k}.{kk} overrides {out[k][kk]!r} with {v!r}')
                out[k][kk] = v
        if a.get('allowed_sections'):
            if any(x not in ('.xtls', '.text') for x in a['allowed_sections']):
                raise BuildError(f'{path}: allowed_sections may only name code sections .xtls/.text, never .data1')
            out['allowed_sections'] = a['allowed_sections']
            warnings.append(f'{path}: allowed_sections overridden to {a["allowed_sections"]}')
        for h in a.get('hooks', []):
            hid = h.get('id', '')
            if not HOOK_ID.match(hid):
                raise BuildError(f'{path}: hook id {hid!r} must match {HOOK_ID.pattern}')
            if hid in seen:
                raise BuildError(f'{path}: duplicate hook id {hid}')
            seen.add(hid)
            if 'raw_sig' in h:
                warnings.append(f'hook {hid}: raw_sig is not from the KB (test/ad-hoc only)')
                kind, sig = runtime_sig(h['raw_sig']['id'], h['raw_sig'], None, warnings, raw=True)
            else:
                sid = h.get('sig')
                if sid not in sigs:
                    raise BuildError(f'hook {hid}: signature {sid!r} not in {args.signatures}')
                kind, sig = runtime_sig(sid, sigs[sid], b.get('sha256'), warnings)
            kind = h.get('kind', kind)
            fs = kind == 'func_start'
            rh = {'id': hid, 'test': h.get('test', test or '-'), 'kind': kind, 'sig': sig}
            regs = h.get('regs', [])
            bad = [r for r in regs if r not in REGS]
            if bad or len(regs) > LIMITS['regs']:
                raise BuildError(f'hook {hid}: bad regs {bad or regs}')
            if regs:
                rh['regs'] = regs
            if h.get('args'):
                if not fs or not 0 <= h['args'] <= LIMITS['args']:
                    raise BuildError(f'hook {hid}: args need kind func_start and 0..{LIMITS["args"]}')
                rh['args'] = h['args']
            fields = [norm_field(f, fs) for f in h.get('fields', [])]
            if len(fields) > LIMITS['fields']:
                raise BuildError(f'hook {hid}: more than {LIMITS["fields"]} fields')
            if len({f['name'] for f in fields}) != len(fields):
                raise BuildError(f'hook {hid}: duplicate field names')
            if fields:
                rh['fields'] = fields
            when = [norm_cond(c, fs) for c in h.get('when', [])]
            if len(when) > LIMITS['when']:
                raise BuildError(f'hook {hid}: more than {LIMITS["when"]} when conditions')
            if when:
                rh['when'] = when
            if h.get('ret'):
                if not fs:
                    raise BuildError(f'hook {hid}: ret needs kind func_start')
                rh['ret'] = True
            if h.get('bt'):
                if not 0 < h['bt'] <= LIMITS['bt']:
                    raise BuildError(f'hook {hid}: bt must be 1..{LIMITS["bt"]}')
                rh['bt'] = h['bt']
            rh['max_per_sec'] = h.get('max_per_sec', 50)
            rh['max_events'] = h.get('max_events', 2000)
            for k in ('enabled', 'skip_boundary_check'):
                if k in h:
                    rh[k] = h[k]
            out['hooks'].append(rh)
    if len(out['hooks']) > LIMITS['hooks']:
        raise BuildError(f'{len(out["hooks"])} hooks; the plugin installs at most {LIMITS["hooks"]}')
    if not out['build'] and not args.allow_build_mismatch:
        raise BuildError('no build block in the signatures file; pass --build-from-exe or --allow-build-mismatch')
    return out, warnings


def main():
    ap = argparse.ArgumentParser(description='Build a runtime swofty/trace.json from authoring configs and the KB signatures.')
    ap.add_argument('author', nargs='+', help='authoring configs (re/tests/T###.trace.json); several are merged into one session config')
    ap.add_argument('-o', '--out', default='-', help='output path (default stdout)')
    ap.add_argument('--signatures', default=DEFAULT_SIGS)
    ap.add_argument('--build-from-exe', help='take the build block (sha256/size/timestamp) from this exe instead of signatures.json')
    ap.add_argument('--allow-build-mismatch', action='store_true')
    ap.add_argument('--no-sha256', action='store_true', help='plugin checks size+timestamp only (skips hashing the exe at startup)')
    args = ap.parse_args()
    try:
        out, warnings = build(args)
    except (BuildError, KeyError, ValueError) as e:
        print(f'trace_build: error: {e}', file=sys.stderr)
        return 1
    for w in warnings:
        print(f'trace_build: warning: {w}', file=sys.stderr)
    text = json.dumps(out, indent=1) + '\n'
    if args.out == '-':
        sys.stdout.write(text)
    else:
        with open(args.out, 'w') as f:
            f.write(text)
        print(f'trace_build: {len(out["hooks"])} hook(s) from {len(args.author)} file(s) -> {args.out}', file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())
