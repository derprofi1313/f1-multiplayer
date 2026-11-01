#!/usr/bin/env python3
"""Generate mod bindings from re/db, applying the export gate (research/mixin-design.md, PROTOCOL.md §2).

  bindgen.py                 write bindings.json, bindings.<build>.json, the C++ headers and the Rust module
  bindgen.py --check         exit 1 if any output is stale (the sets and symbols.tsv only once native/dist exists)
  bindgen.py --no-exe        skip the sigcheck pass over the KB exe
  bindgen.py --exe PATH      run the sigcheck pass over PATH (default work/exe/F1_25.exe)

symbols.tsv (every KB function name by rva) is a crash-log label table, not a binding: it is not gated.

Every re/db/builds/<build>.json overlay yields one bindings set from the same gated bindings: the overlay
whose build is the KB build writes bindings.json, every other one writes bindings.<build>.json beside it.

Gate: L3 whose trace test passed -> stable; L2 (or L3 without a passed test) with an `agree` as the
latest verify.tsv verdict -> experimental; everything else is not exported. Runtime bindings need a
signature verified on the KB build; struct fields go through the same gate.
"""
import argparse, json, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kb
from kb_lint import parse_struct, PRIM
from sigcheck import MIN_LEN, parse_pattern

ROOT = kb.ROOT
BUILDS = os.path.join(kb.DB, 'builds')
BUILD_KEYS = ('exe', 'sha256', 'size', 'pe_timestamp', 'image_base', 'distribution', 'packed')
KB_BUILD_KEYS = ('exe', 'sha256', 'size', 'pe_timestamp', 'image_base')
OVERLAY_KEYS = {'schema', 'build', 'note', 'sections', 'overrides', 'hints', 'unresolvable'}
OVERRIDE_KEYS = {'pattern', 'note', 'anchor', 'resolve', 'expect', 'section'}
HINT_KEYS = {'match', 'func_rva'}
HEX_RE = re.compile(r'^0x[0-9a-fA-F]+$')
OUT_JSON = os.path.join(ROOT, 'native', 'dist', 'swofty', 'addons', 'core', 'bindings.json')
OUT_SYMBOLS = os.path.join(os.path.dirname(OUT_JSON), 'symbols.tsv')
OUT_INC = os.path.join(ROOT, 'native', 'include', 'swofty')
OUT_RS = os.path.join(OUT_INC, 'game.rs')
C_WORDS = {'void', 'char', 'bool', 'int', 'unsigned', 'signed', 'long', 'short', 'float', 'double', 'const', 'struct',
           'size_t', 'uintptr_t', 'intptr_t'} | set(PRIM)
RS_PRIM = {'void': 'c_void', 'char': 'c_char', 'bool': 'bool', 'int': 'c_int', 'unsigned': 'c_uint', 'unsigned int': 'c_uint',
           'float': 'f32', 'double': 'f64', 'uint8_t': 'u8', 'int8_t': 'i8', 'uint16_t': 'u16', 'int16_t': 'i16',
           'uint32_t': 'u32', 'int32_t': 'i32', 'uint64_t': 'u64', 'int64_t': 'i64', 'uintptr_t': 'usize', 'size_t': 'usize',
           'intptr_t': 'isize'}


class Gate:
    dev = False

    def __init__(self):
        self.verdicts = {}
        for r in kb.read_table('verify')[0]:
            self.verdicts[(r['kind'], r['item'])] = (r['verdict'], r['notes'].split()[0])
        self.passed = set()
        tdir = os.path.join(kb.RE, 'tests')
        for f in os.listdir(tdir):
            if re.match(r'^T\d{3}\.md$', f):
                m = re.search(r'^- Verdict:\s*(\S+)', open(os.path.join(tdir, f)).read(), re.M)
                if m and m.group(1).lower().startswith('pass'):
                    self.passed.add(f[:4])

    def __call__(self, level, evidence, keys):
        if level == 'L3':
            tests = sorted({v.split('/')[0] for k, v in kb.tokens(evidence) if k == 'trace'} & self.passed)
            if tests:
                return 'stable', 'trace:' + ','.join(tests)
        if level in ('L2', 'L3'):
            for key in keys:
                v = self.verdicts.get(key)
                if v and v[0] == 'agree':
                    return 'experimental', 'verify:' + v[1]
            if self.dev:
                return 'dev', 'unverified'
            return None, f'{level} without an agree verify row'
        return None, f'{level}'


