import sys, re
sys.path.insert(0, __import__('os').path.dirname(__import__('os').path.abspath(__file__)))
from pe_index import *
from capstone import *
from capstone.x86 import *
md=Cs(CS_ARCH_X86, CS_MODE_64); md.detail=True
va0, code = code_bytes('.xtls'); CODE=bytes(code)
def pattern(va, maxlen=96):
    o=rva2off(va-BASE); b=bytes(DATA[o:o+maxlen+16])
    pat=[]
    for i in md.disasm(b, va):
        bs=list(i.bytes); mask=[True]*len(bs)
        # wildcard rip-relative displacement and rel branch targets
        if i.disp_offset and (any(op.type==X86_OP_MEM and op.mem.base==X86_REG_RIP for op in i.operands)):
            for k in range(i.disp_offset, i.disp_offset+i.disp_size): mask[k]=False
        if i.group(X86_GRP_JUMP) or i.group(X86_GRP_CALL):
            if i.imm_offset and i.imm_size>=4:
                for k in range(i.imm_offset, i.imm_offset+i.imm_size): mask[k]=False
        pat += [(x if m else None) for x,m in zip(bs,mask)]
        if len(pat)>=maxlen: break
    return pat
def to_re(pat): return re.compile(b''.join(re.escape(bytes([x])) if x is not None else b'.' for x in pat), re.S)
def find_unique(va, start=16, maxlen=96, offset_into=0):
    pat=pattern(va, maxlen)
    for n in range(start, len(pat)+1, 4):
        p=pat[:n]
        hits=[m.start() for m in to_re(p).finditer(CODE)]
        if len(hits)==1:
            return ' '.join('??' if x is None else f'{x:02X}' for x in p), hits[0]+va0+BASE
    return None, len(hits)
for a in sys.argv[1:]:
    name, v = a.split('=') if '=' in a else (a,a)
    va=int(v,16)
    s,h=find_unique(va)
    print(f'{name} 0x{va:x}: {s}  (unique hit {h if isinstance(h,int) and h<100 else hex(h) if h else h})')
