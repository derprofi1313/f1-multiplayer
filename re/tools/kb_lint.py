#!/usr/bin/env python3
"""Lint the whole KB under re/db (schema, uniqueness, naming, evidence rules, cross references).

  kb_lint.py            errors exit 1; warnings are printed but do not fail
  kb_lint.py --strict   warnings fail too
  kb_lint.py --no-exe   skip the checks that read work/exe/F1_25.exe (string bytes, sections)
  kb_lint.py --stats    print counts by table and level
"""
import argparse, json, os, re, sys, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kb

PRIM = {'uint8_t': 1, 'int8_t': 1, 'char': 1, 'bool': 1, 'uint16_t': 2, 'int16_t': 2, 'uint32_t': 4, 'int32_t': 4,
        'float': 4, 'uint64_t': 8, 'int64_t': 8, 'double': 8, 'uintptr_t': 8, 'size_t': 8, 'void': 0}
FIELD_RE = re.compile(r'^\s*/\*\s*0x([0-9a-fA-F]+)\s*\*/\s*([^;]+);\s*//\s*(L[0-3])\b\s*(.*)$')
DECL_RE = re.compile(r'^(?P<type>[A-Za-z_][A-Za-z0-9_]*(?:\s+[A-Za-z_][A-Za-z0-9_]*)*?)\s*(?P<ptr>\**)\s*(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*(?:\[(?P<cnt>[^\]]+)\])?$')


