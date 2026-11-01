import sys
sys.path.insert(0, __import__('os').path.dirname(__import__('os').path.abspath(__file__)))
from pe_index import *
from capstone import *
md=Cs(CS_ARCH_X86, CS_MODE_64); md.detail=False
va=int(sys.argv[1],16); n=int(sys.argv[2],16)
o=rva2off(va-BASE); b=bytes(DATA[o:o+n])
for i in md.disasm(b, va):
    print(f'{i.address:x}: {i.bytes.hex():24s} {i.mnemonic} {i.op_str}')