def notes_cxx(notes):
    m = re.search(r'cxx=([^;\s]+(?:\([^)]*\))?)', notes)
    return m.group(1) if m else None


def parse_proto(proto, name):
    m = re.match(rf'^(.+?)\s*\b{re.escape(name)}\s*\((.*)\)$', proto.strip())
    if not m or '(' in m.group(2):
        return None
    params = []
    raw = [x.strip() for x in m.group(2).split(',')] if m.group(2).strip() not in ('', 'void') else []
    for k, p in enumerate(raw):
        if p == '...':
            if k != len(raw) - 1 or not params:
                return None
            params.append(('...', ''))
            continue
        pm = re.match(r'^(.*?[\s*])([A-Za-z_]\w*)$', p)
        if not pm or not pm.group(1).strip():
            return None
        params.append((pm.group(1).strip(), pm.group(2)))
    return m.group(1).strip(), params


def type_names(ctype):
    return {w for w in re.findall(r'[A-Za-z_]\w*', ctype) if w not in C_WORDS}


def binding_types(b):
    if b.get('proto'):
        ret, params = parse_proto(b['proto'], b['id'])
        return set().union(type_names(ret), *(type_names(t) for t, _ in params))
    return type_names(b['type']) if b['kind'] == 'data' else set()


def c_decl(ctype, name):
    return f'{ctype}{"" if ctype.endswith("*") else " "}{name}'


RS_KEYWORDS = {'as', 'async', 'await', 'box', 'break', 'const', 'continue', 'dyn', 'else', 'enum', 'extern', 'false', 'fn', 'for', 'gen',
               'if', 'impl', 'in', 'let', 'loop', 'match', 'mod', 'move', 'mut', 'pub', 'ref', 'return', 'static', 'struct',
               'trait', 'true', 'try', 'type', 'unsafe', 'use', 'where', 'while', 'yield'}


def rs_ident(n):
    return n + '_' if n in ('self', 'Self', 'super', 'crate') else 'r#' + n if n in RS_KEYWORDS else n


def rs_type(ctype, ret=False):
    t = ctype.replace('*', ' * ').split()
    stars = t.count('*')
    const = 'const' in t[:t.index('*')] if stars else False
    base = ' '.join(w for w in t if w not in ('*', 'const', 'struct'))
    base = RS_PRIM.get(base, base)
    if not stars:
        return None if ret and base == 'c_void' else base
    out = base
    for i in range(stars):
        out = ('*const ' if const and i == 0 else '*mut ') + out
    return out


