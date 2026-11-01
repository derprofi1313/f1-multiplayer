import numpy as np, pefile, struct, sys, os, bisect
EXE = os.environ.get('EXE', os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), 'work', 'exe', 'F1_25.exe'))
BASE = 0x140000000
_pe = pefile.PE(EXE, fast_load=True)
SECS = [(s.Name.rstrip(b'\0').decode(), s.VirtualAddress, max(s.Misc_VirtualSize, s.SizeOfRawData), s.PointerToRawData, s.SizeOfRawData) for s in _pe.sections]
DATA = np.memmap(EXE, dtype=np.uint8, mode='r')
def off2rva(o):
    for n, va, vs, ro, rs in SECS:
        if ro <= o < ro + rs: return o - ro + va
def rva2off(r):
    for n, va, vs, ro, rs in SECS:
        if va <= r < va + rs: return r - va + ro
def sec_of(r):
    for n, va, vs, ro, rs in SECS:
        if va <= r < va + vs: return n
def code_bytes(name='.xtls'):
    for n, va, vs, ro, rs in SECS:
        if n == name: return va, DATA[ro:ro+rs]
_pdata = None
def pdata():
    global _pdata
    if _pdata is None:
        for n, va, vs, ro, rs in SECS:
            if n == '.srdata':
                arr = np.frombuffer(DATA[ro:ro+0x17c1f0].tobytes(), dtype=np.uint32).reshape(-1, 3)
                arr = arr[arr[:,0] != 0]
                _pdata = arr[np.argsort(arr[:,0])]
    return _pdata
def func_of(rva):
    p = pdata()
    i = np.searchsorted(p[:,0], rva, side='right') - 1
    if i >= 0 and p[i,0] <= rva < p[i,1]: return int(p[i,0]), int(p[i,1])
    return None
def riprel_refs(targets, secname='.xtls', chunk=1<<24):
    va, b = code_bytes(secname)
    targets = np.array(sorted(set(targets)), dtype=np.int64)
    out = []
    n = len(b)
    for st in range(0, n - 4, chunk):
        en = min(n - 4, st + chunk)
        seg = np.asarray(b[st:en+4]).astype(np.int64)
        d = seg[0:en-st] | (seg[1:en-st+1] << 8) | (seg[2:en-st+2] << 16) | (seg[3:en-st+3] << 24)
        d = np.where(d >= 1 << 31, d - (1 << 32), d)
        pos = np.arange(st, en, dtype=np.int64)
        tgt = va + pos + 4 + d
        m = np.isin(tgt, targets)
        for p_, t_ in zip(pos[m], tgt[m]):
            out.append((int(va + p_), int(t_)))
    return out
def abs_refs(target_rvas):
    import numpy as np
    out = []
    tv = set(BASE + t for t in target_rvas)
    for n, va, vs, ro, rs in SECS:
        if rs < 8: continue
        b = np.frombuffer(bytes(DATA[ro:ro+rs - (rs % 8)]), dtype=np.uint64)
        for align in range(0, 8, 8):
            m = np.isin(b, np.array(sorted(tv), dtype=np.uint64))
            for i in np.nonzero(m)[0]:
                out.append((n, va + int(i)*8, int(b[i]) - BASE))
    return out
