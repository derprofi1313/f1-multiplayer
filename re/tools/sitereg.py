#!/usr/bin/env python3
"""Register a hook site, data table or function together with its signature (PROTOCOL.md §4, §7).

  sitereg.py --agent A site NAME VA [--expect-len N] [--func-start VA] [--data1-reason TEXT]
  sitereg.py --agent A data NAME VA --type CTYPE --via INSN_VA
  sitereg.py --agent A func NAME VA [--proto PROTO]
  common: --subsystem ID --level L2 --evidence "decomp:...;xref:..." --notes "..." --used-by FILE [--match-from VA] [--dry-run]

It makes a unique signature with sigcheck's maker (a `site` gets kind mid, `data` a riprel signature through the
instruction at --via, `func` kind func_start), checks it with sigcheck, writes it into signatures.json under the KB
lock, then adds or updates the functions/globals row through kb.py with the signature id in its sig column. An
existing row at the same address keeps its name unless NAME matches; an existing signature on the same target is
reused. A .data1 site needs --data1-reason and, without a .pdata function, --func-start for the boundary check.
"""
import argparse, datetime, json, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kb
import sigcheck

SIGS = os.path.join(kb.DB, 'signatures.json')


def snake(name):
    core = re.sub(r'^(site|k|g|vt)_[A-Z][A-Za-z0-9]*_', '', name)
    core = re.sub(r'^[A-Z][A-Za-z0-9]*_', '', core) if core == name else core
    s = re.sub(r'(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])', '_', core).replace('__', '_')
    return re.sub(r'[^a-z0-9_]', '', s.lower()).strip('_')


def data_candidate(img, target, via):
    from capstone import Cs, CS_ARCH_X86, CS_MODE_64
    md = Cs(CS_ARCH_X86, CS_MODE_64)
    md.detail = True
    i = next(md.disasm(img.read(via, 16), via, count=1), None)
    if i is None or i.disp_size != 4:
        raise SystemExit(f'--via 0x{via:x} is not an instruction with a disp32')
    disp = int.from_bytes(img.read(via + i.disp_offset, 4), 'little', signed=True)
    if via + i.size + disp != target:
        raise SystemExit(f'--via 0x{via:x} ({i.mnemonic} {i.op_str}) addresses 0x{via + i.size + disp:x}, not 0x{target:x}')
    cand = sigcheck.make(img, via, 'mid')
    if not cand:
        raise SystemExit(f'no unique pattern around 0x{via:x}')
    match = int(cand['match'], 16)
    cand.update({'anchor': hex(via - match + i.disp_offset), 'resolve': {'kind': 'riprel', 'insn': hex(via - match), 'len': i.size},
                 'target': hex(target), 'kind': 'data'})
    cand.pop('expect', None)
    return cand


def anchored_candidate(img, va, start, kind):
    sec = img.sec_of(va)
    for n in range(sigcheck.MIN_LEN, 257, 4):
        pat = sigcheck.pattern_for(img, start, max(n, va - start + 6))
        if len(pat) < max(sigcheck.MIN_LEN, va - start + 1):
            continue
        if len(sigcheck.find_all(img, sec, pat, limit=2)) == 1:
            return {'id': '<subsystem>.<name>', 'section': sec,
                    'pattern': ' '.join('??' if x is None else f'{x:02X}' for x in pat), 'match': hex(start),
                    'anchor': hex(va - start), 'resolve': {'kind': 'addr'}, 'target': hex(va), 'kind': kind,
                    'names': [], 'verified': [], 'used_by': []}
    raise SystemExit(f'no unique pattern from 0x{start:x} covering 0x{va:x} within 256 bytes')