def parse_struct(path):
    s = {'file': path, 'name': None, 'subsystem': None, 'size': None, 'size_level': None, 'size_ev': '', 'fields': [], 'errors': []}
    in_body = False
    for n, line in enumerate(open(path), 1):
        t = line.strip()
        m = re.match(r'^//\s*(struct|subsystem|size|notes):\s*(.*)$', t)
        if m and not in_body:
            k, v = m.groups()
            if k == 'struct':
                s['name'] = v.strip()
            elif k == 'subsystem':
                s['subsystem'] = v.strip()
            elif k == 'size':
                parts = v.split(None, 2)
                if parts and parts[0] != '?':
                    try:
                        s['size'] = int(parts[0], 16)
                    except ValueError:
                        s['errors'].append(f'{n}: size {parts[0]!r} not hex')
                if len(parts) > 1 and re.match(r'^L[0-3]$', parts[1]):
                    s['size_level'] = parts[1]
                    s['size_ev'] = parts[2].split()[0] if len(parts) > 2 else ''
            continue
        if re.match(r'^struct\s+\w+\s*\{', t):
            in_body = True
            if s['name'] and not t.startswith(f'struct {s["name"]} '):
                s['errors'].append(f'{n}: struct name differs from header')
            continue
        if t.startswith('};'):
            in_body = False
            continue
        if not in_body or not t or t.startswith('//'):
            continue
        m = FIELD_RE.match(line.rstrip('\n'))
        if not m:
            s['errors'].append(f'{n}: field line does not match "/* 0xOFF */ type name; // Ln evidence text"')
            continue
        off, decl, lvl, rest = m.groups()
        d = DECL_RE.match(decl.strip())
        if not d:
            s['errors'].append(f'{n}: cannot parse declaration {decl.strip()!r}')
            continue
        ev = rest.split()[0] if rest.split() and ':' in rest.split()[0] and rest.split()[0].split(':')[0] in kb.TOKEN_KINDS else ''
        cnt = d.group('cnt')
        array = cnt is not None
        try:
            cnt = int(cnt, 0) if cnt else 1
        except ValueError:
            s['errors'].append(f'{n}: array count {cnt!r}')
            cnt = 1
        s['fields'].append({'line': n, 'off': int(off, 16), 'type': d.group('type'), 'ptr': len(d.group('ptr')), 'name': d.group('name'),
                            'count': cnt, 'array': array, 'level': lvl, 'evidence': ev})
    return s


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--strict', action='store_true')
    ap.add_argument('--no-exe', action='store_true')
    ap.add_argument('--stats', action='store_true')
    a = ap.parse_args()
    E, W = [], []
    tables = {}
    for t in kb.TABLES:
        rows, errs = kb.read_table(t)
        tables[t] = rows
        E += errs
    subs = {r['id']: r for r in tables['subsystems']}
    for t, rows in tables.items():
        for r in rows:
            for e in kb.validate_row(t, r, subs):
                E.append(f'{t}.tsv:{r["_line"]}: {e}')
        seen = collections.Counter(kb.row_key(t, r) for r in rows if kb.TABLES[t][1])
        for k, c in seen.items():
            if c > 1:
                E.append(f'{t}.tsv: duplicate key {k} ({c} rows)')
    names = collections.Counter(r['name'] for t in ('functions', 'globals') for r in tables[t])
    for n, c in names.items():
        if c > 1:
            E.append(f'name {n} used {c} times across functions/globals')
    prefixes = collections.Counter(r['prefix'] for r in tables['subsystems'])
    for p, c in prefixes.items():
        if c > 1:
            E.append(f'subsystems.tsv: prefix {p} used {c} times')
    for sid in subs:
        md = os.path.join(kb.DB, 'subsystems', f'{sid}.md')
        if not os.path.exists(md):
            E.append(f'subsystems/{sid}.md missing')
        elif '\n## Inbox' not in open(md).read():
            W.append(f'subsystems/{sid}.md has no "## Inbox" section')

    anchors = {r['id'] for r in tables['strings_anchors']}
    tests = {f[:4] for f in os.listdir(os.path.join(kb.RE, 'tests')) if re.match(r'^T\d{3}\.md$', f)}
    sigdoc = json.load(open(os.path.join(kb.DB, 'signatures.json')))
    sigs = {s.get('id'): s for s in sigdoc.get('signatures', [])}

    def check_tokens(where, level, ev):
        for k, v in kb.tokens(ev):
            if k == 'decomp':
                if not os.path.exists(os.path.join(kb.ROOT, v)):
                    (E if level in ('L2', 'L3') else W).append(f'{where}: decomp file {v} missing')
            elif k in ('doc', 'kb'):
                p = v.split('#')[0]
                p = os.path.join(kb.DB, p) if k == 'kb' else os.path.join(kb.ROOT, p)
                if not os.path.exists(p):
                    E.append(f'{where}: {k} file {v} missing')
            elif k == 'str':
                if v not in anchors:
                    E.append(f'{where}: str:{v} not in strings_anchors.tsv')
            elif k == 'trace':
                m = re.match(r'^(T\d{3})/([A-Za-z0-9_.-]+)(?::\d+)?$', v)
                if not m:
                    E.append(f'{where}: trace token {v!r} must be T###/<run>[:line]')
                else:
                    if m.group(1) not in tests:
                        E.append(f'{where}: trace test {m.group(1)} has no re/tests/{m.group(1)}.md')
                    if not os.path.isdir(os.path.join(kb.RE, 'traces', m.group(1), m.group(2))):
                        E.append(f'{where}: trace dir re/traces/{m.group(1)}/{m.group(2)} missing')
            elif k == 'xref':
                for x in v.split('+')[0].split(','):
                    if not re.match(r'^0x1[0-9a-f]{8}$', x):
                        E.append(f'{where}: xref {x!r} malformed')
            elif k == 'imm':
                if not re.match(r'^0x[0-9a-f]+$', v):
                    E.append(f'{where}: imm {v!r} malformed')

    for t in ('functions', 'globals', 'db_keys'):
        for r in tables[t]:
            where = f'{t}.tsv:{r["_line"]} {r.get("name", r.get("path"))}'
            check_tokens(where, r['level'], r['evidence'])
            sig = r.get('sig', '-')
            if sig not in ('-', None):
                if sig not in sigs:
                    E.append(f'{where}: sig {sig} not in signatures.json')
                elif r['name'] not in sigs[sig].get('names', []):
                    E.append(f'{where}: signature {sig} does not list {r["name"]} in names[]')
            if 'DISPUTE(' in r['notes'] and r['level'] in ('L2', 'L3'):
                W.append(f'{where}: open DISPUTE on an {r["level"]} item')

    by_name = {r['name']: r for t in ('functions', 'globals') for r in tables[t]}
    req = ('id', 'section', 'pattern', 'match', 'anchor', 'resolve', 'target', 'kind', 'names', 'verified', 'used_by')
    if sigdoc.get('schema') != 1 or 'sha256' not in sigdoc.get('build', {}):
        E.append('signatures.json: schema must be 1 with build.sha256')
    for sid, s in sigs.items():
        for k in req:
            if k not in s:
                E.append(f'signatures.json {sid}: missing {k}')
        if not kb.SIG_ID_RE.match(sid or ''):
            E.append(f'signatures.json: id {sid!r} malformed')
        elif sid.split('.')[0] not in subs:
            E.append(f'signatures.json {sid}: subsystem {sid.split(".")[0]} not in subsystems.tsv')
        for n in s.get('names', []):
            if n not in by_name:
                E.append(f'signatures.json {sid}: name {n} not in functions/globals')
            elif by_name[n]['sig'] != sid:
                E.append(f'signatures.json {sid}: {n} has sig column {by_name[n]["sig"]!r}')
            elif int(by_name[n]['addr'], 16) != int(str(s.get('target', '0')), 0):
                E.append(f'signatures.json {sid}: target {s.get("target")} != {n} at {by_name[n]["addr"]}')
        for v in s.get('verified', []):
            if not all(k in v for k in ('build', 'date', 'by')):
                E.append(f'signatures.json {sid}: verified entry needs build/date/by')
        if s.get('kind') == 'mid':
            for n in s.get('names', []):
                if n in by_name and by_name[n]['level'] == 'L0':
                    E.append(f'signatures.json {sid}: hook site {n} is L0 (trace-only hooks need L1+)')

    structs = {}
    sdir = os.path.join(kb.DB, 'structs')
    for f in sorted(os.listdir(sdir)):
        if not f.endswith('.h'):
            continue
        s = parse_struct(os.path.join(sdir, f))
        structs[f[:-2]] = s
        where = f'structs/{f}'
        E += [f'{where}:{e}' for e in s['errors']]
        if s['name'] != f[:-2]:
            E.append(f'{where}: "// struct:" must equal the file name')
        if s['subsystem'] not in subs:
            E.append(f'{where}: subsystem {s["subsystem"]!r} not in subsystems.tsv')
        if s['size_level'] in ('L2', 'L3') and not s['size_ev']:
            E.append(f'{where}: size claimed {s["size_level"]} without evidence')
    for name, s in structs.items():
        where = f'structs/{name}.h'
        end = 0
        prev = None
        for fl in s['fields']:
            fw = f'{where}:{fl["line"]} {fl["name"]}'
            if prev and fl['off'] < end:
                E.append(f'{fw}: offset 0x{fl["off"]:x} overlaps {prev["name"]} (ends 0x{end:x})')
            if fl['ptr']:
                sz = 8
            elif fl['type'] in PRIM:
                sz = PRIM[fl['type']]
            elif fl['type'] in structs and structs[fl['type']]['size']:
                sz = structs[fl['type']]['size']
            else:
                sz = 0
                if fl['type'] not in structs:
                    W.append(f'{fw}: type {fl["type"]} is not a primitive or a structs/ file')
            end = fl['off'] + sz * fl['count']
            prev = fl
            if fl['level'] in ('L2', 'L3'):
                kinds = {k for k, _ in kb.tokens(fl['evidence'])}
                if 'decomp' not in kinds or not (kinds & {'xref', 'doc', 'kb'}):
                    E.append(f'{fw}: {fl["level"]} field needs decomp: plus xref:/doc:/kb:')
                if fl['level'] == 'L3' and 'trace' not in kinds:
                    E.append(f'{fw}: L3 field needs trace:')
            check_tokens(fw, fl['level'], fl['evidence'])
        if s['size'] is not None and end > s['size']:
            E.append(f'{where}: last field ends at 0x{end:x} beyond size 0x{s["size"]:x}')

    items = {('func', r['name']) for r in tables['functions']} | {('func', r['addr']) for r in tables['functions']}
    items |= {('global', r['name']) for r in tables['globals']} | {('global', r['addr']) for r in tables['globals']}
    items |= {('sig', s) for s in sigs} | {('struct', s) for s in structs} | {('anchor', x) for x in anchors}
    items |= {('field', f'{s}.{fl["name"]}') for s in structs for fl in structs[s]['fields']}
    for r in tables['verify']:
        if r['kind'] != 'db_key' and (r['kind'], r['item']) not in items:
            E.append(f'verify.tsv:{r["_line"]}: {r["kind"]} {r["item"]} not found')
    last = {}
    for r in tables['verify']:
        last[(r['kind'], r['item'])] = r['verdict']
    for k, v in last.items():
        if v == 'disagree':
            W.append(f'open disagreement on {k[0]} {k[1]}')

    for f in sorted(os.listdir(os.path.join(kb.RE, 'tests'))):
        if re.match(r'^T\d{3}\.md$', f):
            txt = open(os.path.join(kb.RE, 'tests', f)).read()
            for h in ('## Question', '## Hypothesis', '## Build', '## Trace config', '## User steps', '## Expected log lines', '## Result'):
                if h not in txt:
                    E.append(f'tests/{f}: missing section {h}')

    if not a.no_exe:
        try:
            sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
            from sigcheck import Image
            img = Image(os.path.join(kb.ROOT, 'work', 'exe', 'F1_25.exe'))
            for r in tables['strings_anchors']:
                if r['addr'] != '-' and r['enc'] == 'ascii':
                    b = img.read(int(r['addr'], 16), len(r['text'].encode()) + 1)
                    if b is None or not b.startswith(r['text'].encode()):
                        E.append(f'strings_anchors.tsv:{r["_line"]}: bytes at {r["addr"]} are not {r["text"]!r}')
            for t in ('functions', 'globals'):
                for r in tables[t]:
                    sec = img.sec_of(int(r['addr'], 16))
                    if sec is None:
                        E.append(f'{t}.tsv:{r["_line"]}: {r["addr"]} outside every section')
                    elif t == 'functions' and sec == '.data1' and not r['name'].endswith('_Body'):
                        E.append(f'{t}.tsv:{r["_line"]}: {r["name"]} lives in .data1; such bodies are named *_Body')
                    elif t == 'functions' and sec == '.data1' and r['sig'] != '-':
                        E.append(f'{t}.tsv:{r["_line"]}: .data1 bodies may not carry signatures')
        except ImportError as e:
            W.append(f'exe checks skipped ({e})')

    if True:
        lv = collections.Counter()
        for t in ('functions', 'globals', 'db_keys', 'strings_anchors'):
            for r in tables[t]:
                lv[(t, r['level'])] += 1
        fl = collections.Counter(fl['level'] for s in structs.values() for fl in s['fields'])
        print('KB: ' + '; '.join(f'{t} ' + ' '.join(f'{l}={lv[(t, l)]}' for l in kb.LEVELS) for t in ('functions', 'globals', 'db_keys', 'strings_anchors')))
        print(f'KB: structs {len(structs)} (fields ' + ' '.join(f'{l}={fl[l]}' for l in kb.LEVELS) + f'); signatures {len(sigs)}; subsystems {len(subs)}; verify rows {len(tables["verify"])}; tests {len(tests)}')
    for w in W:
        print('WARN ', w)
    for e in E:
        print('ERROR', e)
    print(f'kb_lint: {len(E)} error(s), {len(W)} warning(s)')
    return 1 if E or (a.strict and W) else 0


if __name__ == '__main__':
    sys.exit(main())