def load():
    gate = Gate()
    sigdoc = json.load(open(os.path.join(kb.DB, 'signatures.json')))
    build = sigdoc['build']
    sigs = {s['id']: s for s in sigdoc['signatures']}
    apath = os.path.join(kb.DB, 'anchors.json')
    anchors = json.load(open(apath))['anchors'] if os.path.exists(apath) else {}
    funcs, _ = kb.read_table('functions')
    globs, _ = kb.read_table('globals')
    skipped = {}

    def skip(reason):
        skipped[reason] = skipped.get(reason, 0) + 1

    bindings = []
    for table, rows in (('functions', funcs), ('globals', globs)):
        for r in rows:
            vkind = 'func' if table == 'functions' else 'global'
            status, why = gate(r['level'], r['evidence'], [(vkind, r['name']), (vkind, r['addr'])])
            if not status:
                skip(f'{table}: {why}')
                continue
            if r['sig'] == '-':
                skip(f'{table}: {status} but no signature')
                continue
            s = sigs[r['sig']]
            if not any(build['sha256'].startswith(v.get('build', '-')) for v in s.get('verified', [])):
                skip(f'{table}: signature not verified on the KB build')
                continue
            if table == 'functions':
                if r['name'].endswith('_Body'):
                    skip('functions: .data1 body')
                    continue
                kind = 'func'
            elif r['type'] == 'code':
                kind = 'call' if (s.get('expect') or '').upper().startswith('E8') else 'site'
            elif r['type'].startswith('vtable'):
                kind = 'vtable'
            else:
                kind = 'data'
            want = {'func': ('func_start', 'call'), 'site': ('mid',), 'call': ('mid',), 'vtable': ('data',), 'data': ('data',)}[kind]
            if s['kind'] not in want:
                skip(f'{table}: signature kind {s["kind"]} cannot back a {kind} binding')
                continue
            res = {k: (int(str(v), 0) if k != 'kind' else v) for k, v in s.get('resolve', {'kind': 'addr'}).items()}
            b = {'id': r['name'], 'kind': kind, 'status': status, 'level': r['level'], 'evidence': why,
                 'subsystem': r['subsystem'], 'build': build['sha256'], 'sig': s['id'], 'section': s['section'],
                 'pattern': s['pattern'], 'anchor': int(str(s['anchor']), 0), 'resolve': res, 'expect': s.get('expect'),
                 'rva': hex(int(str(s['target']), 0) - int(build['image_base'], 16)),
                 'match': hex(int(str(s['match']), 0) - int(build['image_base'], 16))}
            if kind == 'func':
                b['proto'] = r['proto'] if r['proto'] != '-' and parse_proto(r['proto'], r['name']) else None
            elif kind == 'vtable':
                m = re.match(r'^vtable\[(\d+)\]$', r['type'])
                b['slots'] = int(m.group(1)) if m else None
            elif kind == 'data':
                b['type'] = r['type']
            cxx = notes_cxx(r['notes'])
            if cxx:
                b['cxx'] = cxx
            if s['id'] in anchors:
                an = anchors[s['id']]
                b['string_anchor'] = {'string': an['string'], 'func_rva': an['func_rva'],
                                      'func_offset': an['func_offset'], 'durable': an['durable']}
            bindings.append(b)

    structs = {}
    sdir = os.path.join(kb.DB, 'structs')
    for f in sorted(os.listdir(sdir)):
        if f.endswith('.h'):
            s = parse_struct(os.path.join(sdir, f))
            fields = []
            for fl in s['fields']:
                status, why = gate(fl['level'], fl['evidence'], [('field', f'{s["name"]}.{fl["name"]}')])
                if status:
                    fields.append({**fl, 'status': status})
                else:
                    skip(f'struct fields: {why}')
            sstatus, _ = gate(s['size_level'] or 'L0', s['size_ev'], [('struct', s['name'])])
            structs[s['name']] = {'name': s['name'], 'subsystem': s['subsystem'], 'fields': fields,
                                  'size': s['size'] if sstatus else None}
    return build, sorted(bindings, key=lambda b: b['id']), structs, skipped


def is_num(v):
    return isinstance(v, int) and not isinstance(v, bool) or isinstance(v, str) and bool(HEX_RE.match(v))


def hexnum(v):
    return hex(int(str(v), 0))


def pattern_errors(text, wildcards=True):
    try:
        pat = parse_pattern(text)
    except ValueError as e:
        return [str(e)]
    if not wildcards:
        return ['has a wildcard'] if None in pat else []
    errs = []
    if len(pat) < MIN_LEN:
        errs.append(f'is shorter than {MIN_LEN} bytes')
    if pat and (pat[0] is None or pat[-1] is None):
        errs.append('starts or ends with a wildcard')
    return errs


def resolve_errors(r):
    if not isinstance(r, dict) or r.get('kind') not in ('addr', 'rel32', 'riprel'):
        return ['needs kind addr, rel32 or riprel']
    return [f'{k} is not a number' for k, v in r.items() if k != 'kind' and not is_num(v)]