def expect_bytes(img, va, n):
    if n is None:
        from capstone import Cs, CS_ARCH_X86, CS_MODE_64
        md = Cs(CS_ARCH_X86, CS_MODE_64)
        n, pos = 0, va
        for i in md.disasm(img.read(va, 32), va):
            n += i.size
            if n >= 5:
                break
    return ' '.join(f'{x:02X}' for x in img.read(va, n))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--agent', required=True)
    ap.add_argument('what', choices=('site', 'data', 'func'))
    ap.add_argument('name')
    ap.add_argument('va')
    ap.add_argument('--subsystem', default='career')
    ap.add_argument('--level', default='L2')
    ap.add_argument('--evidence', required=True)
    ap.add_argument('--notes', default='-')
    ap.add_argument('--used-by', action='append', default=[])
    ap.add_argument('--type', default=None)
    ap.add_argument('--via', default=None)
    ap.add_argument('--proto', default='-')
    ap.add_argument('--expect-len', type=int, default=None)
    ap.add_argument('--func-start', default=None)
    ap.add_argument('--data1-reason', default=None)
    ap.add_argument('--id', default=None)
    ap.add_argument('--match-from', default=None, help='start the pattern at this instruction (for code cloned around the target)')
    ap.add_argument('--dry-run', action='store_true')
    ap.add_argument('--force', action='store_true')
    a = ap.parse_args()

    va = int(a.va, 16)
    fs = int(a.func_start, 16) if a.func_start else None
    img = sigcheck.Image(sigcheck.DEFAULT_EXE)
    table = 'functions' if a.what == 'func' else 'globals'
    rows, _ = kb.read_table(table)
    existing = next((r for r in rows if int(r['addr'], 16) == va), None)
    name = a.name
    if existing and existing['name'] != a.name:
        print(f'note: 0x{va:x} is already {existing["name"]} in {table}.tsv; keeping that name')
        name = existing['name']

    doc = json.load(open(SIGS))
    sid = a.id or f'{a.subsystem}.{snake(name)}'
    reuse = next((s for s in doc['signatures'] if int(str(s['target']), 0) == va and
                  s['kind'] == {'site': 'mid', 'data': 'data', 'func': 'func_start'}[a.what]), None)
    if reuse:
        sid = reuse['id']
        sig = reuse
        print(f'reusing signature {sid}')
    else:
        if any(s['id'] == sid for s in doc['signatures']):
            raise SystemExit(f'signature id {sid} already used for another target; pass --id')
        if a.what == 'data':
            if not a.via or not a.type:
                raise SystemExit('data needs --type and --via')
            sig = data_candidate(img, va, int(a.via, 16))
        else:
            kind = 'mid' if a.what == 'site' else 'func_start'
            sig = anchored_candidate(img, va, int(a.match_from, 16), kind) if a.match_from else sigcheck.make(img, va, kind, fs)
            if not sig:
                raise SystemExit(f'no unique pattern within 96 bytes of 0x{va:x}')
            if a.what == 'site':
                sig['expect'] = expect_bytes(img, va, a.expect_len)
            else:
                sig.pop('expect', None)
        sig['id'] = sid
        if fs is not None:
            sig['func_start'] = hex(fs)
        if a.data1_reason:
            sig['data1_reason'] = a.data1_reason
        sig['verified'] = [{'build': doc['build']['sha256'][:16], 'date': datetime.date.today().isoformat(), 'by': a.agent,
                            'how': 'static'}]
        sig = {k: v for k, v in sig.items() if v is not None}
    sig.setdefault('names', [])
    if name not in sig['names']:
        sig['names'].append(name)
    sig.setdefault('used_by', [])
    for u in a.used_by:
        if u not in sig['used_by']:
            sig['used_by'].append(u)

    names = sigcheck.load_kb_names()
    names[name] = (va, sid)
    errs, info = sigcheck.check(img, sig, doc['build']['sha256'], names, True)
    if errs:
        raise SystemExit(f'signature {sid} fails sigcheck:\n  ' + '\n  '.join(errs) + f'\n{json.dumps(sig, indent=1)}')
    if a.dry_run:
        print(json.dumps(sig, indent=1))
        return 0

    with kb.locked():
        doc = json.load(open(SIGS))
        doc['signatures'] = [s for s in doc['signatures'] if s['id'] != sid] + [sig]
        tmp = f'{SIGS}.{os.getpid()}.tmp'
        with open(tmp, 'w') as f:
            json.dump(doc, f, indent=1)
            f.write('\n')
        os.replace(tmp, SIGS)
        kb.journal(a.agent, 'sig', 'signatures', sid, None, sig)

    base = [sys.executable, os.path.join(HERE, 'kb.py'), '--agent', a.agent] + (['--force'] if a.force else [])
    if existing:
        cmd = base + ['set', table, existing['addr'], f'sig={sid}', f'level={a.level}', f'evidence+={a.evidence}']
        if a.notes != '-':
            cmd.append(f'notes+={a.notes}')
        if a.what == 'func' and a.proto != '-':
            cmd.append(f'proto={a.proto}')
        if a.what == 'data' and a.type:
            cmd.append(f'type={a.type}')
    else:
        cols = [f'addr=0x{va:x}', f'name={name}', f'sig={sid}', f'level={a.level}', f'subsystem={a.subsystem}',
                f'evidence={a.evidence}', f'notes={a.notes}']
        if a.what == 'func':
            cols.append(f'proto={a.proto}')
        else:
            cols.append(f'type={"code" if a.what == "site" else a.type}')
        cmd = base + ['add', table] + cols
    r = subprocess.run(cmd)
    if r.returncode:
        raise SystemExit(f'kb.py refused the {table} row; signature {sid} is written, fix the row and rerun')
    print(f'{name} 0x{va:x} -> {sid} ({info.get("hits")} hit, match {hex(info["match"])})')
    return 0


if __name__ == '__main__':
    sys.exit(main())
