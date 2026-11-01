"""Static mirror of the core's runtime string-anchor resolver (native/src/core/manager.cpp).

A binding is normally resolved by scanning for a byte pattern baked from one build. That fails on
any other build. The string-anchor path instead, for each binding:

  1. locates the anchor string's address in the image (scan the read-only data sections for the bytes),
  2. finds the instruction in the code section that RIP-relative references that address (the rel32 that
     points at the string VA),
  3. maps that xref back to the enclosing function via the exception directory (RUNTIME_FUNCTION table) to
     get the function's base address,
  4. records func_base + func_offset as the signature `match`, then applies the binding's own
     anchor/resolve/offset adjustment, exactly as the pattern path does.

The strings are stable across builds, so this resolves on builds the baked pattern cannot (the packed EA
App build, future patches). This module is the single source of truth for both anchormk.py (generate the
shipped anchors) and anchorcheck.py (prove, against the Steam image, that the anchor resolves to the same
address the signature does).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from pe_index import DATA, SECS, rva2off, off2rva, sec_of, BASE
import pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_64
from capstone.x86 import X86_OP_MEM, X86_REG_RIP

_pe = pefile.PE(os.environ.get('EXE', os.path.join(
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), 'work', 'exe', 'F1_25.exe')),
    fast_load=True)
IMAGE_BASE = _pe.OPTIONAL_HEADER.ImageBase

_CH = {}
for _s in _pe.sections:
    _CH[_s.Name.rstrip(b'\0').decode('latin1')] = (_s.VirtualAddress,
                                                   max(_s.Misc_VirtualSize, _s.SizeOfRawData),
                                                   _s.Characteristics, _s.PointerToRawData, _s.SizeOfRawData)
MEM_EXECUTE, MEM_READ, MEM_WRITE = 0x20000000, 0x40000000, 0x80000000


def _is(nm, flag):
    return bool(_CH.get(nm, (0, 0, 0, 0, 0))[2] & flag)


def code_sections():
    """Executable code sections (EXECUTE and READ, not WRITE): Steam .xtls, EA .rdata. The first such
    section is the primary code section; the packed RWX blobs (Steam .data1, EA .xcode) are excluded."""
    out = []
    for n, va, vs, ro, rs in SECS:
        if _is(n, MEM_EXECUTE) and _is(n, MEM_READ) and not _is(n, MEM_WRITE):
            out.append((n, va, vs, ro, rs))
    return out


def data_sections():
    """Readable, non-executable sections that hold the anchor strings."""
    out = []
    for n, va, vs, ro, rs in SECS:
        if _is(n, MEM_READ) and not _is(n, MEM_EXECUTE):
            out.append((n, va, vs, ro, rs))
    return out


def _exc_dir():
    d = _pe.OPTIONAL_HEADER.DATA_DIRECTORY[3]
    return d.VirtualAddress, d.Size


_FUNCS = None


def _func_table():
    """RUNTIME_FUNCTION array {BeginAddress, EndAddress, UnwindData} sorted by BeginAddress."""
    global _FUNCS
    if _FUNCS is None:
        va, size = _exc_dir()
        o = rva2off(va)
        import numpy as np
        arr = np.frombuffer(DATA[o:o + size].tobytes(), dtype=np.uint32).reshape(-1, 3)
        arr = arr[arr[:, 0] != 0]
        _FUNCS = arr[np.argsort(arr[:, 0])]
    return _FUNCS


def func_base_of(site_rva):
    """Function base RVA of the function whose [Begin, End) contains site_rva, via the exception directory."""
    import numpy as np
    p = _func_table()
    i = int(np.searchsorted(p[:, 0], site_rva, side='right')) - 1
    if i >= 0 and p[i, 0] <= site_rva < p[i, 1]:
        return int(p[i, 0])
    return None


def locate_string(needle):
    """Unique RVA of the NUL-terminated byte string in the read-only data sections, or None if absent or
    not unique (the runtime resolver requires exactly one occurrence)."""
    if isinstance(needle, str):
        needle = needle.encode('latin1')
    want = needle + b'\0'
    hit = None
    for n, va, vs, ro, rs in data_sections():
        blob = bytes(DATA[ro:ro + rs])
        start = 0
        while True:
            i = blob.find(want, start)
            if i < 0:
                break
            if hit is not None:
                return None
            hit = va + i
            start = i + 1
    return hit


def xref_sites_many(target_rvas):
    """One pass over the code sections: {target_rva: [site_rva, ...]} for every rel32 displacement that
    RIP-relative references one of target_rvas. A single pass keeps the scan over the ~90 MB code section
    off the per-string hot path."""
    import numpy as np
    want = np.array(sorted(set(int(t) for t in target_rvas)), dtype=np.int64)
    out = {int(t): [] for t in want}
    if len(want) == 0:
        return out
    for n, va, vs, ro, rs in code_sections():
        b = np.frombuffer(bytes(DATA[ro:ro + rs]), dtype=np.uint8).astype(np.int64)
        m = len(b)
        if m < 4:
            continue
        d = b[0:m - 3] | (b[1:m - 2] << 8) | (b[2:m - 1] << 16) | (b[3:m] << 24)
        d = np.where(d >= 1 << 31, d - (1 << 32), d)
        pos = np.arange(0, m - 3, dtype=np.int64)
        tgt = va + pos + 4 + d
        mask = np.isin(tgt, want)
        for p_, t_ in zip(pos[mask], tgt[mask]):
            out[int(t_)].append(va + int(p_))
    return out


def xref_sites(string_rva):
    """RVAs of every rel32 displacement in the code sections that RIP-relative references string_rva."""
    return xref_sites_many([string_rva])[int(string_rva)]


_MD = Cs(CS_ARCH_X86, CS_MODE_64)
_MD.detail = True


def _read_cstr(rva, maxlen=512, minlen=5):
    o = rva2off(rva)
    if o is None:
        return None
    b = bytes(DATA[o:o + maxlen])
    z = b.find(b'\0')
    if z < minlen:
        return None
    s = b[:z]
    return s if all(32 <= c < 127 for c in s) else None


def _func_string_refs(func_rva):
    """Printable C strings in read-only data that the function RIP-relative references."""
    p = _func_table()
    import numpy as np
    i = int(np.searchsorted(p[:, 0], func_rva, side='right')) - 1
    if i < 0 or p[i, 0] != func_rva:
        return []
    start, end = int(p[i, 0]), int(p[i, 1])
    o = rva2off(start)
    code = bytes(DATA[o:o + (end - start)])
    out = []
    for ins in _MD.disasm(code, IMAGE_BASE + start):
        for op in ins.operands:
            if op.type == X86_OP_MEM and op.mem.base == X86_REG_RIP:
                trva = ins.address + ins.size + op.mem.disp - IMAGE_BASE
                sec = sec_of(trva)
                if sec and not _is(sec, MEM_EXECUTE):
                    cs = _read_cstr(trva)
                    if cs:
                        out.append((trva, cs))
    return out


def anchor_xref_map(func_rvas):
    """Precompute, in one code-section pass, {string_rva: set(func_base)} for every C string referenced by
    any function in func_rvas. Lets best_anchor_string decide uniqueness without re-scanning per string."""
    locs = {}
    for f in func_rvas:
        for _, cs in _func_string_refs(f):
            if cs not in locs:
                locs[cs] = locate_string(cs)
    sites = xref_sites_many([r for r in locs.values() if r is not None])
    funcmap = {}
    for r, ss in sites.items():
        fs = {func_base_of(s) for s in ss}
        fs.discard(None)
        funcmap[r] = fs
    return locs, funcmap


def best_anchor_string(func_rva, locs=None, funcmap=None):
    """The longest C string that is referenced only from inside func_rva and is byte-unique in read-only
    data, so it unambiguously locates this one function on any build. None if the function has none.
    Pass the anchor_xref_map() result to avoid a per-string code-section scan."""
    if locs is None:
        locs, funcmap = anchor_xref_map([func_rva])
    cands = sorted({cs for _, cs in _func_string_refs(func_rva)}, key=lambda s: -len(s))
    for cs in cands:
        srva = locs.get(cs)
        if srva is not None and funcmap.get(srva) == {func_rva}:
            return cs
    return None


def apply_resolve(match_va, anchor, resolve):
    """Reproduce native/src/core/manager.cpp resolve_binding: match + anchor/resolve -> target VA."""
    kind = resolve.get('kind', 'addr')
    at = match_va + int(str(anchor), 0)
    if kind == 'addr':
        return at
    o = rva2off(at - IMAGE_BASE)
    rel = int.from_bytes(bytes(DATA[o:o + 4]), 'little', signed=True)
    if kind == 'rel32':
        return at + 4 + rel
    if kind == 'riprel':
        return match_va + int(str(resolve['insn']), 0) + int(str(resolve['len']), 0) + rel
    raise ValueError('unknown resolve kind ' + kind)


def resolve_via_anchor(anchor_string, func_offset, anchor, resolve):
    """Full runtime algorithm: string -> xref -> function base -> match -> target VA. None on any failure."""
    srva = locate_string(anchor_string)
    if srva is None:
        return None, 'string not found or not unique'
    bases = {func_base_of(s) for s in xref_sites(srva)}
    bases.discard(None)
    if len(bases) != 1:
        return None, f'xref maps to {len(bases)} functions'
    func_rva = next(iter(bases))
    match_va = IMAGE_BASE + func_rva + int(str(func_offset), 0)
    return apply_resolve(match_va, anchor, resolve), func_rva
