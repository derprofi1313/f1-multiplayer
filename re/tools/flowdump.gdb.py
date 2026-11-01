import gdb

STATE_FLOW_UPDATE = 0x144423CE0
STATE_FLOW_LINK_LOOKUP_DONE = 0x144424F08


def rd(addr, size):
    return bytes(gdb.selected_inferior().read_memory(addr, size))


def u32(a):
    return int.from_bytes(rd(a, 4), "little")


def u64(a):
    return int.from_bytes(rd(a, 8), "little")


def name_of(s):
    ptr, ln, h = u64(s + 0x28), u64(s + 0x20), u32(s + 0xB0)
    raw = rd(ptr, min(ln, 96)).split(b"\0")[0] if ptr and ln else b""
    return "%08x %s" % (h, raw.decode("latin1"))


def children(s):
    vec = u64(s + 0xC0)
    if not vec:
        return []
    b, e = u64(vec + 8), u64(vec + 0x10)
    return [u64(p) for p in range(b, e, 8)]


def links(s):
    n = min(u32(s + 0xB30), 16)
    return [(u32(s + 0xAF0 + 4 * i), u32(s + 0xF0 + 0xA0 * i + 0x98)) for i in range(n)]


def root_of(s):
    while u64(s + 0xC10):
        s = u64(s + 0xC10)
    return s


def dump(s, depth=0):
    pad = "  " * depth
    print("%s%s @%x status=%d" % (pad, name_of(s), s, u32(s + 0xB8)))
    for lid, tgt in links(s):
        print("%s  link %08x -> %08x" % (pad, lid, tgt))
    for c in children(s):
        dump(c, depth + 1)


class FlowDump(gdb.Command):
    def __init__(self):
        super().__init__("flowdump", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        dump(root_of(int(gdb.parse_and_eval(arg or "$rcx"))))


FlowDump()
gdb.execute("break *0x%x" % STATE_FLOW_LINK_LOOKUP_DONE)
gdb.execute("commands\nsilent\nprintf \"state %x link %x -> target %x\\n\", $rdi, *(unsigned*)($rsp+0x40), $eax\ncontinue\nend")