def overlay_errors(name, doc, kb_build):
    if not isinstance(doc, dict):
        return ['not a JSON object']
    errs = [] if re.fullmatch(r'[a-z0-9][a-z0-9_-]*', name) else ['file name must be lowercase letters, digits, - or _']
    if doc.get('schema') != 1:
        errs.append('schema must be 1')
    errs += [f'unknown key {k!r}' for k in sorted(set(doc) - OVERLAY_KEYS)]
    b = doc.get('build')
    if not isinstance(b, dict):
        return errs + ['build block missing']
    errs += [f'build.{k} missing' for k in BUILD_KEYS if k not in b]
    errs += [f'build.{k} is not a build field' for k in sorted(set(b) - set(BUILD_KEYS))]
    if not re.fullmatch(r'[0-9a-f]{64}', str(b.get('sha256'))):
        errs.append('build.sha256 must be 64 lowercase hex digits')
    if not isinstance(b.get('packed'), bool):
        errs.append('build.packed must be true or false')
    for k in ('size', 'pe_timestamp'):
        if not isinstance(b.get(k), int) or isinstance(b.get(k), bool):
            errs.append(f'build.{k} must be an integer')
    if b.get('sha256') == kb_build['sha256']:
        errs += [f'build.{k} {b.get(k)!r} differs from signatures.json {kb_build[k]!r}' for k in KB_BUILD_KEYS
                 if b.get(k) != kb_build[k]]
        if doc.get('overrides'):
            errs.append('the KB build takes no overrides: its patterns are the signatures in signatures.json')
    tables = {}
    for k in ('sections', 'overrides', 'hints', 'unresolvable'):
        tables[k] = doc.get(k, {})
        if not isinstance(tables[k], dict):
            errs.append(f'{k} must be an object')
            tables[k] = {}
    for k, v in tables['sections'].items():
        if not (isinstance(v, str) and k.startswith('.') and v.startswith('.')):
            errs.append(f'sections.{k} must map a section name to a section name')
    for i, o in tables['overrides'].items():
        if not isinstance(o, dict):
            errs.append(f'overrides.{i} must be an object')
            continue
        errs += [f'overrides.{i}.{k} is not an override field' for k in sorted(set(o) - OVERRIDE_KEYS)]
        for k in ('pattern', 'note'):
            if not isinstance(o.get(k), str) or not o[k].strip():
                errs.append(f'overrides.{i}.{k} missing')
        if isinstance(o.get('pattern'), str) and o['pattern'].strip():
            errs += [f'overrides.{i}.pattern {e}' for e in pattern_errors(o['pattern'])]
        if 'anchor' in o and not is_num(o['anchor']):
            errs.append(f'overrides.{i}.anchor is not a number')
        if 'resolve' in o:
            errs += [f'overrides.{i}.resolve {e}' for e in resolve_errors(o['resolve'])]
        if o.get('expect') is not None:
            errs += [f'overrides.{i}.expect {e}' for e in pattern_errors(str(o['expect']), wildcards=False)]
        if 'section' in o and not (isinstance(o['section'], str) and o['section'].startswith('.')):
            errs.append(f'overrides.{i}.section is not a section name')
    for i, h in tables['hints'].items():
        if not isinstance(h, dict) or not h or set(h) - HINT_KEYS:
            errs.append(f'hints.{i} must hold match and/or func_rva')
            continue
        errs += [f'hints.{i}.{k} must be a 0x hex string' for k, v in h.items()
                 if not (isinstance(v, str) and HEX_RE.match(v))]
    for i, r in tables['unresolvable'].items():
        if not isinstance(r, str) or not r.strip():
            errs.append(f'unresolvable.{i} needs a reason')
        if i in tables['overrides'] or i in tables['hints']:
            errs.append(f'{i} is unresolvable and also has an override or a hint')
    return errs


def load_overlays(kb_build, bdir=BUILDS):
    overlays, errs = {}, []
    for f in sorted(os.listdir(bdir)) if os.path.isdir(bdir) else []:
        if f.endswith('.json'):
            with open(os.path.join(bdir, f)) as fh:
                doc = json.load(fh)
            errs += [f'builds/{f}: {e}' for e in overlay_errors(f[:-5], doc, kb_build)]
            overlays[f[:-5]] = doc
    if errs:
        raise SystemExit('bindgen: invalid build overlay\n  ' + '\n  '.join(errs))
    kb_names = [n for n, d in overlays.items() if d['build']['sha256'] == kb_build['sha256']]
    if len(kb_names) != 1:
        raise SystemExit(f'bindgen: {len(kb_names)} overlays in re/db/builds describe the KB build '
                         f'{kb_build["sha256"][:16]}, need exactly one')
    stamps = [d['build']['pe_timestamp'] for d in overlays.values()]
    if len(set(stamps)) != len(stamps):
        raise SystemExit('bindgen: two overlays in re/db/builds share a pe_timestamp, which core selects by')
    return kb_names[0], overlays


