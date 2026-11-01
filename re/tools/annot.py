import sys, re, os
sys.path.insert(0, os.path.dirname(__file__))
from pe_index import *
def cstr(rva, maxn=200):
    o = rva2off(rva)
    if o is None: return None
    b = bytes(DATA[o:o+maxn])
    z = b.find(b'\0')
    s = b[:z] if z >= 0 else b
    if len(s) >= 2 and all(32 <= c < 127 or c in (9,10,13) for c in s): return s.decode()
    w = bytes(DATA[o:o+maxn*2])
    try:
        z = w.find(b'\0\0\0')
        ws = w[:z+1 if z%2 else z].decode('utf-16le')
        if len(ws) >= 3 and all(32 <= ord(c) < 127 for c in ws): return 'L' + ws
    except Exception: pass
    return None
pat = re.compile(r'\b(?:UNK|DAT|PTR_DAT|s_[A-Za-z0-9_]*?)_?(14[0-9a-f]{7})\b')
for fn in sys.argv[1:]:
    src = open(fn).read()
    def rep(m):
        a = int(m.group(1), 16) - BASE
        s = cstr(a)
        return m.group(0) + (f'/*"{s[:80]}"*/' if s else '')
    out = pat.sub(rep, src)
    open(fn, 'w').write(out)
