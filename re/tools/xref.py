import sys, os
sys.path.insert(0, __import__('os').path.dirname(__import__('os').path.abspath(__file__)))
from pe_index import *
from collections import defaultdict
# args: file offsets (hex) or name=offset
tg = {}
for a in sys.argv[1:]:
    if '=' in a: n, o = a.split('='); 
    else: n, o = a, a
    tg[off2rva(int(o, 16))] = n
refs = riprel_refs(list(tg))
by = defaultdict(list)
for site, t in refs:
    by[t].append(site)
for t, n in tg.items():
    sites = by.get(t, [])
    fs = sorted(set((func_of(s) or (0,0))[0] for s in sites))
    print(f'{n:40s} rva={t:08x} refs={len(sites)} sites={[hex(BASE+s) for s in sites[:8]]} funcs={[hex(BASE+f) for f in fs[:8]]}')