def build_set(overlay, bindings, is_kb):
    b = overlay['build']
    sections = overlay.get('sections', {})
    overrides, hints, unres = (overlay.get(k, {}) for k in ('overrides', 'hints', 'unresolvable'))
    out = []
    for base in bindings:
        i = base['id']
        ov, hint = overrides.get(i, {}), hints.get(i, {})
        if 'func_rva' in hint and 'string_anchor' not in base:
            raise SystemExit(f'bindgen: hint func_rva for {i}, which has no string anchor')
        e = {}
        for k, v in base.items():
            if k == 'build':
                v = b['sha256']
            elif k == 'section':
                v = ov.get('section', sections.get(v, v))
            elif k in ('pattern', 'expect') and k in ov:
                v = ov[k]
            elif k == 'anchor' and k in ov:
                v = int(str(ov[k]), 0)
            elif k == 'resolve' and k in ov:
                v = {rk: rv if rk == 'kind' else int(str(rv), 0) for rk, rv in ov[k].items()}
            elif k == 'rva' and not is_kb:
                continue
            elif k == 'match':
                if 'match' in hint:
                    v = hexnum(hint['match'])
                elif not is_kb:
                    continue
            elif k == 'string_anchor':
                v = dict(v)
                if 'func_rva' in hint:
                    v['func_rva'] = hexnum(hint['func_rva'])
                elif not is_kb:
                    v.pop('func_rva', None)
            e[k] = v
        if i in unres:
            e['status'] = 'unresolvable'
            e['reason'] = unres[i]
        out.append(e)
    doc = {'schema': 1, 'generator': 're/tools/bindgen.py', 'build': {k: b[k] for k in BUILD_KEYS}}
    if sections:
        # core's read_section_map sends core and addon scans of a KB section name to this build's section
        doc['sections'] = dict(sections)
    doc['bindings'] = out
    return doc


def unknown_ids(overlay, bindings):
    ids = {b['id'] for b in bindings}
    named = set().union(*(overlay.get(k, {}) for k in ('overrides', 'hints', 'unresolvable')))
    return sorted(named - ids)


def set_path(json_path, name, is_kb):
    return json_path if is_kb else os.path.join(os.path.dirname(json_path), f'bindings.{name}.json')


def field_size(fl, structs):
    if fl['ptr']:
        return 8, 8
    if fl['type'] in PRIM:
        return PRIM[fl['type']], max(PRIM[fl['type']], 1)
    st = structs.get(fl['type'])
    if st and st['fields']:
        end, align = layout_end(st, structs)
        return end, align
    return None, None


def layout_end(st, structs):
    end, align = 0, 1
    for fl in st['fields']:
        sz, al = field_size(fl, structs)
        end = fl['off'] + sz * fl['count']
        align = max(align, al)
    end = max(end, st['size'] or 0)
    return end, align


def layout(st, structs):
    out, pos = [], 0
    for fl in st['fields']:
        sz, al = field_size(fl, structs)
        if sz is None or fl['off'] < pos or fl['off'] % al:
            continue
        if fl['off'] > pos:
            out.append(('pad', pos, fl['off'] - pos))
        out.append(('field', fl['off'], fl))
        pos = fl['off'] + sz * fl['count']
    if st['size'] and st['size'] > pos:
        out.append(('pad', pos, st['size'] - pos))
    return out


def deps_by_value(st, structs):
    return {fl['type'] for fl in st['fields'] if not fl['ptr'] and fl['type'] in structs}


