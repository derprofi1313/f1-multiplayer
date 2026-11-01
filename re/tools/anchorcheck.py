#!/usr/bin/env python3
"""Offline proof that the string-anchor resolver computes the same address the baked signature does.

Runs the full runtime algorithm (anchorlib.resolve_via_anchor: locate the string in the image, find its
RIP-relative xref in the code section, map it to the function base via the exception directory, apply the
binding's offset and resolve) against the Steam image for every anchor in re/db/anchors.json, and asserts
the result equals the signature's target in re/db/signatures.json. The game cannot be run here, so this is
how correctness is established statically. Exit 1 on any mismatch.

  anchorcheck.py [--exe path/to/F1_25.exe]
"""
import argparse, json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
RE = os.path.dirname(HERE)
DB = os.path.join(RE, 'db')


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--exe')
    a = ap.parse_args()
    if a.exe:
        os.environ['EXE'] = a.exe
    sys.path.insert(0, HERE)
    import anchorlib as A

    sigdoc = json.load(open(os.path.join(DB, 'signatures.json')))
    sigs = {s['id']: s for s in sigdoc['signatures']}
    anchors = json.load(open(os.path.join(DB, 'anchors.json')))
    ib = A.IMAGE_BASE
    kb_ib = int(sigdoc['build']['image_base'], 16)

    if A._pe.FILE_HEADER.TimeDateStamp != sigdoc['build']['pe_timestamp']:
        print('anchorcheck: the exe is not the KB build; cross-check needs the KB Steam build', file=sys.stderr)
        return 2

    ok = bad = 0
    for sid, an in sorted(anchors['anchors'].items()):
        s = sigs[sid]
        target = int(str(s['target']), 0)
        resolve = {k: v for k, v in s.get('resolve', {'kind': 'addr'}).items()}
        got, func_rva = A.resolve_via_anchor(an['string'], an['func_offset'], s['anchor'], resolve)
        want_func = int(str(an['func_rva']), 0)
        tag = 'durable ' if an['durable'] else 'offset  '
        if got == target and func_rva == want_func:
            ok += 1
            print(f'  ok   {tag} {sid:34s} string -> func base+{func_rva:#010x} -> target {got - ib:#010x} '
                  f'(== signature)')
        else:
            bad += 1
            g = 'None' if got is None else f'{got - ib:#x}'
            print(f'  FAIL {tag} {sid:34s} resolved func {func_rva} target {g}, signature target '
                  f'{(target - kb_ib):#x}')
    print(f'anchorcheck: {ok} ok, {bad} mismatch, over {len(anchors["anchors"])} anchored signature(s)')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
