#!/usr/bin/env python3
"""Generate re/db/anchors.json: a string anchor for every signature whose function carries a unique,
build-stable string, so the binding resolves on any build without a per-build byte pattern.

  anchormk.py            write re/db/anchors.json
  anchormk.py --check    exit 1 if re/db/anchors.json is stale

For each signature in re/db/signatures.json this finds the function that contains the signature `match`
(via the exception directory), picks the longest C string referenced only from inside that function and
byte-unique in read-only data, and records string + func_rva + func_offset. It then runs the full runtime
resolver (anchorlib.resolve_via_anchor) and keeps the anchor only when it reproduces the signature target.

durable: the binding resolves to the function base itself (func_offset 0, resolve addr) and so is fully
build-agnostic. Otherwise the function is located by string but a fixed byte offset is applied on top, which
holds on the KB build and on patches that do not move the offset, but not across a repack.
"""
import argparse, json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import anchorlib as A

RE = os.path.dirname(HERE)
DB = os.path.join(RE, 'db')
OUT = os.path.join(DB, 'anchors.json')


def build():
    sigdoc = json.load(open(os.path.join(DB, 'signatures.json')))
    build = sigdoc['build']
    ib = int(build['image_base'], 16)
    anchors, skipped = {}, {}

    def skip(sid, why):
        skipped[sid] = why

    fbases = {A.func_base_of(int(str(s['match']), 0) - ib) for s in sigdoc['signatures']}
    fbases.discard(None)
    locs, funcmap = A.anchor_xref_map(sorted(fbases))

    for s in sigdoc['signatures']:
        sid = s['id']
        match = int(str(s['match']), 0)
        target = int(str(s['target']), 0)
        resolve = {k: v for k, v in s.get('resolve', {'kind': 'addr'}).items()}
        fbase = A.func_base_of(match - ib)
        if fbase is None:
            skip(sid, 'match is not inside a function in the exception directory')
            continue
        cs = A.best_anchor_string(fbase, locs, funcmap)
        if cs is None:
            skip(sid, 'function has no unique build-stable string')
            continue
        foff = (match - ib) - fbase
        got, func_rva = A.resolve_via_anchor(cs.decode('latin1'), hex(foff), s['anchor'], resolve)
        if got != target:
            skip(sid, f'anchor resolved to {got if got is None else hex(got)}, not the signature target '
                      f'{hex(target)}')
            continue
        durable = foff == 0 and resolve.get('kind', 'addr') == 'addr' and int(str(s['anchor']), 0) == 0
        anchors[sid] = {'string': cs.decode('latin1'), 'func_rva': hex(fbase), 'func_offset': hex(foff),
                        'durable': durable}
    doc = {'schema': 1, 'generator': 're/tools/anchormk.py',
           'build': {'sha256': build['sha256'], 'image_base': build['image_base'],
                     'pe_timestamp': build['pe_timestamp']},
           'anchors': dict(sorted(anchors.items()))}
    return doc, skipped


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--check', action='store_true')
    ap.add_argument('--out', default=OUT)
    a = ap.parse_args()
    doc, skipped = build()
    text = json.dumps(doc, indent=1) + '\n'
    old = open(a.out).read() if os.path.exists(a.out) else None
    dur = sum(1 for v in doc['anchors'].values() if v['durable'])
    print(f'anchormk: {len(doc["anchors"])} anchor(s) ({dur} durable, {len(doc["anchors"]) - dur} '
          f'offset-based); {len(skipped)} signature(s) keep the pattern')
    for sid, why in sorted(skipped.items()):
        print(f'  pattern-only: {sid}: {why}')
    if a.check:
        if old != text:
            print(f'stale: {os.path.relpath(a.out, RE)}')
            return 1
        return 0
    if old != text:
        open(a.out, 'w').write(text)
        print(f'anchormk: wrote {os.path.relpath(a.out, RE)}')
    else:
        print('anchormk: up to date')
    return 0


if __name__ == '__main__':
    sys.exit(main())