def struct_order(names, structs):
    done, order = set(), []

    def visit(n, stack):
        if n in done:
            return
        if n in stack:
            raise SystemExit(f'bindgen: by-value struct cycle through {n}')
        for d in sorted(deps_by_value(structs[n], structs)):
            visit(d, stack | {n})
        done.add(n)
        order.append(n)
    for n in sorted(names):
        visit(n, set())
    return order


def cpp_field(fl):
    return c_decl(fl['type'] + (' ' + '*' * fl['ptr'] if fl['ptr'] else ''), fl['name']) + (f'[{fl["count"]}]' if fl['array'] else '')


def tag(status, level, cxx=None):
    return f' // {level} {status}' + (f', {cxx}' if cxx else '')


def gen_cpp(build, bindings, structs):
    used = {s['name'] for s in structs.values() if s['fields']}
    referenced = set(structs).union(*(binding_types(b) for b in bindings))
    subs = sorted({b['subsystem'] for b in bindings} | {structs[n]['subsystem'] for n in used})
    files = {}
    head = '#pragma once\n'
    base = [head, '#include <cstddef>\n#include <cstdint>\n#include <swofty/mixin.hpp>\n\nnamespace swofty::game {\n\n',
            f'inline constexpr const char build_sha256[] = "{build["sha256"]}";\n\n']
    base += [f'struct {n};\n' for n in sorted(referenced)]
    base.append('\n}\n')
    files['game/base.hpp'] = ''.join(base)
    for sub in subs:
        names = [n for n in struct_order(used, structs) if structs[n]['subsystem'] == sub]
        incs = sorted({structs[d]['subsystem'] for n in names for d in deps_by_value(structs[n], structs)} - {sub})
        out = [head, '#include <swofty/game/base.hpp>\n'] + [f'#include <swofty/game/{i}.hpp>\n' for i in incs]
        out.append('\nnamespace swofty::game {\n')
        for n in names:
            st = structs[n]
            out.append(f'\nstruct {n} {{\n')
            asserts = []
            for k, off, x in layout(st, structs):
                if k == 'pad':
                    out.append(f'    uint8_t _pad_{off:#x}[{x:#x}];\n')
                else:
                    out.append(f'    {cpp_field(x)};{tag(x["status"], x["level"])}\n')
                    asserts.append(f'static_assert(offsetof({n}, {x["name"]}) == {off:#x});\n')
            out.append('};\n')
            out += asserts
            if st['size']:
                out.append(f'static_assert(sizeof({n}) == {st["size"]:#x});\n')
        mine = [b for b in bindings if b['subsystem'] == sub]
        if mine:
            out.append('\n')
        for b in mine:
            t = tag(b['status'], b['level'], b.get('cxx'))
            i = b['id']
            if b['kind'] == 'func' and b.get('proto'):
                ret, params = parse_proto(b['proto'], i)
                args = ', '.join('...' if pt == '...' else c_decl(pt, pn) for pt, pn in params)
                out.append(f'using {i}_t = {ret}({args});\n')
                out.append(f'inline constexpr fn<"{i}", {i}_t> {i}{{}};{t}\n')
            elif b['kind'] == 'func':
                out.append(f'inline constexpr address<"{i}"> {i}{{}};{t}\n')
            elif b['kind'] == 'data':
                out.append(f'inline constexpr data<"{i}", {b["type"]}> {i}{{}};{t}\n')
            elif b['kind'] == 'vtable':
                out.append(f'inline constexpr vtable<"{i}", {b["slots"] or 0}> {i}{{}};{t}\n')
            else:
                out.append(f'inline constexpr {b["kind"]}<"{i}"> {i}{{}};{t}\n')
        out.append('\n}\n')
        files[f'game/{sub}.hpp'] = ''.join(out)
    files['game.hpp'] = head + ''.join(f'#include <swofty/game/{s}.hpp>\n' for s in subs)
    return files


def gen_rust(build, bindings, structs):
    used = struct_order({s['name'] for s in structs.values() if s['fields']}, structs)
    referenced = set(structs).union(*(binding_types(b) for b in bindings))
    out = ['#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code, unused_imports)]\n',
           'use core::ffi::{c_char, c_int, c_uint, c_void};\n\n',
           f'pub const BUILD_SHA256: &str = "{build["sha256"]}";\n\n',
           '#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum Status {\n    Stable,\n    Experimental,\n}\n\n',
           '#[derive(Clone, Copy, Debug)]\npub struct Binding {\n    pub id: &\'static str,\n    pub kind: &\'static str,\n    pub status: Status,\n}\n']
    for n in sorted(referenced - set(used)):
        out.append(f'\n#[repr(C)]\npub struct {n} {{\n    _opaque: [u8; 0],\n}}\n')
    for n in used:
        st = structs[n]
        body, asserts = [], []
        for k, off, x in layout(st, structs):
            if k == 'pad':
                body.append(f'    _pad_{off:#x}: [u8; {x:#x}],\n')
            else:
                t = rs_type(x['type'] + ' ' + '*' * x['ptr'])
                if x['array']:
                    t = f'[{t}; {x["count"]}]'
                body.append(f'    pub {rs_ident(x["name"])}: {t},\n')
                asserts.append(f'const _: () = assert!(core::mem::offset_of!({n}, {rs_ident(x["name"])}) == {off:#x});\n')
        out.append(f'\n#[repr(C)]\npub struct {n} {{\n' + ''.join(body) + '}\n')
        out += asserts
        if st['size']:
            out.append(f'const _: () = assert!(core::mem::size_of::<{n}>() == {st["size"]:#x});\n')
    out.append('\n')
    for b in bindings:
        if b['kind'] == 'func' and b.get('proto'):
            ret, params = parse_proto(b['proto'], b['id'])
            r = rs_type(ret, ret=True)
            args = ', '.join('...' if pt == '...' else f'{rs_ident(pn)}: {rs_type(pt)}' for pt, pn in params)
            out.append(f'pub type {b["id"]}_t = unsafe extern "C" fn({args})' + (f' -> {r}' if r else '') + ';\n')
    out.append('\npub mod bindings {\n    use super::{Binding, Status};\n')
    for b in bindings:
        st = 'Stable' if b['status'] == 'stable' else 'Experimental'
        out.append(f'    pub const {b["id"]}: Binding = Binding {{ id: "{b["id"]}", kind: "{b["kind"]}", status: Status::{st} }};\n')
    out.append('}\n')
    return ''.join(out)


def rustfmt(text):
    """The committed game.rs is rustfmt output under the repo's rustfmt.toml; without rustfmt the text stays as is."""
    try:
        r = subprocess.run(['rustfmt', '--edition', '2024', '--config-path', os.path.join(ROOT, 'rustfmt.toml')],
                           input=text, capture_output=True, text=True)
    except OSError:
        return text, False
    return (r.stdout, True) if r.returncode == 0 else (text, False)


def gen_symbols(build):
    base = int(build['image_base'], 16)
    rows = sorted((int(r['addr'], 16) - base, r['name'], r['level']) for r in kb.read_table('functions')[0])
    return f'# {build["exe"]} {build["sha256"]} {build["pe_timestamp"]}\n' + ''.join(f'{a:x}\t{n}\t{l}\n' for a, n, l in rows)


def run_sigcheck(build, bindings, exe):
    if not exe:
        return bindings, 'sigcheck skipped (--no-exe)'
    if not os.path.exists(exe):
        return bindings, f'sigcheck skipped (no {os.path.relpath(exe, ROOT)})'
    try:
        import sigcheck
        img = sigcheck.Image(exe)
    except ImportError as e:
        return bindings, f'sigcheck skipped ({e})'
    if sigcheck.sha256_file(exe) != build['sha256']:
        raise SystemExit(f'bindgen: {exe} is not the KB build')
    sigs = {s['id']: s for s in json.load(open(os.path.join(kb.DB, 'signatures.json')))['signatures']}
    names = sigcheck.load_kb_names()
    ok = []
    for b in bindings:
        errs, _ = sigcheck.check(img, sigs[b['sig']], build['sha256'], names, True)
        if errs:
            print(f'bindgen: {b["id"]} dropped, sigcheck: {"; ".join(errs)}', file=sys.stderr)
        else:
            ok.append(b)
    return ok, f'sigcheck passed for {len(ok)}/{len(bindings)}'


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--check', action='store_true')
    ap.add_argument('--no-exe', action='store_true')
    ap.add_argument('--exe', default=os.path.join(ROOT, 'work', 'exe', 'F1_25.exe'))
    ap.add_argument('--json', default=OUT_JSON)
    ap.add_argument('--include', default=OUT_INC)
    ap.add_argument('--rust', default=OUT_RS)
    ap.add_argument('--symbols', default=OUT_SYMBOLS)
    ap.add_argument('--dev', action='store_true',
                    help='also export L2 rows that have no agree verify row (status dev); needs --json, --include, --rust '
                         'and --symbols outside the shipped paths')
    a = ap.parse_args()
    if a.dev:
        shipped = {os.path.normpath(p) for p in (OUT_JSON, OUT_INC, OUT_RS, OUT_SYMBOLS)}
        if {os.path.normpath(p) for p in (a.json, a.include, a.rust, a.symbols)} & shipped:
            raise SystemExit('--dev writes only to scratch paths: pass --json, --include, --rust and --symbols')
        Gate.dev = True
    build, bindings, structs, skipped = load()
    kb_name, overlays = load_overlays(build)
    bindings, sc = run_sigcheck(build, bindings, None if a.no_exe else a.exe)
    rust, formatted = rustfmt(gen_rust(build, bindings, structs))
    if not formatted:
        print('bindgen: rustfmt unavailable or failed; game.rs is written unformatted', file=sys.stderr)
    outputs = {a.rust: rust, a.symbols: gen_symbols(build)}
    built = {a.symbols}
    sets = []
    for name, overlay in overlays.items():
        doc = build_set(overlay, bindings, name == kb_name)
        path = set_path(a.json, name, name == kb_name)
        outputs[path] = json.dumps(doc, indent=1) + '\n'
        built.add(path)
        sets.append((path, overlay, doc, unknown_ids(overlay, bindings)))
    outputs |= {os.path.join(a.include, k): v for k, v in gen_cpp(build, bindings, structs).items()}
    stale = []
    for p, text in outputs.items():
        if a.check and p in built and not os.path.isdir(os.path.dirname(p)):
            continue
        old = open(p).read() if os.path.exists(p) else None
        if old != text:
            stale.append(p)
            if not a.check:
                os.makedirs(os.path.dirname(p), exist_ok=True)
                open(p, 'w').write(text)
    gdir = os.path.join(a.include, 'game')
    for f in sorted(os.listdir(gdir)) if os.path.isdir(gdir) else []:
        p = os.path.join(gdir, f)
        if f.endswith('.hpp') and os.path.normpath(p) not in {os.path.normpath(k) for k in outputs}:
            stale.append(p)
            if not a.check:
                os.remove(p)
    n = {s: sum(b['status'] == s for b in bindings) for s in ('stable', 'experimental')}
    print(f'bindgen: {len(bindings)} bindings ({n["stable"]} stable, {n["experimental"]} experimental), '
          f'{sum(bool(s["fields"]) for s in structs.values())} structs with exported fields; {sc}')
    for why, c in sorted(skipped.items()):
        print(f'  not exported: {c:4d}  {why}')
    for path, overlay, doc, unknown in sets:
        b = doc['build']
        print(f'bindgen: {os.path.basename(path)}: {b["distribution"]} {b["sha256"][:16]}'
              f'{" packed" if b["packed"] else ""}, {len(overlay.get("overrides", {}))} override(s), '
              f'{len(overlay.get("hints", {}))} hint(s), '
              f'{sum(e["status"] == "unresolvable" for e in doc["bindings"])} unresolvable')
        if unknown:
            print(f'  ignored: {len(unknown)} overlay entr{"y" if len(unknown) == 1 else "ies"} for bindings that are '
                  f'not exported: {", ".join(unknown)}')
    if a.check:
        for p in stale:
            print(f'stale: {os.path.relpath(p, ROOT)}')
        return 1 if stale else 0
    print(f'bindgen: {len(stale)} file(s) written')
    return 0


if __name__ == '__main__':
    sys.exit(main())
